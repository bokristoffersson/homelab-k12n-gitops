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

### Step 2: Create Application

1. Navigate to **Applications** → **Applications**
2. Create new application:
   - **Name**: `Homelab API`
   - **Slug**: `homelab-api`
   - **Provider**: Select the provider created above
   - **Launch URL**: `https://api.k12n.com`

### Step 3: Configure Default Scopes

Set default scopes for all users:

1. Navigate to **Flows & Stages** → **Flows**
2. Edit the `default-authentication-flow`
3. Add **User Write Stage** with default groups/scopes:
   - Default scopes: `openid email profile read:energy read:heatpump read:settings write:settings`

### Step 4: Create Users

1. Navigate to **Directory** → **Users**
2. Create admin user:
   - **Username**: `admin`
   - **Email**: `admin@k12n.com`
   - **Groups**: `Administrators`
3. Create regular users as needed

### Step 5: Test OIDC Flow

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

```

## Integration with Traefik

Authentik integrates with Traefik through:

1. **Direct JWT validation**: Backend services validate JWT tokens using Authentik's JWKS endpoint
2. **oauth2-proxy (optional)**: ForwardAuth middleware delegates authentication to oauth2-proxy
3. **OIDC Flow**: Frontend SPAs use Authorization Code Flow with PKCE

**Current Architecture**:
- Frontend apps (heatpump-web) authenticate via OIDC and obtain JWT tokens
- JWT tokens are included in API requests as `Authorization: Bearer <token>`
- Backend APIs validate JWT signatures using Authentik's public key
- No session state - fully stateless authentication

For detailed authentication architecture, see `docs/AUTHENTICATION.md`

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

### Automated S3 Backups

A CronJob runs daily at **3 AM** to backup the Authentik PostgreSQL database to S3:

```yaml
Schedule: "0 3 * * *"
Retention: 3 successful jobs, 3 failed jobs
Backup format: pg_dump + gzip
S3 path: s3://<bucket>/<prefix>/authentik-YYYYMMDD-HHMMSS.sql.gz
```

**Setup S3 backup** (if not already configured):

```bash
# Create sealed secret for AWS credentials
kubectl create secret generic authentik-backup-aws \
  --namespace=authentik \
  --from-literal=AWS_ACCESS_KEY_ID="your-key" \
  --from-literal=AWS_SECRET_ACCESS_KEY="your-secret" \
  --from-literal=AWS_REGION=eu-north-1 \
  --from-literal=S3_BUCKET="your-bucket" \
  --from-literal=S3_PREFIX=authentik-backups \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace authentik \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/authentik/backup-aws-credentials-sealed.yaml

# Commit and push the sealed secret
git add gitops/apps/base/authentik/backup-aws-credentials-sealed.yaml
git commit -m "Add Authentik backup AWS credentials"
git push
```

**Manually trigger a backup** (for testing):

```bash
# Create one-time job from CronJob
kubectl create job authentik-backup-manual \
  --from=cronjob/authentik-postgres-backup \
  -n authentik

# Watch the backup progress
kubectl logs -n authentik -l job-name=authentik-backup-manual --follow

# Verify in S3
aws s3 ls s3://your-bucket/authentik-backups/ --region eu-north-1

# Clean up test job
kubectl delete job authentik-backup-manual -n authentik
```

### Manual Backup

For ad-hoc backups outside the automated schedule:

```bash
# Export database to local file
kubectl exec -n authentik deploy/authentik-postgres -- \
  pg_dump -U authentik authentik > authentik-backup-$(date +%Y%m%d).sql

# Or create compressed backup
kubectl exec -n authentik deploy/authentik-postgres -- \
  pg_dump -U authentik authentik | gzip > authentik-backup-$(date +%Y%m%d).sql.gz
```

### Restore from S3 Backup

```bash
# 1. Download backup from S3
aws s3 cp s3://your-bucket/authentik-backups/authentik-20250130-030000.sql.gz . \
  --region eu-north-1

# 2. Decompress
gunzip authentik-20250130-030000.sql.gz

# 3. Stop Authentik server and worker to prevent writes
kubectl scale deployment authentik-server -n authentik --replicas=0
kubectl scale deployment authentik-worker -n authentik --replicas=0

# 4. Restore database
kubectl exec -i -n authentik deploy/authentik-postgres -- \
  psql -U authentik authentik < authentik-20250130-030000.sql

# 5. Restart Authentik
kubectl scale deployment authentik-server -n authentik --replicas=1
kubectl scale deployment authentik-worker -n authentik --replicas=1

# 6. Verify
kubectl logs -n authentik -l app=authentik-server --tail=50
```

### Restore from Manual Backup

```bash
# Stop Authentik services
kubectl scale deployment authentik-server -n authentik --replicas=0
kubectl scale deployment authentik-worker -n authentik --replicas=0

# Restore from local backup
kubectl exec -i -n authentik deploy/authentik-postgres -- \
  psql -U authentik authentik < authentik-backup.sql

# Restart services
kubectl scale deployment authentik-server -n authentik --replicas=1
kubectl scale deployment authentik-worker -n authentik --replicas=1
```

### Disaster Recovery

For complete disaster recovery:

1. **Restore sealed secrets**: Ensure sealed-secrets controller has the same sealing key
2. **Restore PostgreSQL data**: Use S3 backup or PVC snapshot
3. **Redeploy Authentik**: Apply kustomization to recreate all resources
4. **Verify health**: Check server and worker logs, test login

## Security Considerations

1. **Secrets**: All sensitive data stored in sealed secrets (encrypted at rest in Git)
2. **Network**: PostgreSQL and Redis only accessible within cluster
3. **HTTPS**: Uses HTTPS in production via Cloudflare Tunnel + Traefik
4. **Passwords**: Strong bootstrap password (32+ characters) generated with openssl
5. **Token Expiry**: 24h access tokens, 30d refresh tokens
6. **JWT Validation**: Backend services validate JWT signatures using Authentik's JWKS
7. **Backup Security**: S3 backups encrypted in transit, AWS credentials in sealed secrets
8. **Blueprint Automation**: OAuth2 applications managed as code via blueprints

## Next Steps

After Authentik is deployed and configured:

1. **Configure OAuth2 Applications** - Use blueprints for declarative app configuration
2. **Update Frontend Apps** - Implement OIDC Authorization Code Flow
3. **Configure Traefik** - Set up CORS and security headers middleware
4. **Test Authentication** - Verify OIDC flow and JWT validation
5. **Enable Backups** - Create sealed secret for S3 backup credentials
6. **Monitor Logs** - Check server and worker logs for issues

## References

- [Authentik Documentation](https://docs.goauthentik.io/)
- [OAuth2/OIDC Spec](https://oauth.net/2/)
- [Token Introspection RFC](https://tools.ietf.org/html/rfc7662)
