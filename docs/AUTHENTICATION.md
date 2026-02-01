# Authentication & Authorization Architecture

## Table of Contents
- [Overview](#overview)
- [Architecture Diagram](#architecture-diagram)
- [Components](#components)
- [Authentication Flows](#authentication-flows)
- [Security Considerations](#security-considerations)
- [Configuration Details](#configuration-details)
- [Troubleshooting](#troubleshooting)
- [Migration Notes](#migration-notes)

---

## Overview

The homelab uses a modern, cloud-native authentication stack built on **Authentik** (Identity Provider), **traefikoidc** (native Traefik OIDC plugin), and **Traefik** (ingress controller with middleware support).

### High-Level Architecture

```
                         Internet
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Cloudflare Tunnel                         │
│                         (TLS Termination)                        │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Traefik Ingress                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Security   │  │     CORS     │  │ traefikoidc  │          │
│  │   Headers    │  │  Middleware  │  │   Plugin     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└────┬────────────────────┬────────────────────┬──────────────────┘
     │                    │                    │
     ▼                    ▼                    ▼
┌─────────┐         ┌──────────┐        ┌──────────────┐
│Authentik│◄────────│traefikoidc│───────│   Protected  │
│  (IdP)  │  OIDC   │  Session │       │   Services   │
└─────────┘         └──────────┘        └──────────────┘
     │                                        │
     ▼                                        ▼
┌──────────┐                           ┌──────────────┐
│PostgreSQL│                           │ Backend APIs │
└──────────┘                           └──────────────┘
```

### Key Features

- **Single Sign-On (SSO)**: Centralized authentication through Authentik
- **OAuth2/OIDC**: Industry-standard protocols for secure authentication
- **Session-Based Auth**: traefikoidc manages encrypted session cookies
- **Native Traefik Plugin**: No separate proxy deployment required
- **Sealed Secrets**: Encrypted secrets stored in Git (cluster-specific encryption)
- **Blueprint Automation**: Infrastructure-as-code for identity configuration

---

## Components

### 1. Authentik (Identity Provider)

**Version**: 2024.12.1
**Namespace**: `authentik`
**Role**: Central identity provider and OAuth2/OIDC authorization server

#### Services
- **authentik-server**: Main application (ports 9000 HTTP, 9443 HTTPS)
- **authentik-worker**: Background task processor
- **PostgreSQL 16**: Identity database (5Gi storage)
- **Redis 7**: Session cache and message broker

#### Key Endpoints
- `/application/o/authorize/` - OAuth2 authorization endpoint
- `/application/o/token/` - Token exchange endpoint
- `/application/o/userinfo/` - User information endpoint
- `/.well-known/openid-configuration` - OIDC discovery endpoint
- `/-/health/live/` - Liveness health check
- `/-/health/ready/` - Readiness health check

#### Configured Applications

| Application | Client Type | Auth Method | Scopes |
|-------------|-------------|-------------|--------|
| **traefikoidc** | Confidential | Session cookies via traefikoidc plugin | `openid`, `profile`, `email` |
| **heatpump-web-leptos** | Public (SPA) | JWT tokens via SPA OIDC flow | `openid`, `profile`, `email`, `read:energy`, `read:heatpump`, `read:settings`, `write:settings` |

#### Blueprint Automation

Authentik configuration is managed through **blueprints** - YAML files that define OAuth2 providers, applications, and scopes. These are automatically applied on every Flux sync via a Kubernetes Job.

**Blueprint Application Process**:
1. Job starts after Authentik is healthy
2. Waits for `/-/health/ready/` endpoint
3. Runs `ak apply_blueprint` command
4. Applies all blueprints from `/blueprints/custom/`
5. Cleans up after 5 minutes (TTL: 300s)

**Blueprint Files**:
- `blueprint-traefikoidc.yaml` - traefikoidc plugin authentication
- `blueprint-heatpump-web.yaml` - Leptos frontend SPA authentication (JWT-based)

#### Resource Limits

| Component | CPU Request | Memory Request | CPU Limit | Memory Limit |
|-----------|-------------|----------------|-----------|--------------|
| Server | 250m | 512Mi | 1000m | 1Gi |
| Worker | 250m | 512Mi | 1000m | 1Gi |
| PostgreSQL | 100m | 256Mi | 1000m | 512Mi |
| Redis | 50m | 64Mi | 500m | 256Mi |

---

### 2. traefikoidc (Native Traefik OIDC Plugin)

**Version**: v0.7.10
**Repository**: [github.com/lukaszraczylo/traefikoidc](https://github.com/lukaszraczylo/traefikoidc)
**Namespace**: `traefik` (runs as Traefik plugin)
**Role**: Native OIDC authentication middleware for Traefik

#### How It Works

traefikoidc is a Traefik middleware plugin that handles the complete OIDC authentication flow natively within Traefik:

1. **Intercepts unauthenticated requests** to protected routes
2. **Redirects to Authentik** for user authentication
3. **Handles the callback** and exchanges authorization code for tokens
4. **Creates encrypted session cookies** to maintain authentication state
5. **Passes user information** to backend services via HTTP headers

#### Configuration

```yaml
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: traefikoidc-auth
  namespace: traefik
spec:
  plugin:
    traefikoidc:
      providerURL: https://authentik.k12n.com/application/o/traefikoidc/
      clientID: traefikoidc
      clientSecretFile: /etc/traefik/secrets/client-secret
      sessionEncryptionKeyFile: /etc/traefik/secrets/session-encryption-key
      callbackURL: /oidc/callback
      forceHTTPS: true
      logLevel: info
      scopes:
        - openid
        - profile
        - email
      excludedURLs:
        - /health
        - /ready
      sessionMaxAge: 86400
```

#### Configuration Options

| Parameter | Value | Purpose |
|-----------|-------|---------|
| `providerURL` | `https://authentik.k12n.com/application/o/traefikoidc/` | OIDC discovery endpoint |
| `clientID` | `traefikoidc` | OAuth2 client identifier |
| `clientSecretFile` | `/etc/traefik/secrets/client-secret` | Path to client secret |
| `sessionEncryptionKeyFile` | `/etc/traefik/secrets/session-encryption-key` | 32+ byte encryption key |
| `callbackURL` | `/oidc/callback` | OAuth redirect path |
| `forceHTTPS` | `true` | Required for TLS termination at Cloudflare |
| `sessionMaxAge` | `86400` | Session lifetime (24 hours) |
| `scopes` | `openid`, `profile`, `email` | OIDC scopes requested |
| `excludedURLs` | `/health`, `/ready` | Paths bypassing authentication |
| `logLevel` | `info` | Logging verbosity |

#### Plugin Features

**Security Features**:
- PKCE (Proof Key for Code Exchange) support
- JTI-based replay attack detection
- Encrypted session cookies
- Automatic token refresh with configurable grace period

**Access Control** (available but not currently used):
- Email domain restrictions (`allowedDomains`)
- Individual user allowlists (`allowedUsers`)
- Role/group-based access control (`allowedRoles`, `allowedGroups`)

**Session Management**:
- Bounded in-memory cache with LRU eviction
- Automatic cleanup of expired sessions
- Optional Redis backend for multi-replica deployments

#### Headers Passed to Backends

traefikoidc passes authenticated user information to backend services via HTTP headers:

| Header | Content |
|--------|---------|
| `X-Forwarded-User` | User email or identifier |
| `X-User-Groups` | User group memberships |
| `X-User-Roles` | User role assignments |

#### Secrets (Sealed)

Secrets are stored in `traefikoidc-secrets-sealed.yaml` and mounted at `/etc/traefik/secrets/`:

- `client-secret` - Authentik OIDC client secret
- `session-encryption-key` - 32+ byte key for session cookie encryption

---

### 3. Traefik (Ingress Controller)

**Version**: 38.0.1 (Helm Chart)
**Namespace**: `traefik`
**Role**: Ingress controller with middleware-based authentication

#### Configuration

```yaml
Service Type: ClusterIP (Cloudflare Tunnel handles external access)
Ports:
  - web: 8000 (HTTP)
  - websecure: 8443 (HTTPS)

Providers:
  - kubernetesCRD: enabled (IngressRoute, Middleware)
  - kubernetesIngress: disabled

Experimental:
  plugins:
    traefikoidc:
      moduleName: github.com/lukaszraczylo/traefikoidc
      version: v0.7.10
```

#### Middlewares

**1. traefikoidc Middleware** (`traefikoidc-auth`)

Handles OIDC authentication natively in Traefik. Applied to protected routes.

**2. Security Headers Middleware** (`security-headers`)

```yaml
headers:
  browserXssFilter: true          # Enable XSS protection
  contentTypeNosniff: true        # Prevent MIME sniffing
  frameDeny: true                 # Deny iframe embedding
  sslRedirect: false              # Cloudflare handles TLS
  customRequestHeaders:
    X-Forwarded-Proto: "https"    # Preserve HTTPS scheme
```

**3. CORS Middleware** (`cors`)

```yaml
accessControlAllowOriginList:
  - https://heatpump.k12n.com
  - https://heatpump-leptos.k12n.com
  - http://localhost:5173
accessControlAllowMethods:
  - GET, POST, PUT, PATCH, DELETE, OPTIONS
accessControlAllowHeaders:
  - Authorization, Content-Type, Accept
accessControlAllowCredentials: true
accessControlMaxAge: 3600
```

**4. HTTPS Scheme Middleware** (`https-scheme`)

```yaml
headers:
  customRequestHeaders:
    X-Forwarded-Proto: "https"
```

Required because Cloudflare Tunnel terminates TLS and internal traffic is HTTP.

---

## Authentication Flows

### Flow 1: Session-Based Authentication (traefikoidc)

**Used by**: `heatpump-web` (React SPA), API routes on `heatpump.k12n.com`

This is the primary authentication flow. The entire SPA and its API routes are protected by traefikoidc middleware.

```
┌──────┐                    ┌────────┐                    ┌─────────┐
│ User │                    │Traefik │                    │Authentik│
│      │                    │ oidc   │                    │  (IdP)  │
└──┬───┘                    └───┬────┘                    └────┬────┘
   │                            │                              │
   │  1. GET https://heatpump.k12n.com/dashboard              │
   │───────────────────────────>│                              │
   │                            │                              │
   │                      2. Check session cookie              │
   │                         (no valid session)                │
   │                            │                              │
   │  3. 302 Redirect to Authentik                            │
   │     https://authentik.k12n.com/application/o/authorize/  │
   │     ?client_id=traefikoidc                               │
   │     &redirect_uri=https://heatpump.k12n.com/oidc/callback│
   │     &scope=openid+profile+email                          │
   │     &response_type=code                                  │
   │     &state=<random>                                      │
   │<───────────────────────────┤                              │
   │                            │                              │
   │  4. User redirected to Authentik login page              │
   │──────────────────────────────────────────────────────────>│
   │                            │                              │
   │  5. User enters credentials                              │
   │──────────────────────────────────────────────────────────>│
   │                            │                              │
   │  6. 302 Redirect to callback with authorization code     │
   │     https://heatpump.k12n.com/oidc/callback?code=ABC123  │
   │<──────────────────────────────────────────────────────────┤
   │                            │                              │
   │  7. GET /oidc/callback?code=ABC123                       │
   │───────────────────────────>│                              │
   │                            │                              │
   │                      8. Exchange code for tokens          │
   │                            │  POST /token                 │
   │                            │  code=ABC123                 │
   │                            │  client_id=traefikoidc       │
   │                            │  client_secret=<secret>      │
   │                            │─────────────────────────────>│
   │                            │                              │
   │                            │  9. ID Token + Access Token  │
   │                            │<─────────────────────────────┤
   │                            │                              │
   │                      10. Create encrypted session cookie  │
   │                            │                              │
   │  11. 302 Redirect to original URL                        │
   │      Set-Cookie: _traefikoidc_session=<encrypted>        │
   │<───────────────────────────┤                              │
   │                            │                              │
   │  12. GET /dashboard (with session cookie)                │
   │───────────────────────────>│                              │
   │                            │                              │
   │                      13. Validate session cookie          │
   │                          (valid - extract user info)      │
   │                            │                              │
   │                            │  14. Forward to backend      │
   │                            │      X-Forwarded-User: email │
   │                            │─────────────────────────────>│
   │                            │                              │
   │  15. Dashboard HTML        │                              │
   │<───────────────────────────┤                              │
```

**Key Points**:
- Full page redirects (works for SPAs because user hasn't loaded app yet)
- Session cookie is HTTP-only and encrypted
- Same cookie works for SPA and API requests (same origin)
- No client-side token storage required

---

### Flow 2: API Request with Session Cookie

**Used by**: API calls from authenticated `heatpump-web` SPA

```
┌──────┐                    ┌────────┐                    ┌─────────┐
│ SPA  │                    │Traefik │                    │ Backend │
│      │                    │ oidc   │                    │   API   │
└──┬───┘                    └───┬────┘                    └────┬────┘
   │                            │                              │
   │  GET /api/v1/energy/latest │                              │
   │  Cookie: _traefikoidc_session=<encrypted>                │
   │───────────────────────────>│                              │
   │                            │                              │
   │                      Validate session cookie              │
   │                      (valid - user authenticated)         │
   │                            │                              │
   │                            │  Forward request             │
   │                            │  X-Forwarded-User: user@mail │
   │                            │  X-User-Roles: user          │
   │                            │─────────────────────────────>│
   │                            │                              │
   │                            │  JSON Response               │
   │  { "power": 1234 }         │<─────────────────────────────┤
   │<───────────────────────────┤                              │
```

**Key Points**:
- Browser automatically sends session cookie (same origin)
- traefikoidc validates cookie and passes user info to backend
- Backend receives user identity via headers
- No JWT validation needed in backend for session-based routes

---

### Flow 3: JWT-Based Authentication (Leptos Frontend)

**Used by**: `heatpump-leptos` frontend (alternative SPA with client-side OIDC)

```
┌──────┐                    ┌────────┐                    ┌─────────┐
│ SPA  │                    │Traefik │                    │ Backend │
└──┬───┘                    └───┬────┘                    └────┬────┘
   │                            │                              │
   │  1. SPA handles OIDC flow directly with Authentik        │
   │     (stores JWT tokens in localStorage)                   │
   │                            │                              │
   │  GET /api/v1/energy/latest │                              │
   │  Authorization: Bearer <JWT from Authentik>              │
   │───────────────────────────>│                              │
   │                            │                              │
   │                            │  Forward request             │
   │                            │  (no traefikoidc middleware) │
   │                            │─────────────────────────────>│
   │                            │                              │
   │                            │                       Validate JWT:
   │                            │                       - Verify signature
   │                            │                       - Check expiration
   │                            │                       - Extract claims
   │                            │                              │
   │                            │  JSON Response               │
   │  { "power": 1234 }         │<─────────────────────────────┤
   │<───────────────────────────┤                              │
```

**Key Points**:
- SPA handles OIDC flow (authorization code with PKCE)
- JWT tokens stored in browser localStorage
- Backend validates JWT using Authentik's JWKS endpoint
- Routes do NOT have traefikoidc middleware (stateless auth)

---

### Flow 4: Hybrid Auth (Bearer Token Priority)

**Used by**: `api.k12n.com` routes (supports both JWT and session)

Routes on `api.k12n.com` support both authentication methods using priority-based routing:

```yaml
# Higher priority - Bearer token route (direct JWT validation by backend)
- match: Host(`api.k12n.com`) && PathPrefix(`/api/v1`) && HeaderRegexp(`Authorization`, `^Bearer .+`)
  priority: 100
  services:
    - name: homelab-api  # Backend validates JWT

# Lower priority - Session-based route (traefikoidc middleware)
- match: Host(`api.k12n.com`) && PathPrefix(`/api/v1`)
  priority: 50
  middlewares:
    - name: traefikoidc-auth
  services:
    - name: homelab-api
```

**Behavior**:
- Requests with `Authorization: Bearer <token>` header → JWT validation by backend
- Requests without Bearer header → traefikoidc session validation

---

## Route Configuration

### heatpump.k12n.com Routes

| Route | Priority | Middleware | Purpose |
|-------|----------|------------|---------|
| `/oidc/callback` | 105 | `traefikoidc-auth` | OIDC callback endpoint |
| `/auth/login` | 100 | `traefikoidc-auth` | Explicit login trigger |
| `/ws` | 95 | `cors` | WebSocket (session cookie) |
| `/api/v1/heatpump/settings` | 95 | `traefikoidc-auth`, `cors` | Settings API |
| `/api/*` | 90 | `traefikoidc-auth`, `cors` | Data API |
| `/*` (catch-all) | default | `traefikoidc-auth` | SPA frontend |

**Note**: The entire SPA is protected by traefikoidc. Users authenticate before the SPA loads.

### api.k12n.com Routes

| Route | Priority | Middleware | Purpose |
|-------|----------|------------|---------|
| `/oidc/callback` | 120 | `traefikoidc-auth` | OIDC callback |
| `/api/v1/*` + Bearer header | 100-110 | `cors`, `security-headers` | JWT auth (backend validates) |
| `/api/v1/*` | 50-90 | `traefikoidc-auth`, `cors` | Session auth |
| `/ws` | - | `cors` | WebSocket (JWT via query param) |
| `/mcp` | - | `cors`, `security-headers` | MCP endpoint (JWT auth) |

---

## Security Considerations

### 1. Secret Management

All secrets are encrypted using **SealedSecrets** (Bitnami) with cluster-specific sealing keys.

**Sealed Secret Locations**:
- `gitops/apps/base/authentik/authentik-secret-sealed.yaml`
- `gitops/apps/base/authentik/postgres-secret-sealed.yaml`
- `gitops/apps/base/traefik-middlewares/traefikoidc-secrets-sealed.yaml`

**Critical Secrets**:
- `AUTHENTIK_SECRET_KEY` - Signs Authentik sessions and tokens
- `client-secret` - traefikoidc OAuth2 client credential
- `session-encryption-key` - Encrypts traefikoidc session cookies
- `POSTGRES_PASSWORD` - Database access

---

### 2. Session Security

**traefikoidc Session Cookies**:

| Setting | Value | Purpose |
|---------|-------|---------|
| `Secure` | true | Only sent over HTTPS |
| `HttpOnly` | true | No JavaScript access |
| `SameSite` | Lax | CSRF protection |
| `Max-Age` | 86400 | 24 hour lifetime |
| Encryption | AES-256 | Session data encrypted |

**Session Storage**:
- In-memory cache with LRU eviction
- Bounded cache prevents memory exhaustion
- Automatic cleanup of expired sessions

---

### 3. TLS/HTTPS Configuration

**Cloudflare Tunnel** handles TLS termination:
- External traffic encrypted via Cloudflare's SSL
- Internal cluster traffic uses HTTP
- `X-Forwarded-Proto: https` header preserves original scheme

**traefikoidc Configuration**:
```yaml
forceHTTPS: true  # Required when TLS terminated upstream
```

This ensures OAuth redirect URIs use `https://` even though internal traffic is HTTP.

---

### 4. CORS Policy

**Allowed Origins**:
- `https://heatpump.k12n.com` (production frontend)
- `https://heatpump-leptos.k12n.com` (Leptos frontend)
- `http://localhost:5173` (local development)

**Credentials**: Allowed (enables session cookie transmission)

---

### 5. Network Isolation

**Internal-Only Services** (no external access):
- PostgreSQL (Authentik database)
- Redis (session cache)

**Public Services** (via Traefik):
- Authentik (SSO login page)
- heatpump-web (frontend SPA, protected)
- homelab-api (protected by traefikoidc or JWT)

---

## Configuration Details

### Environment Variables

**Authentik**:
```bash
AUTHENTIK_REDIS__HOST=authentik-redis
AUTHENTIK_POSTGRESQL__HOST=authentik-postgresql
AUTHENTIK_POSTGRESQL__NAME=authentik
AUTHENTIK_POSTGRESQL__USER=authentik
AUTHENTIK_POSTGRESQL__PASSWORD=<sealed-secret>
AUTHENTIK_SECRET_KEY=<sealed-secret>
AUTHENTIK_BOOTSTRAP_PASSWORD=<sealed-secret>
AUTHENTIK_BOOTSTRAP_TOKEN=<sealed-secret>
```

### Kubernetes Resources

**Namespaces**:
- `authentik` - Identity provider and dependencies
- `traefik` - Ingress controller, middlewares, and traefikoidc plugin

**Custom Resources**:
- `IngressRoute` (traefik.io/v1alpha1) - HTTP routing
- `Middleware` (traefik.io/v1alpha1) - Request processing
- `SealedSecret` (bitnami.com/v1alpha1) - Encrypted secrets

---

## Troubleshooting

### Check Authentik Health

```bash
# Check pod status
kubectl get pods -n authentik

# Check logs
kubectl logs -n authentik -l app.kubernetes.io/name=authentik --tail=100

# Test health endpoint
kubectl exec -n authentik deployment/authentik-server -- curl http://localhost:9000/-/health/ready/
```

### Check traefikoidc

```bash
# Check Traefik logs for OIDC-related messages
kubectl logs -n traefik -l app.kubernetes.io/name=traefik --tail=100 | grep -i oidc

# Verify middleware is configured
kubectl get middleware -n traefik traefikoidc-auth -o yaml

# Check secrets are mounted
kubectl exec -n traefik deployment/traefik -- ls -la /etc/traefik/secrets/
```

### Check Traefik Routes

```bash
# List IngressRoutes
kubectl get ingressroute -A

# Describe specific route
kubectl describe ingressroute frontend-routes -n traefik

# Check Traefik logs
kubectl logs -n traefik -l app.kubernetes.io/name=traefik --tail=100
```

### Test Authentication Flow

```bash
# Test in browser (incognito mode):
# 1. Visit https://heatpump.k12n.com
# 2. Should redirect to Authentik login
# 3. After login, should return to dashboard
# 4. Check browser DevTools > Application > Cookies for session cookie
```

### Check Blueprint Application

```bash
# Check if blueprints were applied
kubectl get jobs -n authentik

# Check blueprint job logs
kubectl logs -n authentik job/authentik-blueprint-apply

# Manually trigger blueprint application
kubectl delete job authentik-blueprint-apply -n authentik
# Flux will recreate it
```

### Common Issues

**Issue**: Redirect loop on login
**Cause**: Cookie domain mismatch or `forceHTTPS` not set
**Fix**: Verify `forceHTTPS: true` in traefikoidc config

**Issue**: 401 on API requests after successful login
**Cause**: Session cookie not being sent (CORS or SameSite issue)
**Fix**: Verify `withCredentials: true` in frontend API client and CORS allows credentials

**Issue**: "Invalid state parameter" error
**Cause**: Session expired during login flow or multiple tabs
**Fix**: Clear browser cookies and try again

**Issue**: OIDC callback fails with "client not found"
**Cause**: Authentik blueprint not applied or client ID mismatch
**Fix**: Check blueprint job logs, verify clientID matches Authentik application

**Issue**: Session expires unexpectedly
**Cause**: `sessionMaxAge` too short or clock skew
**Fix**: Verify `sessionMaxAge` setting (86400 = 24 hours)

---

## File Locations

| Component | Location |
|-----------|----------|
| Authentik Base | `gitops/apps/base/authentik/` |
| Authentik Blueprints | `gitops/apps/base/authentik/blueprint-*.yaml` |
| traefikoidc Middleware | `gitops/apps/base/traefik-middlewares/traefikoidc.yaml` |
| traefikoidc Secrets | `gitops/apps/base/traefik-middlewares/traefikoidc-secrets-sealed.yaml` |
| Traefik Middlewares | `gitops/apps/base/traefik-middlewares/` |
| Traefik Routes | `gitops/apps/base/traefik-routes/` |
| Traefik Controller | `gitops/infrastructure/controllers/traefik/` |
| Sealed Secrets Controller | `gitops/infrastructure/controllers/sealed-secrets/` |

---

## Migration Notes

### oauth2-proxy to traefikoidc Migration (2026-02)

The authentication system has been migrated from oauth2-proxy to the native traefikoidc Traefik plugin.

#### What Changed

| Aspect | Before (oauth2-proxy) | After (traefikoidc) |
|--------|----------------------|---------------------|
| Architecture | Separate proxy deployment | Native Traefik plugin |
| Auth Method | ForwardAuth middleware | Plugin middleware |
| Callback Path | `/oauth2/callback` | `/oidc/callback` |
| Session Storage | oauth2-proxy cookies | traefikoidc encrypted cookies |
| Replicas | 2 pods | 0 (runs in Traefik) |
| Resource Usage | Additional CPU/memory | Minimal (plugin) |

#### SPA Authentication Change

| Aspect | Before | After |
|--------|--------|-------|
| SPA Route | Public (no auth) | Protected by traefikoidc |
| Auth Trigger | SPA makes API call → 401 → redirect | Full page redirect before SPA loads |
| Token Storage | localStorage (JWT) | HTTP-only session cookie |
| API Auth | Bearer token header | Session cookie (automatic) |

#### Why This Change?

1. **XHR Redirect Limitation**: When APIs return 302 redirects to Authentik, XHR/fetch cannot follow cross-origin redirects. Protecting the SPA route itself means authentication happens via full page redirect before the SPA loads.

2. **Simplified Architecture**: No need for client-side OAuth code, token storage, or refresh logic. The browser handles cookies automatically.

3. **Improved Security**: Session cookies are HTTP-only (no XSS access) and encrypted.

#### Deprecated Resources

The following resources are deprecated and will be removed after validation:
- `gitops/apps/base/oauth2-proxy/` directory
- OAuth2 callback routes (`/oauth2/*`) in route files
- `oauth2-proxy-auth` middleware references

---

## References

- [Authentik Documentation](https://docs.goauthentik.io/)
- [traefikoidc Plugin](https://github.com/lukaszraczylo/traefikoidc)
- [Traefik Plugins Documentation](https://doc.traefik.io/traefik/plugins/)
- [Sealed Secrets](https://github.com/bitnami-labs/sealed-secrets)
- [OAuth 2.0 RFC](https://datatracker.ietf.org/doc/html/rfc6749)
- [OpenID Connect](https://openid.net/connect/)

---

**Last Updated**: 2026-02-01
**Author**: Homelab GitOps System
**Status**: Production
