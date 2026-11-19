# Authentication & Authorization

How users and services prove who they are, and how the APIs decide what they may do.

## Overview

| Piece | Role | Where it is defined |
|---|---|---|
| **Authelia** | OpenID Connect provider at `https://auth.k12n.com` | `gitops/apps/authelia/` |
| **oauth2-proxy** | Login gateway: holds the browser session and hands the user's access token to the APIs | `gitops/apps/oauth2-proxy/` |
| **Traefik** | Routes requests and calls oauth2-proxy through a ForwardAuth middleware | `gitops/apps/traefik-middlewares/`, `gitops/apps/traefik-routes/` |
| **Backend APIs** | Validate the access token and check its scopes | each app's config under `gitops/apps/<app>/` |

```
Browser ──► Cloudflare Tunnel ──► Traefik ──► ForwardAuth ──► oauth2-proxy ──► Authelia
                                      │            (session cookie → access token)
                                      ▼
                                 backend API  (validates the token, checks scopes)
```

TLS ends at Cloudflare. Traffic inside the cluster is HTTP.

## Authelia

Authelia is configured entirely from Git. There is no admin UI.

- **Configuration**: `gitops/apps/authelia/configuration.yaml` holds every OIDC client,
  scope, token lifetime and access rule.
- **Users**: a file backend with argon2id hashes, stored as a SealedSecret
  (`authelia-users-sealed.yaml`). Password reset is disabled; to change a password, seal
  a new users file.
- **Storage**: a local SQLite file and a `k12n.com` session cookie. There is no identity
  database to run or back up.
- **Login policy**: one factor (password). Four failed attempts within two minutes ban
  the account for five minutes.

Token lifetimes: access token 1 hour, ID token 1 hour, refresh token 90 days.

### Scopes

Authorization is scope-based. There are no groups or roles.

| Scope | Grants |
|---|---|
| `read:energy` | Energy readings and aggregates |
| `read:heatpump` | Heat pump telemetry |
| `read:temperature` | Temperature and humidity readings |
| `read:settings` / `write:settings` | Read and change heat pump settings |
| `read:plugs` / `write:plugs` | Read and switch smart plugs, and manage their schedules |
| `read:spotprice` | Electricity spot prices |

### Clients

| Client | Type | Used by | Access token |
|---|---|---|---|
| `oauth2-proxy` | confidential | The browser login for `homelab.k12n.com` and `gasa.k12n.com` | JWT (RS256) |
| `heatpump-web` | public, PKCE | The SPA's own OIDC flow, including local development on `localhost:5173` | opaque |
| `homelab-macos` | public, PKCE | The native macOS and iOS apps | JWT (RS256) |
| `backstage` | confidential | Backstage sign-in | opaque |
| `redpanda-console` | confidential | Redpanda Console sign-in | opaque |
| `homelab-chat-agent` | client credentials | homelab-chat-api calling the other APIs as itself | opaque |
| `homelab-api-backend` | client credentials | homelab-api's token introspection calls | opaque |
| `homelab-settings-api-backend` | client credentials | homelab-settings-api's token introspection calls | opaque |
| `traefikoidc` | confidential | Nothing. Left from an earlier Traefik OIDC plugin setup; its redirect hosts have no routes | — |

Access tokens are opaque unless a client sets `access_token_signed_response_alg`. The
two clients whose tokens reach APIs that validate by JWKS get JWTs.

To add or change a client, edit `configuration.yaml`. Client secrets are stored as
pbkdf2 hashes in that file; the plain value goes into the consuming app's SealedSecret.

## How a request is authenticated

### Browser sessions

1. The SPA is served without authentication. It contains no data.
2. Its first API call hits a route with the `oauth2-proxy-auth` ForwardAuth middleware.
   Without a session, oauth2-proxy answers 401.
3. The SPA sends the browser to `/oauth2/start`, oauth2-proxy redirects to Authelia,
   the user logs in, and oauth2-proxy sets the `_oauth2_proxy` cookie on `.k12n.com`.
4. On later API calls the middleware validates the cookie and adds headers to the
   request before it reaches the backend:

   | Header | Content |
   |---|---|
   | `Authorization` | `Bearer <Authelia access token>` |
   | `X-Auth-Request-User`, `X-Auth-Request-Email` | The logged-in user |
   | `X-Auth-Request-Access-Token` | The same access token |

5. The backend validates the token and checks the scope the endpoint needs.

### Bearer tokens

Native apps and service accounts send `Authorization: Bearer <token>` themselves. Those
requests match higher-priority routes that skip oauth2-proxy, and the backend validates
the token directly.

### WebSocket

`/ws` has no ForwardAuth. The client passes its access token when connecting and
energy-ws validates it.

## How each backend validates

| Service | Validation | Notes |
|---|---|---|
| homelab-api | JWKS first, introspection for opaque tokens | Read-only; checks `read:*` scopes |
| homelab-settings-api | JWKS first, introspection for opaque tokens | Checks `read:settings`, `write:settings`, `read:plugs`, `write:plugs` |
| energy-ws | JWKS | Token supplied by the WebSocket client |
| spotprice-api | JWKS | Reached with bearer tokens from the native app |
| homelab-chat-api | Behind ForwardAuth | Calls the other APIs with its own client-credentials token |
| gasa-api | Trusts `X-Auth-Request-*` headers | See below |

JWKS endpoint: `https://auth.k12n.com/jwks.json`. Introspection endpoint:
`https://auth.k12n.com/api/oidc/introspection`.

### gasa is different

Everything under `gasa.k12n.com/api` sits behind ForwardAuth, and gasa-api does not
validate tokens. It trusts the `X-Auth-Request-*` headers that oauth2-proxy sets, and
decides who is an admin from an email allowlist in
`gitops/apps/gasa/configs/gasa-api-config.yaml`. This is safe only because no route
reaches gasa-api without passing the middleware, with one deliberate exception: `/ical`
is public and guarded by a secret token in the path, since calendar apps cannot log in.

## Routes on homelab.k12n.com

Higher priority wins.

| Priority | Match | Auth | Backend |
|---|---|---|---|
| 210 | `/api`, method `OPTIONS` | none (CORS preflight) | homelab-api |
| 200 | `/oauth2` | none (login endpoints) | oauth2-proxy |
| 180 | `/admin/backstage` | ForwardAuth | backstage |
| 170 | `/admin/redpanda` | ForwardAuth | Redpanda Console |
| 150 | `/ws` | token checked by the service | energy-ws |
| 135 | `/api/v1/spotprice` with a bearer token | token checked by the service | spotprice-api |
| 130 | `/api/v1/heatpump/settings`, `/api/v1/plugs` with a bearer token | token checked by the service | homelab-settings-api |
| 120 | `/api/v1` with a bearer token | token checked by the service | homelab-api |
| 100 | `/api/v1/chat` | ForwardAuth | homelab-chat-api |
| 100 | `/api/v1/heatpump/settings`, `/api/v1/plugs` | ForwardAuth | homelab-settings-api |
| 90 | `/api` | ForwardAuth | homelab-api |
| 10 | everything else | none (static SPA) | heatpump-web |

Other hosts: `auth.k12n.com` is Authelia itself. `grafana.k12n.com` and
`pihole.k12n.com` use their applications' built-in logins. `granbok.k12n.com` is
LAN-only and does not go through Traefik.

## CORS

The `cors` middleware (`gitops/apps/traefik-middlewares/cors.yaml`) allows
`https://homelab.k12n.com`, `http://localhost:5173` and `http://localhost:8080`.
Preflight `OPTIONS` requests carry no credentials, so they have their own route at
priority 210 that skips authentication.

## Secrets

All are SealedSecrets next to the app that uses them.

| Secret | Namespace | Contains |
|---|---|---|
| `authelia-secrets` | `authelia` | Session, storage and OIDC signing keys |
| `authelia-users` | `authelia` | The users file |
| `oauth2-proxy-secrets` | `oauth2-proxy` | Client secret and cookie secret |
| `homelab-api-introspection` | `homelab-api` | Client ID and secret for introspection |
| `homelab-settings-api-introspection` | `homelab-settings` | Client ID and secret for introspection |

## Exception: Homebridge

HomeKit commands from Homebridge are published straight to Mosquitto and do not pass
through any of this. See "Write Paths to Devices" in [CLAUDE.md](../CLAUDE.md).

## Troubleshooting

```bash
kubectl logs -n authelia deployment/authelia
kubectl logs -n oauth2-proxy deployment/oauth2-proxy
curl -s https://auth.k12n.com/.well-known/openid-configuration | jq .issuer
```

| Symptom | Likely cause |
|---|---|
| Login loops back to the login page | The oauth2-proxy cookie is not being set: check the cookie domain and that the redirect URI is listed for the `oauth2-proxy` client |
| API returns 401 right after login | The backend cannot reach the JWKS or introspection endpoint, or the introspection client secret is wrong |
| API returns 403 | The token lacks the scope for that endpoint. The scopes a session gets are the `--scope` list in the oauth2-proxy Deployment, limited to what the client allows |
| `invalid_client` during login | The client secret in the app's SealedSecret does not match the hash in `configuration.yaml` |
| Authelia does not start after a config change | A configuration error; the log names the key |

After changing `configuration.yaml`, restart Authelia: ConfigMaps are not reloaded
automatically.

```bash
kubectl rollout restart deployment/authelia -n authelia
```
