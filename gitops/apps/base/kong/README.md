# Kong API Gateway (DB-less Mode) with Authentik Integration

Kong API Gateway provides centralized authentication, authorization, and routing for all backend services. It runs in **DB-less mode** with declarative configuration stored in Git, perfect for GitOps workflows.

## Architecture

```
Client Request
     ↓
Cloudflare Tunnel (TLS termination)
     ↓
Kong Gateway (authentication + routing)
     ↓ (token introspection)
Authentik (validate opaque token)
     ↓ (authenticated request with headers)
Backend Services (redpanda-sink, energy-ws, etc.)
```

## Why DB-less Mode?

- **GitOps Native**: All configuration in YAML, version controlled in Git
- **Simpler**: No database, no migrations, no Admin API calls
- **Declarative**: Configuration is immutable and auditable
- **Faster**: No database queries, config loaded in memory
- **Easier to Review**: Config changes visible in Git PRs

## Components

### 1. Kong Gateway
- **Image**: `kong:3.8`
- **Mode**: DB-less (no database required)
- **Config**: Declarative YAML in ConfigMap
- **Ports**:
  - `8000`: HTTP proxy (client traffic)
  - `8443`: HTTPS proxy (client traffic)
  - `8001`: Admin API (read-only in DB-less mode)

### 2. Configuration (config.yaml)
- **Services**: Backend service definitions (redpanda-sink, energy-ws)
- **Routes**: Path-based routing (/api/v1, /ws)
- **Plugins**: OpenID Connect, request-transformer

## Configuration Structure

The declarative config in `config.yaml` defines:

```yaml
services:
  - name: redpanda-sink
    url: http://redpanda-sink.redpanda-sink.svc.cluster.local:8080
    routes:
      - name: api-route
        paths: [/api/v1]
    plugins:
      - name: request-transformer
        config:
          add:
            headers:
              - X-Authenticated-User-Id:$(X-Userinfo-Sub)
              - X-Authenticated-User-Email:$(X-Userinfo-Email)
              - X-Authenticated-Scope:$(X-Userinfo-Scope)

  - name: energy-ws
    url: http://energy-ws.energy-ws.svc.cluster.local:8080
    routes:
      - name: ws-route
        paths: [/ws]

plugins:
  - name: openid-connect
    config:
      issuer: http://authentik-server.authentik.svc.cluster.local:9000/application/o/api-gateway/
      client_id: $(KONG_OIDC_CLIENT_ID)
      client_secret: $(KONG_OIDC_CLIENT_SECRET)
      bearer_only: yes
      introspection_endpoint: http://authentik-server.authentik.svc.cluster.local:9000/application/o/introspect/
```

**Key Features:**
- Environment variable substitution: `$(KONG_OIDC_CLIENT_ID)` from secret
- Global OIDC plugin applies to all routes
- Request transformer adds authenticated user headers to backend requests

## Deployment

### Step 1: Configure Authentik OAuth2 Provider

Before deploying Kong, create an OAuth2 provider in Authentik:

1. Access Authentik UI: `https://authentik.k12n.com/if/admin/`
2. Navigate to **Applications** → **Providers** → **Create**
3. Select **OAuth2/OpenID Provider**

**Provider Configuration:**
- **Name**: `Kong API Gateway`
- **Authorization flow**: `default-provider-authorization-implicit-consent`
- **Client type**: `Confidential`
- **Client ID**: `kong-api-gateway`
- **Client Secret**: Click "Generate" and **copy the secret** (you'll need it next)
- **Redirect URIs**:
  ```
  http://kong-proxy.kong.svc.cluster.local:8000/*
  https://api.k12n.com/*
  ```
- **Signing Key**: Select `authentik Self-signed Certificate`
- **Scopes**: Add the following scopes to the provider:
  - `openid`
  - `profile`
  - `email`
  - `read:energy`
  - `read:heatpump`
  - `read:settings`
  - `write:settings`

4. Click **Create**

**Create Application:**
1. Navigate to **Applications** → **Applications** → **Create**
2. **Name**: `API Gateway`
3. **Slug**: `api-gateway`
4. **Provider**: Select `Kong API Gateway`
5. **Launch URL**: `https://api.k12n.com`
6. Click **Create**

### Step 2: Create Custom Scopes (Optional)

If custom scopes don't exist, create them:

1. Navigate to **Customization** → **Property Mappings** → **Create** → **Scope Mapping**

For each scope (`read:energy`, `read:heatpump`, `read:settings`, `write:settings`):
- **Name**: `Scope <scope-name>`
- **Scope name**: `<scope-name>`
- **Expression**:
```python
return {
    "<scope-name>": True
}
```

2. Edit the `Kong API Gateway` provider and add these scope mappings

### Step 3: Create Sealed Secret

Using the Client Secret from Step 1:

```bash
# Use the client secret from Authentik
AUTHENTIK_CLIENT_ID="kong-api-gateway"
AUTHENTIK_CLIENT_SECRET="<paste client secret from Authentik>"

# Create sealed secret
kubectl create secret generic kong-oidc-secret \
  --namespace=kong \
  --from-literal=CLIENT_ID="$AUTHENTIK_CLIENT_ID" \
  --from-literal=CLIENT_SECRET="$AUTHENTIK_CLIENT_SECRET" \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace kong \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/kong/oidc-secret-sealed.yaml

# Verify sealed secret was created
cat gitops/apps/base/kong/oidc-secret-sealed.yaml
```

### Step 4: Update Kustomization

Uncomment the sealed secret in `kustomization.yaml`:

```yaml
resources:
  # ...
  - oidc-secret-sealed.yaml  # Uncomment this line
```

### Step 5: Create Homelab Overlay

```bash
mkdir -p gitops/apps/homelab/kong

cat > gitops/apps/homelab/kong/kustomization.yaml <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base/kong
EOF
```

### Step 6: Add to Flux Apps

Edit `gitops/clusters/homelab/apps.yaml` and add Kong (alphabetically after authentik):

```yaml
---
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: kong
  namespace: flux-system
spec:
  interval: 10m0s
  sourceRef:
    kind: GitRepository
    name: flux-system
  path: ./gitops/apps/homelab/kong
  prune: true
  wait: true
  timeout: 5m
  dependsOn:
    - name: authentik  # Kong depends on Authentik
```

### Step 7: Deploy

```bash
# Commit and push
git add gitops/apps/base/kong gitops/apps/homelab/kong gitops/clusters/homelab/apps.yaml
git commit -m "feat: add Kong API Gateway (DB-less mode) with Authentik OIDC"
git push origin main

# Wait for pod to be ready
kubectl wait --for=condition=ready --timeout=300s pod -l app=kong-gateway -n kong

# Verify pod is running
kubectl get pods -n kong
# Expected:
# kong-gateway-xxxxx     1/1     Running     0

# Check Kong status
kubectl exec -n kong deploy/kong-gateway -- kong health
```

## Updating Configuration

Since Kong runs in DB-less mode, all configuration changes go through Git:

### Example: Add New Backend Service

Edit `gitops/apps/base/kong/config.yaml`:

```yaml
services:
  # ... existing services

  # New service
  - name: heatpump-api
    url: http://heatpump-api.heatpump.svc.cluster.local:8080
    routes:
      - name: heatpump-route
        paths:
          - /api/v1/heatpump
    plugins:
      - name: request-transformer
        config:
          add:
            headers:
              - X-Authenticated-User-Id:$(X-Userinfo-Sub)
              - X-Authenticated-User-Email:$(X-Userinfo-Email)
              - X-Authenticated-Scope:$(X-Userinfo-Scope)
```

Commit and push:

```bash
git add gitops/apps/base/kong/config.yaml
git commit -m "feat: add heatpump-api route to Kong"
git push origin main

# Flux will update the ConfigMap
# Kong will detect the change and reload (may take ~1 minute)

# Force immediate reload (optional)
kubectl rollout restart deployment/kong-gateway -n kong
```

### Example: Update OIDC Scopes

Edit the `openid-connect` plugin config in `config.yaml`:

```yaml
plugins:
  - name: openid-connect
    config:
      # ... existing config
      scopes:
        - openid
        - profile
        - email
        - read:energy
        - read:heatpump
        - read:settings
        - write:settings
        - admin:all  # New scope
```

Commit, push, and Kong will reload automatically.

## Testing

### 1. Get Access Token from Authentik

```bash
# Port-forward Authentik
kubectl port-forward -n authentik svc/authentik-server 9000:9000 &

# Get token (password grant - testing only)
curl -X POST http://localhost:9000/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "username=admin" \
  -d "password=<your-bootstrap-password>" \
  -d "client_id=kong-api-gateway" \
  -d "client_secret=<your-client-secret>" \
  -d "scope=openid profile email read:energy"

# Save the access_token from response
TOKEN="<access-token-from-response>"
```

### 2. Test API Request via Kong

```bash
# Port-forward Kong proxy
kubectl port-forward -n kong svc/kong-proxy 8000:8000 &

# Test authenticated request
curl -v -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/api/v1/energy/latest

# Should receive:
# - HTTP 200
# - JSON response with energy data
# - Backend received X-Authenticated-* headers
```

### 3. Test WebSocket Connection

```bash
# Install wscat if needed
npm install -g wscat

# Connect to WebSocket
wscat -c "ws://localhost:8000/ws/energy" \
  -H "Authorization: Bearer $TOKEN"

# Should connect and receive real-time messages
```

### 4. Verify Headers Forwarded to Backend

```bash
# Check backend logs for authenticated headers
kubectl logs -n redpanda-sink -l app=redpanda-sink | grep "X-Authenticated"

# Should see:
# X-Authenticated-User-Id: <user-uuid>
# X-Authenticated-User-Email: admin@example.com
# X-Authenticated-Scope: openid profile email read:energy
```

### 5. Test Unauthorized Request

```bash
# Without token
curl -v http://localhost:8000/api/v1/energy/latest

# Should receive:
# HTTP 401 Unauthorized

# With invalid token
curl -v -H "Authorization: Bearer invalid-token" \
  http://localhost:8000/api/v1/energy/latest

# Should receive:
# HTTP 401 Unauthorized
```

## Cloudflare Tunnel Integration

Update Cloudflare Tunnel to route all API traffic through Kong:

Edit `gitops/infrastructure/controllers/cloudflare-tunnel/deployment.yaml`:

```yaml
ingress:
  # All API and WebSocket traffic goes through Kong
  - hostname: api.k12n.com
    service: http://kong-proxy.kong.svc.cluster.local:8000

  # Authentik (for login/logout pages)
  - hostname: authentik.k12n.com
    service: http://authentik-server.authentik.svc.cluster.local:9000

  # Frontend (static site)
  - hostname: heatpump.k12n.com
    service: http://heatpump-web.heatpump-web.svc.cluster.local:80

  # ... other services
```

After updating:

```bash
git add gitops/infrastructure/controllers/cloudflare-tunnel/deployment.yaml
git commit -m "feat: route API traffic through Kong gateway"
git push origin main

# Wait for tunnel to update
kubectl rollout status deployment/cloudflared -n cloudflare-tunnel
```

## Monitoring

### Check Kong Status

```bash
# Pod status
kubectl get pods -n kong

# Kong health
kubectl exec -n kong deploy/kong-gateway -- kong health

# View configuration loaded
kubectl exec -n kong deploy/kong-gateway -- kong config db_export /dev/stdout

# View logs
kubectl logs -n kong -l app=kong-gateway -f
```

### Verify Configuration

```bash
# Port-forward Admin API (read-only in DB-less mode)
kubectl port-forward -n kong svc/kong-admin 8001:8001 &

# List services
curl http://localhost:8001/services

# List routes
curl http://localhost:8001/routes

# List plugins
curl http://localhost:8001/plugins

# Check plugin configuration
curl http://localhost:8001/plugins | jq '.data[] | select(.name == "openid-connect")'
```

### Monitor Token Introspection

```bash
# Kong logs show introspection requests
kubectl logs -n kong -l app=kong-gateway | grep introspect

# Check Authentik logs for introspection requests
kubectl logs -n authentik -l app=authentik-server | grep introspect
```

## Troubleshooting

### Kong Pod Not Starting

```bash
# Check pod logs
kubectl logs -n kong -l app=kong-gateway

# Common issues:
# 1. ConfigMap not found
kubectl get configmap -n kong kong-config

# 2. Secret not found
kubectl get secret -n kong kong-oidc-secret

# 3. Invalid YAML in config
kubectl get configmap -n kong kong-config -o yaml
```

### Configuration Syntax Error

```bash
# Kong will refuse to start with invalid config
kubectl logs -n kong -l app=kong-gateway

# Look for errors like:
# "Error: /kong/declarative/kong.yml:XX: <error message>"

# Validate config locally (requires kong CLI)
docker run --rm -v $(pwd)/gitops/apps/base/kong:/kong kong:3.8 \
  kong config parse /kong/config.yaml
```

### Token Validation Fails

```bash
# Test introspection manually
curl -X POST http://authentik-server.authentik.svc.cluster.local:9000/application/o/introspect/ \
  -u "kong-api-gateway:<client-secret>" \
  -d "token=$TOKEN"

# Should return:
# {"active": true, "sub": "<user-id>", ...}

# If "active": false:
# - Token expired
# - Token invalid
# - Wrong client credentials
```

### Headers Not Forwarded

```bash
# Check request-transformer plugin config
kubectl get configmap -n kong kong-config -o yaml | grep -A 10 "request-transformer"

# Ensure header names match:
# Kong OIDC plugin sets: X-Userinfo-Sub, X-Userinfo-Email, X-Userinfo-Scope
# We transform to: X-Authenticated-User-Id, X-Authenticated-User-Email, X-Authenticated-Scope

# Test with httpbin
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/api/v1/debug/headers
```

### Config Changes Not Applied

```bash
# Check if ConfigMap updated
kubectl get configmap -n kong kong-config -o yaml

# Force reload
kubectl rollout restart deployment/kong-gateway -n kong

# Kong in DB-less mode watches the file, but Kubernetes
# ConfigMap updates may be cached up to 1 minute
```

## Configuration Examples

### Add Rate Limiting

```yaml
services:
  - name: redpanda-sink
    # ... existing config
    plugins:
      - name: rate-limiting
        config:
          minute: 60
          policy: local
```

### Add CORS

```yaml
plugins:
  - name: cors
    config:
      origins:
        - https://heatpump.k12n.com
      methods:
        - GET
        - POST
        - PUT
        - DELETE
      headers:
        - Authorization
        - Content-Type
      exposed_headers:
        - X-Auth-Token
      credentials: true
      max_age: 3600
```

### Add IP Restriction

```yaml
services:
  - name: redpanda-sink
    plugins:
      - name: ip-restriction
        config:
          allow:
            - 10.0.0.0/8
            - 192.168.0.0/16
```

### Add Request/Response Logging

```yaml
plugins:
  - name: file-log
    config:
      path: /dev/stdout
      reopen: true
```

## Backend Service Changes

After Kong is deployed, update backend services to remove JWT validation and trust Kong headers:

### redpanda-sink

Remove JWT validation logic and trust headers:

```rust
// src/auth.rs
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: String,
    pub scope: String,
}

pub fn get_user_from_headers(headers: &HeaderMap) -> Result<AuthenticatedUser> {
    let user_id = headers
        .get("X-Authenticated-User-Id")
        .ok_or(Error::Unauthorized)?
        .to_str()?
        .to_string();

    let email = headers
        .get("X-Authenticated-User-Email")
        .ok_or(Error::Unauthorized)?
        .to_str()?
        .to_string();

    let scope = headers
        .get("X-Authenticated-Scope")
        .ok_or(Error::Unauthorized)?
        .to_str()?
        .to_string();

    Ok(AuthenticatedUser { user_id, email, scope })
}

pub fn require_scope(user: &AuthenticatedUser, required: &str) -> Result<()> {
    if user.scope.contains(required) {
        Ok(())
    } else {
        Err(Error::Forbidden(format!("Missing required scope: {}", required)))
    }
}
```

Usage in handlers:

```rust
async fn get_energy_latest(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<EnergyLatest>> {
    let user = get_user_from_headers(&headers)?;
    require_scope(&user, "read:energy")?;

    // ... rest of handler
}
```

### energy-ws

Same pattern - trust headers instead of validating tokens.

## Security Considerations

1. **Trust Kong Headers**: Backend services MUST only be accessible via Kong (use NetworkPolicies)
2. **No Public Access**: Backend services should not be exposed to Cloudflare Tunnel directly
3. **Scope Validation**: Always check X-Authenticated-Scope in backend services
4. **Token Revocation**: Revoke tokens in Authentik, Kong will detect on next introspection
5. **Config in Git**: All Kong config is auditable and reviewed via PRs
6. **Admin API**: Port 8001 is read-only in DB-less mode, but still shouldn't be exposed

## Next Steps

1. ✅ Deploy Kong in DB-less mode
2. ✅ Configure Authentik OAuth2 provider
3. ✅ Test authentication flow
4. Update Cloudflare Tunnel to route through Kong
5. Update backend services to trust Kong headers (remove JWT validation)
6. Update frontend to use OAuth2 authorization code flow
7. Add NetworkPolicies to restrict backend access to Kong only
8. Monitor and tune introspection cache TTL for performance
