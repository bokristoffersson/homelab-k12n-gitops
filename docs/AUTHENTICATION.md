# Authentication & Authorization Architecture

## Table of Contents
- [Overview](#overview)
- [Architecture Diagram](#architecture-diagram)
- [Components](#components)
- [Authentication Flows](#authentication-flows)
- [Security Considerations](#security-considerations)
- [Configuration Details](#configuration-details)
- [Troubleshooting](#troubleshooting)

---

## Overview

The homelab uses a modern, cloud-native authentication stack built on **Authentik** (Identity Provider), **oauth2-proxy** (authentication gateway), and **Traefik** (ingress controller with middleware support).

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Cloudflare Tunnel                         │
│                         (TLS Termination)                        │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Traefik Ingress                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Security   │  │     CORS     │  │  ForwardAuth │          │
│  │   Headers    │  │  Middleware  │  │  Middleware  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└────┬────────────────────┬────────────────────┬──────────────────┘
     │                    │                    │
     ▼                    ▼                    ▼
┌─────────┐        ┌─────────────┐      ┌──────────────┐
│Authentik│◄───────│ oauth2-proxy│      │   Protected  │
│  (IdP)  │        │  (AuthN GW) │      │   Services   │
└─────────┘        └─────────────┘      └──────────────┘
     │                    │                    │
     ▼                    ▼                    ▼
┌──────────┐        ┌──────────┐        ┌──────────────┐
│PostgreSQL│        │  Redis   │        │ Backend APIs │
└──────────┘        └──────────┘        └──────────────┘
```

### Key Features

- **Single Sign-On (SSO)**: Centralized authentication through Authentik
- **OAuth2/OIDC**: Industry-standard protocols for secure authentication
- **JWT Tokens**: Stateless authentication with cryptographically signed tokens
- **ForwardAuth Pattern**: Traefik delegates authentication to oauth2-proxy
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
- `/application/o/introspect/` - Token validation endpoint
- `/-/health/live/` - Liveness health check
- `/-/health/ready/` - Readiness health check

#### Configured Applications

| Application | Client Type | Redirect URI | Token Validity | Scopes |
|-------------|-------------|--------------|----------------|--------|
| **heatpump-web** | Public (SPA) | `https://heatpump.k12n.com/auth/callback` | 24h access / 30d refresh | `openid`, `profile`, `email`, `read:energy`, `read:heatpump`, `read:settings`, `write:settings` |
| **oauth2-proxy** | Confidential | `https://api.k12n.com/oauth2/callback` | 24h access / 30d refresh | `openid`, `email`, `profile` |
| **kong-api-gateway** | Confidential | `https://api.k12n.com/callback` | 10h access / 30d refresh | `openid`, `email`, `profile` |

#### Blueprint Automation

Authentik configuration is managed through **blueprints** - YAML files that define OAuth2 providers, applications, and scopes. These are automatically applied on every Flux sync via a Kubernetes Job.

**Blueprint Application Process**:
1. Job starts after Authentik is healthy
2. Waits for `/-/health/ready/` endpoint
3. Runs `ak apply_blueprint` command
4. Applies all blueprints from `/blueprints/custom/`
5. Cleans up after 5 minutes (TTL: 300s)

**Blueprint Files**:
- `blueprint-heatpump-web.yaml` - Frontend SPA authentication
- `blueprint-oauth2-proxy.yaml` - OAuth2-proxy service authentication
- `blueprint-kong-api-gateway.yaml` - API gateway authentication

#### Resource Limits

| Component | CPU Request | Memory Request | CPU Limit | Memory Limit |
|-----------|-------------|----------------|-----------|--------------|
| Server | 250m | 512Mi | 1000m | 1Gi |
| Worker | 250m | 512Mi | 1000m | 1Gi |
| PostgreSQL | 100m | 256Mi | 1000m | 512Mi |
| Redis | 50m | 64Mi | 500m | 256Mi |

---

### 2. OAuth2-Proxy (Authentication Gateway)

**Version**: v7.7.1
**Namespace**: `oauth2-proxy`
**Replicas**: 2 (High Availability)
**Role**: Forward authentication proxy for protecting backend services

#### Configuration

```yaml
Provider: OIDC (OpenID Connect)
OIDC Issuer: https://authentik.k12n.com/application/o/oauth2-proxy/
Client ID: oauth2-proxy
Redirect URL: https://api.k12n.com/oauth2/callback
Upstream: static://202  # ForwardAuth mode (returns 202 Accepted)
```

#### Key Settings

| Setting | Value | Purpose |
|---------|-------|---------|
| `cookie-secure` | true | Require HTTPS for cookies |
| `cookie-httponly` | true | Prevent JavaScript cookie access |
| `cookie-samesite` | lax | CSRF protection |
| `skip-provider-button` | true | Auto-redirect to Authentik |
| `pass-authorization-header` | true | Forward auth headers to backend |
| `pass-access-token` | true | Include access token in headers |
| `set-xauthrequest` | true | Set `X-Auth-Request-*` headers |

#### Response Headers Forwarded to Backend

- `X-Auth-Request-User` - Authenticated username
- `X-Auth-Request-Email` - User email address
- `X-Auth-Request-Access-Token` - JWT access token
- `Authorization` - Bearer token header

#### Health Checks

- **Liveness**: `GET /ping` (delay: 10s, period: 10s)
- **Readiness**: `GET /ready` (delay: 5s, period: 5s)

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
```

#### Middlewares

**1. ForwardAuth Middleware** (`oauth2-proxy-auth`)

```yaml
forwardAuth:
  address: http://oauth2-proxy.oauth2-proxy.svc.cluster.local:4180/
  trustForwardHeader: true
  authResponseHeaders:
    - X-Auth-Request-User
    - X-Auth-Request-Email
    - X-Auth-Request-Access-Token
    - Authorization
```

**Purpose**: Delegates authentication to oauth2-proxy before allowing access to protected services.

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
  - http://localhost:5173
accessControlAllowMethods:
  - GET, POST, PUT, PATCH, DELETE, OPTIONS
accessControlAllowHeaders:
  - Authorization, Content-Type, Accept
accessControlExposeHeaders:
  - Authorization
accessControlAllowCredentials: true
accessControlMaxAge: 3600
```

#### IngressRoute Examples

**Authentik (Public - No Auth)**
```yaml
apiVersion: traefik.io/v1alpha1
kind: IngressRoute
metadata:
  name: authentik-routes
  namespace: traefik
spec:
  entryPoints:
    - web
  routes:
    - match: Host(`authentik.k12n.com`)
      middlewares:
        - name: security-headers
      services:
        - name: authentik-server
          namespace: authentik
          port: 9000
```

**API Routes (JWT Protected)**
```yaml
routes:
  - match: Host(`api.k12n.com`) && PathPrefix(`/api/v1`)
    middlewares:
      - name: cors
      - name: security-headers
    services:
      - name: homelab-api
        namespace: homelab-api
        port: 8080
```

**OAuth2 Callbacks (Public)**
```yaml
routes:
  - match: Host(`api.k12n.com`) && PathPrefix(`/oauth2`)
    middlewares:
      - name: cors
    services:
      - name: oauth2-proxy
        namespace: oauth2-proxy
        port: 4180
```

**Frontend (Public - SPA Handles OIDC)**
```yaml
routes:
  - match: Host(`heatpump.k12n.com`)
    services:
      - name: heatpump-web
        namespace: heatpump-web
        port: 80
```

---

## Authentication Flows

### Flow 1: Frontend SPA Authentication (OIDC Authorization Code Flow)

**Used by**: `heatpump-web` frontend application

```
┌──────┐                                  ┌──────────┐                    ┌─────────┐
│ User │                                  │ Frontend │                    │Authentik│
│      │                                  │   SPA    │                    │  (IdP)  │
└──┬───┘                                  └────┬─────┘                    └────┬────┘
   │                                           │                               │
   │  1. Visit https://heatpump.k12n.com     │                               │
   │─────────────────────────────────────────>│                               │
   │                                           │                               │
   │  2. SPA loads, detects no session        │                               │
   │                                           │                               │
   │  3. Redirect to Authentik authorize      │                               │
   │<──────────────────────────────────────────┤                               │
   │                                           │                               │
   │  4. GET /application/o/heatpump-web/authorize/                           │
   │      ?client_id=heatpump-web                                             │
   │      &redirect_uri=https://heatpump.k12n.com/auth/callback               │
   │      &scope=openid+profile+email+read:energy                             │
   │      &response_type=code                                                 │
   │──────────────────────────────────────────────────────────────────────────>│
   │                                           │                               │
   │  5. Login form (if not authenticated)    │                               │
   │<──────────────────────────────────────────────────────────────────────────┤
   │                                           │                               │
   │  6. Submit credentials                   │                               │
   │──────────────────────────────────────────────────────────────────────────>│
   │                                           │                               │
   │  7. Redirect to callback with code       │                               │
   │      https://heatpump.k12n.com/auth/callback?code=ABC123                │
   │<──────────────────────────────────────────────────────────────────────────┤
   │─────────────────────────────────────────>│                               │
   │                                           │                               │
   │                                           │  8. Exchange code for tokens │
   │                                           │  POST /application/o/heatpump-web/token/
   │                                           │  grant_type=authorization_code
   │                                           │  code=ABC123                 │
   │                                           │  client_id=heatpump-web      │
   │                                           │──────────────────────────────>│
   │                                           │                               │
   │                                           │  9. JWT tokens               │
   │                                           │  {                           │
   │                                           │    "access_token": "<JWT>",  │
   │                                           │    "refresh_token": "<JWT>", │
   │                                           │    "expires_in": 86400       │
   │                                           │  }                           │
   │                                           │<──────────────────────────────┤
   │                                           │                               │
   │  10. Store tokens & redirect to app      │                               │
   │<──────────────────────────────────────────┤                               │
   │                                           │                               │
```

**Token Storage**: SPA stores tokens in sessionStorage or localStorage
**Token Usage**: All API requests include `Authorization: Bearer <access_token>`
**Token Validation**: Backend validates JWT signature using Authentik's public key

---

### Flow 2: ForwardAuth Pattern (Optional - Not Currently Active)

**Used when**: Traefik middleware `oauth2-proxy-auth` is applied to routes

```
┌──────┐       ┌────────┐       ┌─────────────┐       ┌─────────┐
│ User │       │Traefik │       │oauth2-proxy │       │ Backend │
└──┬───┘       └───┬────┘       └──────┬──────┘       └────┬────┘
   │               │                    │                   │
   │  GET /api/v1/protected            │                   │
   │──────────────>│                    │                   │
   │               │                    │                   │
   │               │  ForwardAuth       │                   │
   │               │  GET /             │                   │
   │               │  (with headers)    │                   │
   │               │───────────────────>│                   │
   │               │                    │                   │
   │               │  Valid cookie?     │                   │
   │               │  Valid JWT?        │                   │
   │               │                    │                   │
   │               │  If NO:            │                   │
   │  Redirect to  │  302 Redirect      │                   │
   │  Authentik    │<───────────────────┤                   │
   │<──────────────┤                    │                   │
   │               │                    │                   │
   │  [User authenticates via Authentik OIDC flow]         │
   │               │                    │                   │
   │               │  If YES:           │                   │
   │               │  202 OK + Headers  │                   │
   │               │  X-Auth-Request-User: user@example.com│
   │               │  X-Auth-Request-Email: user@example.com
   │               │  Authorization: Bearer <JWT>          │
   │               │<───────────────────┤                   │
   │               │                    │                   │
   │               │  Forward request   │                   │
   │               │  with auth headers │                   │
   │               │────────────────────────────────────────>│
   │               │                    │                   │
   │               │                    │  Validate JWT    │
   │               │                    │  Extract claims  │
   │               │                    │                   │
   │               │  Protected data    │                   │
   │  Response     │<────────────────────────────────────────┤
   │<──────────────┤                    │                   │
```

**Current Status**: ForwardAuth middleware available but not applied to routes
**Reason**: Backend APIs validate JWT tokens directly (stateless authentication)

---

### Flow 3: Direct JWT Validation (Currently Used)

**Used by**: `homelab-api`, `energy-ws`

```
┌──────┐                  ┌─────────────┐                ┌─────────┐
│ SPA  │                  │   Traefik   │                │ Backend │
└──┬───┘                  └──────┬──────┘                └────┬────┘
   │                             │                            │
   │  GET /api/v1/energy/latest │                            │
   │  Authorization: Bearer <JWT from Authentik>             │
   │────────────────────────────>│                            │
   │                             │                            │
   │                             │  Forward request           │
   │                             │  (CORS + Security Headers) │
   │                             │───────────────────────────>│
   │                             │                            │
   │                             │                     Validate JWT:
   │                             │                     - Verify signature
   │                             │                     - Check expiration
   │                             │                     - Extract claims
   │                             │                            │
   │                             │  Protected data            │
   │  JSON response              │<───────────────────────────┤
   │<────────────────────────────┤                            │
```

**Validation Method**: Backend uses Authentik's public JWKS endpoint
**Claims Extracted**: `sub` (user ID), `email`, `scope` (permissions)
**No Session**: Stateless - every request validated independently

---

## Security Considerations

### 1. Secret Management

All secrets are encrypted using **SealedSecrets** (Bitnami) with cluster-specific sealing keys.

**Sealed Secret Locations**:
- `gitops/apps/base/authentik/authentik-secret-sealed.yaml`
- `gitops/apps/base/authentik/postgres-secret-sealed.yaml`
- `gitops/apps/base/authentik/oauth2-proxy-client-secret-sealed.yaml`
- `gitops/apps/base/oauth2-proxy/secrets-sealed.yaml`

**Critical Secrets**:
- `AUTHENTIK_SECRET_KEY` - Signs Authentik sessions and tokens
- `OAUTH2_PROXY_CLIENT_SECRET` - OAuth2 client credential
- `OAUTH2_PROXY_COOKIE_SECRET` - Encrypts session cookies
- `POSTGRES_PASSWORD` - Database access

**Process**:
1. Plain secret created locally (never committed)
2. Encrypted with `kubeseal` using cluster's public key
3. SealedSecret YAML committed to Git
4. sealed-secrets controller decrypts in-cluster

---

### 2. Token Security

**JWT Token Configuration**:

| Application | Token Type | Access Validity | Refresh Validity | Issuer Mode |
|-------------|------------|-----------------|------------------|-------------|
| heatpump-web | JWT | 24 hours | 30 days | per_provider |
| oauth2-proxy | JWT | 24 hours | 30 days | per_provider |
| kong-api-gateway | JWT | 10 hours | 30 days | per_provider |

**JWT Structure**:
```json
{
  "iss": "https://authentik.k12n.com/application/o/heatpump-web/",
  "sub": "user-uuid",
  "aud": "heatpump-web",
  "exp": 1735473600,
  "iat": 1735387200,
  "email": "user@example.com",
  "scope": "openid profile email read:energy read:heatpump"
}
```

**Validation**:
- Signature verified using Authentik's public key (JWKS endpoint)
- Expiration checked (`exp` claim)
- Audience validated (`aud` claim matches application)
- Issuer verified (`iss` matches expected Authentik URL)

---

### 3. TLS/HTTPS Configuration

**Cloudflare Tunnel** handles TLS termination:
- External traffic encrypted via Cloudflare's SSL
- Internal traffic uses HTTP (within cluster)
- `X-Forwarded-Proto: https` header preserves original scheme

**Traefik Configuration**:
```yaml
sslRedirect: false  # Cloudflare already enforced HTTPS
```

**Cookie Security**:
```yaml
cookie-secure: true      # Cookies only sent over HTTPS
cookie-httponly: true    # No JavaScript access
cookie-samesite: lax     # CSRF protection
```

---

### 4. CORS Policy

**Allowed Origins**:
- `https://heatpump.k12n.com` (production frontend)
- `http://localhost:5173` (local development)

**Allowed Methods**: GET, POST, PUT, PATCH, DELETE, OPTIONS
**Allowed Headers**: Authorization, Content-Type, Accept
**Credentials**: Allowed (cookies/auth headers)
**Max Age**: 3600 seconds (1 hour preflight cache)

---

### 5. Network Isolation

**Internal-Only Services** (no external access):
- PostgreSQL (Authentik database)
- Redis (session cache)
- oauth2-proxy (accessed only via Traefik ForwardAuth)

**Public Services** (via Traefik):
- Authentik (SSO login page)
- heatpump-web (frontend SPA)
- homelab-api (protected by JWT validation)

**Service Mesh**:
- All traffic routed through Traefik
- No direct pod-to-pod external access
- ClusterIP services (internal only)

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

**OAuth2-Proxy**:
```bash
OAUTH2_PROXY_CLIENT_SECRET=<sealed-secret>
OAUTH2_PROXY_COOKIE_SECRET=<sealed-secret>
```

### Kubernetes Resources

**Namespaces**:
- `authentik` - Identity provider and dependencies
- `oauth2-proxy` - Authentication gateway
- `traefik` - Ingress controller and middlewares

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

### Check OAuth2-Proxy Status

```bash
# Check pod status
kubectl get pods -n oauth2-proxy

# Check logs
kubectl logs -n oauth2-proxy -l app=oauth2-proxy --tail=100

# Test ping endpoint
kubectl exec -n oauth2-proxy deployment/oauth2-proxy -- curl http://localhost:4180/ping
```

### Verify JWT Token

```bash
# Decode JWT (paste your token)
echo "<JWT_TOKEN>" | cut -d. -f2 | base64 -d | jq

# Check token expiration
echo "<JWT_TOKEN>" | cut -d. -f2 | base64 -d | jq '.exp | todate'
```

### Check Traefik Routes

```bash
# List IngressRoutes
kubectl get ingressroute -A

# Describe specific route
kubectl describe ingressroute api-routes -n traefik

# Check Traefik logs
kubectl logs -n traefik -l app.kubernetes.io/name=traefik --tail=100
```

### Test ForwardAuth Flow

```bash
# Test oauth2-proxy auth endpoint
curl -v http://oauth2-proxy.oauth2-proxy.svc.cluster.local:4180/ \
  -H "Authorization: Bearer <YOUR_JWT>"
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
**Fix**: Check cookie domain and secure settings, verify redirect URIs match

**Issue**: 401 Unauthorized on API requests
**Fix**: Verify JWT token is valid, check backend JWT validation configuration

**Issue**: CORS errors in browser
**Fix**: Verify origin in CORS middleware, check browser console for specific headers

**Issue**: OAuth2-proxy not forwarding headers
**Fix**: Check ForwardAuth middleware `authResponseHeaders` configuration

**Issue**: Sealed secret not decrypting
**Fix**: Verify sealed-secrets controller is running, check sealing key matches cluster

---

## File Locations

| Component | Location |
|-----------|----------|
| Authentik Base | `gitops/apps/base/authentik/` |
| Authentik Overlay | `gitops/apps/homelab/authentik/` |
| OAuth2-Proxy Base | `gitops/apps/base/oauth2-proxy/` |
| OAuth2-Proxy Overlay | `gitops/apps/homelab/oauth2-proxy/` |
| Traefik Middlewares | `gitops/apps/base/traefik-middlewares/` |
| Traefik Routes | `gitops/apps/base/traefik-routes/` |
| Traefik Controller | `gitops/infrastructure/controllers/traefik/` |
| Sealed Secrets Controller | `gitops/infrastructure/controllers/sealed-secrets/` |

---

## References

- [Authentik Documentation](https://docs.goauthentik.io/)
- [OAuth2-Proxy Documentation](https://oauth2-proxy.github.io/oauth2-proxy/)
- [Traefik ForwardAuth](https://doc.traefik.io/traefik/middlewares/http/forwardauth/)
- [Sealed Secrets](https://github.com/bitnami-labs/sealed-secrets)
- [OAuth 2.0 RFC](https://datatracker.ietf.org/doc/html/rfc6749)
- [OpenID Connect](https://openid.net/connect/)

---

**Last Updated**: 2025-12-30
**Author**: Homelab GitOps System
**Status**: Production
