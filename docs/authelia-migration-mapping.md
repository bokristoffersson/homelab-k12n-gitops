# Authentik → Authelia OIDC Client Mapping

Migration of the cluster OIDC provider from **Authentik** (`https://authentik.k12n.com`,
per-provider issuers) to **Authelia 4.39** (`https://auth.k12n.com`, single issuer).

## Endpoints

| Concern | Authentik (per provider) | Authelia (single) |
| --- | --- | --- |
| Issuer | `https://authentik.k12n.com/application/o/<slug>/` | `https://auth.k12n.com` |
| Discovery | `…/application/o/<slug>/.well-known/openid-configuration` | `https://auth.k12n.com/.well-known/openid-configuration` |
| Authorization | `…/application/o/authorize/` | `https://auth.k12n.com/api/oidc/authorization` |
| Token | `…/application/o/token/` | `https://auth.k12n.com/api/oidc/token` |
| UserInfo | `…/application/o/userinfo/` | `https://auth.k12n.com/api/oidc/userinfo` |
| JWKS | `…/application/o/<slug>/jwks/` | `https://auth.k12n.com/jwks.json` |
| Introspection | `…/application/o/introspect/` | `https://auth.k12n.com/api/oidc/introspection` |
| End session | `…/application/o/<slug>/end-session/` | `https://auth.k12n.com/api/oidc/logout` |

All apps that previously held a per-provider issuer/JWKS/introspection URL now point at the
single Authelia endpoints above. Backends validate the `iss` claim against `https://auth.k12n.com`.

## Token format

Opaque access tokens remain the default. JWT (RFC 9068) access tokens are enabled **only** for the
two clients whose tokens are validated by `energy-ws`, which is JWKS-only with no introspection
fallback. Everything else stays opaque and is validated by introspection on the backends.

| Client | Access token | Why |
| --- | --- | --- |
| `oauth2-proxy` | JWT (RS256) | energy-ws validates its tokens via JWKS |
| `homelab-macos` | JWT (RS256) | energy-ws validates its tokens via JWKS |
| all others | opaque | introspection on homelab-api / homelab-settings-api |

## Clients migrated (9)

| client_id | Type | redirect_uris | scopes | consumer / validator |
| --- | --- | --- | --- | --- |
| `heatpump-web` | public + PKCE | `https://homelab.k12n.com/auth/callback`, `http://localhost:5173/auth/callback` | openid, profile, email, offline_access, read:energy, read:heatpump, read:temperature, read:settings, write:settings, read:plugs, write:plugs | SPA → homelab-api / homelab-settings-api (introspection) |
| `homelab-macos` | public + PKCE | `homelab-macos://oauth/callback`, `homelab://oauth/callback`, `http://127.0.0.1:9876/callback` | openid, profile, email, offline_access, read:energy, read:heatpump, read:settings, write:settings | native app → all backends + energy-ws (JWKS) |
| `oauth2-proxy` | confidential | `https://homelab.k12n.com/oauth2/callback` | openid, profile, email, offline_access, read:energy, read:heatpump, read:temperature, read:settings, write:settings, read:plugs, write:plugs | ForwardAuth → energy-ws (JWKS), homelab-settings-api |
| `backstage` | confidential | `https://backstage.k12n.com/api/auth/oidc/handler/frame` | openid, profile, email, offline_access | Backstage OIDC discovery |
| `redpanda-console` | confidential | `https://redpanda.k12n.com/auth/callbacks/oidc` | openid, profile, email | Redpanda Console OIDC |
| `claude-mcp` | confidential, client_credentials | — | openid, profile, email | external Claude MCP client |
| `homelab-chat-agent` | confidential, client_credentials | — | openid, profile, email, read:energy, read:heatpump, read:settings, read:temperature, read:plugs, write:plugs | homelab-chat-api → MCP tools |
| `homelab-api-backend` | confidential | — | openid | homelab-api introspection caller |
| `traefikoidc` | confidential | `https://api.k12n.com/oidc/callback`, `https://heatpump.k12n.com/oidc/callback` | openid, profile, email | Traefik OIDC middleware |

A dedicated confidential client `homelab-settings-api-backend` is added for homelab-settings-api's
introspection calls (Authentik reused another client's credentials; Authelia gets its own).

## Custom scopes

Defined at provider level as pure authorization markers (no mapped claims). When granted they appear
in the token `scope` value and in introspection responses, which is exactly what the backends read.
Backends already accept the standard space-delimited `scope` string, so no custom claims policy is
needed.

`read:energy`, `read:heatpump`, `read:temperature`, `read:settings`, `write:settings`,
`read:plugs`, `write:plugs`.

## Refresh tokens

Authelia only issues refresh tokens when `offline_access` is both granted to the client and
requested by the app. Authentik issued them without it. Clients that previously received refresh
tokens (heatpump-web, homelab-macos, oauth2-proxy, backstage) gain `offline_access` in their scope
list **and** the apps are updated to request it.

## Authorization

All authorization is scope-based; no app makes group-based decisions (Authentik had only
`authentik Admins` and an empty `authentik Read-only`, unused by apps). Human users `johanna` and an
`admin` account migrate to the Authelia file backend (argon2id, sealed). All OIDC clients use
`authorization_policy: one_factor` (no second factor was configured in Authentik).

## Dropped (not migrated — removed with Authentik)

| client_id | Reason |
| --- | --- |
| `kong-api-gateway` | UI-only drift, not in GitOps; `api.k12n.com` Kong gateway not retained |
| `kubernetes-mcp-server` | UI-only drift, Cursor MCP client not retained |
