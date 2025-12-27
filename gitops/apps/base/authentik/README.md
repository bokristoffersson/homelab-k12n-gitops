# Authentik Identity Provider

Authentik is a self-hosted identity provider (IdP) that provides authentication and authorization for all homelab services.

## Architecture

```
┌─────────────────────────────────────────────────┐
│              Authentik Server                    │
│  - Web UI (login, user management)              │
│  - OIDC/OAuth2 Provider                         │
│  - Token Introspection Endpoint                 │
│  - Issues opaque tokens                         │
└──────────────┬──────────────────────────────────┘
               │
               ├─→ PostgreSQL (user data, sessions)
               └─→ Redis (cache, session store)

┌─────────────────────────────────────────────────┐
│              Authentik Worker                    │
│  - Background tasks                              │
│  - Email notifications                           │
│  - Scheduled jobs                                │
└─────────────────────────────────────────────────┘
```

## Components

### PostgreSQL
- **Image**: `postgres:16-alpine`
- **Storage**: 5Gi PersistentVolumeClaim
- **Resources**: 100m CPU / 256Mi Memory (request), 1000m CPU / 512Mi Memory (limit)
- **Database**: `authentik`

### Redis
- **Image**: `redis:7-alpine`
- **Storage**: In-memory (ephemeral)
- **Resources**: 50m CPU / 64Mi Memory (request), 500m CPU / 256Mi Memory (limit)
- **Persistence**: Snapshots every 60 seconds

### Authentik Server
- **Image**: `ghcr.io/goauthentik/server:2024.12.1`
- **Ports**: 9000 (HTTP), 9443 (HTTPS)
- **Resources**: 100m CPU / 256Mi Memory (request), 1000m CPU / 512Mi Memory (limit)
- **Endpoints**:
  - `/-/health/live/` - Liveness probe
  - `/-/health/ready/` - Readiness probe
  - `/application/o/authorize/` - OAuth2 authorization
  - `/application/o/token/` - Token endpoint
  - `/application/o/introspect/` - Token introspection

### Authentik Worker
- **Image**: `ghcr.io/goauthentik/server:2024.12.1`
- **Resources**: 50m CPU / 256Mi Memory (request), 500m CPU / 512Mi Memory (limit)
- **Function**: Handles background tasks and scheduled jobs

## Deployment

### Step 1: Create Sealed Secrets

SSH to your Kubernetes control node:

```bash
# Generate strong secrets
POSTGRES_PASSWORD=$(openssl rand -base64 32)
AUTHENTIK_SECRET_KEY=$(openssl rand -base64 60)
AUTHENTIK_BOOTSTRAP_PASSWORD=$(openssl rand -base64 32)
AUTHENTIK_BOOTSTRAP_TOKEN=$(openssl rand -base64 32)

# Save these credentials securely!
echo "PostgreSQL Password: ${POSTGRES_PASSWORD}"
echo "Bootstrap Password: ${AUTHENTIK_BOOTSTRAP_PASSWORD}"
echo "Bootstrap Token: ${AUTHENTIK_BOOTSTRAP_TOKEN}"

# Create PostgreSQL secret
kubectl create secret generic authentik-postgres-secret \
  --namespace=authentik \
  --from-literal=POSTGRES_USER='authentik' \
  --from-literal=POSTGRES_PASSWORD="${POSTGRES_PASSWORD}" \
  --from-literal=POSTGRES_DB='authentik' \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace authentik \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/authentik/postgres-secret-sealed.yaml

# Create Authentik secret
kubectl create secret generic authentik-secret \
  --namespace=authentik \
  --from-literal=AUTHENTIK_SECRET_KEY="${AUTHENTIK_SECRET_KEY}" \
  --from-literal=AUTHENTIK_BOOTSTRAP_PASSWORD="${AUTHENTIK_BOOTSTRAP_PASSWORD}" \
  --from-literal=AUTHENTIK_BOOTSTRAP_TOKEN="${AUTHENTIK_BOOTSTRAP_TOKEN}" \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace authentik \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/authentik/authentik-secret-sealed.yaml
```

### Step 2: Update Kustomization

Uncomment the sealed secret resources in `kustomization.yaml`:

```yaml
resources:
  # ...
  - postgres-secret-sealed.yaml
  - authentik-secret-sealed.yaml
```

### Step 3: Deploy

```bash
# Commit sealed secrets
git add gitops/apps/base/authentik/
git commit -m "Add Authentik sealed secrets"
git push origin main

# Deploy
kubectl apply -k gitops/apps/base/authentik/

# OR use Flux
flux reconcile kustomization authentik --with-source
```

### Step 4: Wait for Pods

```bash
# Watch deployment
kubectl get pods -n authentik -w

# Expected pods:
# - authentik-postgres-xxx
# - authentik-redis-xxx
# - authentik-server-xxx
# - authentik-worker-xxx
```

### Step 5: Access Initial Setup

```bash
# Port-forward to access UI locally
kubectl port-forward -n authentik svc/authentik-server 9000:9000

# Open browser to: http://localhost:9000/if/flow/initial-setup/

# Initial login:
#   Username: akadmin
#   Password: <AUTHENTIK_BOOTSTRAP_PASSWORD from Step 1>
```

## Configuration

### Step 1: Create Provider Application

1. Navigate to **Applications** → **Providers**
2. Create new **OAuth2/OpenID Provider**:
   - **Name**: `Homelab Services`
   - **Client Type**: `Confidential`
   - **Redirect URIs**:
     ```
     https://api.k12n.com/auth/callback
     http://localhost:8080/auth/callback  # For local testing
     ```
   - **Scopes**:
     - `openid` (required)
     - `email` (required)
     - `profile` (required)
     - Custom scopes:
       - `read:energy`
       - `read:heatpump`
       - `read:settings`
       - `write:settings`
   - **Token validity**:
     - Access token: `24 hours`
     - Refresh token: `30 days`
   - **Token type**: `Opaque` (important!)
   - **Subject mode**: `Based on User's ID`

3. Save and note the **Client ID** and **Client Secret**

### Step 2: Configure Token Introspection

The introspection endpoint will be used by Kong:

```
Endpoint: http://authentik-server.authentik.svc.cluster.local:9000/application/o/introspect/
Method: POST
Authentication: Basic Auth (Client ID:Client Secret)
```

### Step 3: Create Application

1. Navigate to **Applications** → **Applications**
2. Create new application:
   - **Name**: `Homelab API`
   - **Slug**: `homelab-api`
   - **Provider**: Select the provider created above
   - **Launch URL**: `https://api.k12n.com`

### Step 4: Configure Default Scopes

Set default scopes for all users:

1. Navigate to **Flows & Stages** → **Flows**
2. Edit the `default-authentication-flow`
3. Add **User Write Stage** with default groups/scopes:
   - Default scopes: `openid email profile read:energy read:heatpump read:settings write:settings`

### Step 5: Create Users

1. Navigate to **Directory** → **Users**
2. Create admin user:
   - **Username**: `admin`
   - **Email**: `admin@k12n.com`
   - **Groups**: `Administrators`
3. Create regular users as needed

### Step 6: Test OIDC Flow

Test the authentication flow:

```bash
# Get authorization URL (replace CLIENT_ID)
https://api.k12n.com/auth/application/o/authorize/?client_id=<CLIENT_ID>&redirect_uri=https://api.k12n.com/auth/callback&response_type=code&scope=openid%20email%20profile%20read:energy

# After login, exchange code for token:
curl -X POST https://api.k12n.com/auth/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=<AUTHORIZATION_CODE>" \
  -d "redirect_uri=https://api.k12n.com/auth/callback" \
  -d "client_id=<CLIENT_ID>" \
  -d "client_secret=<CLIENT_SECRET>"

# Response (opaque token):
{
  "access_token": "random-opaque-string-here",
  "token_type": "Bearer",
  "expires_in": 86400,
  "refresh_token": "another-random-string",
  "scope": "openid email profile read:energy read:heatpump read:settings write:settings"
}

# Test introspection (what Kong will do):
curl -X POST http://localhost:9000/application/o/introspect/ \
  -u "<CLIENT_ID>:<CLIENT_SECRET>" \
  -d "token=<ACCESS_TOKEN>"

# Response:
{
  "active": true,
  "sub": "user-uuid-here",
  "email": "admin@k12n.com",
  "scope": "openid email profile read:energy read:heatpump read:settings write:settings",
  "client_id": "<CLIENT_ID>",
  "token_type": "Bearer",
  "exp": 1735401234,
  "iat": 1735314834
}
```

## Integration with Kong

Kong will be configured to:

1. Intercept requests to `/api/*` and `/ws/*`
2. Extract opaque token from `Authorization: Bearer <token>` header or cookie
3. Validate token via introspection endpoint
4. Forward authenticated requests with headers:
   - `X-Authenticated-User-Id: <sub from introspection>`
   - `X-Authenticated-User-Email: <email from introspection>`
   - `X-Authenticated-Scope: <scope from introspection>`

## Monitoring

```bash
# View server logs
kubectl logs -n authentik -l app=authentik-server -f

# View worker logs
kubectl logs -n authentik -l app=authentik-worker -f

# Check database connection
kubectl exec -n authentik -it deploy/authentik-postgres -- psql -U authentik -d authentik -c "SELECT COUNT(*) FROM auth_user;"

# Check Redis
kubectl exec -n authentik -it deploy/authentik-redis -- redis-cli ping
```

## Troubleshooting

### Server pod won't start

```bash
# Check logs
kubectl logs -n authentik -l app=authentik-server

# Common issues:
# 1. PostgreSQL not ready - wait for postgres pod
# 2. Missing secrets - verify sealed secrets are applied
# 3. Database migration failed - check postgres logs
```

### Can't access UI

```bash
# Verify service
kubectl get svc -n authentik authentik-server

# Port-forward
kubectl port-forward -n authentik svc/authentik-server 9000:9000

# Check if server is responding
curl http://localhost:9000/-/health/live/
```

### Token introspection failing

```bash
# Test introspection endpoint directly
kubectl run curl-test --image=curlimages/curl -it --rm -- sh

# Inside pod:
curl -X POST http://authentik-server.authentik.svc.cluster.local:9000/application/o/introspect/ \
  -u "CLIENT_ID:CLIENT_SECRET" \
  -d "token=YOUR_TOKEN"
```

### Database migration errors

```bash
# Run migrations manually
kubectl exec -n authentik -it deploy/authentik-server -- ak migrate

# Check migration status
kubectl exec -n authentik -it deploy/authentik-server -- ak showmigrations
```

## Backup and Recovery

### Backup PostgreSQL

```bash
# Export database
kubectl exec -n authentik deploy/authentik-postgres -- pg_dump -U authentik authentik > authentik-backup.sql

# Or use PVC snapshots
kubectl get pvc -n authentik
```

### Restore PostgreSQL

```bash
# Restore from backup
kubectl exec -i -n authentik deploy/authentik-postgres -- psql -U authentik authentik < authentik-backup.sql
```

## Security Considerations

1. **Secrets**: All sensitive data in sealed secrets
2. **Network**: PostgreSQL and Redis only accessible within cluster
3. **HTTPS**: Should use HTTPS in production (via Kong/Cloudflare Tunnel)
4. **Passwords**: Use strong bootstrap password (32+ characters)
5. **Token Expiry**: Configure appropriate expiry times (24h access, 30d refresh)
6. **Introspection**: Only Kong should access introspection endpoint (enforce via NetworkPolicy)

## Next Steps

After Authentik is deployed and configured:

1. **Deploy Kong** - API Gateway with OIDC plugin
2. **Configure Kong** - Point to Authentik introspection endpoint
3. **Update Services** - Remove JWT validation from redpanda-sink and energy-ws
4. **Update Frontend** - Implement OIDC authentication flow
5. **Update Cloudflare Tunnel** - Route through Kong

## References

- [Authentik Documentation](https://docs.goauthentik.io/)
- [OAuth2/OIDC Spec](https://oauth.net/2/)
- [Token Introspection RFC](https://tools.ietf.org/html/rfc7662)
