# Kubernetes MCP Server Deployment Guide

## Overview

This guide walks through deploying the Kubernetes MCP server with Authentik OIDC authentication and Traefik ingress.

## What Was Created

### 1. Authentik Blueprint (`blueprint-kubernetes-mcp.yaml`)

Creates an OAuth2/OIDC provider in Authentik with:
- **Client ID**: `kubernetes-mcp-server`
- **Client Type**: Confidential (with secret)
- **Redirect URI**: `https://api.k12n.com/mcp/kubernetes/callback`
- **Token Validity**: 1 hour access, 7 days refresh
- **Scopes**: openid, profile, email, offline_access

### 2. MCP Server Configuration (`config-configmap.yaml`)

ConfigMap with OIDC settings:
- Authorization URL pointing to Authentik
- OAuth scopes and audience
- Token validation settings
- Server port configuration (8080)

### 3. Traefik IngressRoute (`mcp-routes.yaml`)

Creates routing for MCP servers:
- **Path**: `/mcp/kubernetes` (with prefix stripping)
- **Host**: `api.k12n.com`
- **Middlewares**: HTTPS redirect, CORS, security headers
- **Target**: kubernetes-mcp-server service on port 8080

### 4. Updated HelmRelease

Enhanced with:
- ConfigMap and Secret mounts
- Environment variables for config paths
- ServiceAccount configuration

### 5. RBAC Configuration (existing)

ClusterRole with read-only access to:
- Pods, Services, ConfigMaps, Nodes, Events
- Deployments, StatefulSets, Jobs, CronJobs
- Flux resources (HelmReleases, Kustomizations)
- Pod logs

## Deployment Steps

### Prerequisites

- Authentik deployed and accessible at `https://authentik.k12n.com`
- Traefik deployed as ingress controller
- Sealed Secrets operator running
- Flux CD managing GitOps deployments

### Step 1: Apply Authentik Blueprint

```bash
# Apply the blueprint ConfigMap
kubectl apply -f gitops/apps/base/authentik/blueprint-kubernetes-mcp.yaml

# Restart Authentik server to load blueprint
kubectl rollout restart deployment/authentik-server -n authentik

# Wait for restart to complete
kubectl rollout status deployment/authentik-server -n authentik

# Watch logs to verify blueprint application
kubectl logs -n authentik -l app=authentik-server -f | grep -A 10 "kubernetes-mcp"
```

Expected log output:
```
Applied blueprint kubernetes-mcp
Created provider: kubernetes-mcp
Created application: Kubernetes MCP Server
```

### Step 2: Retrieve Client Secret from Authentik

1. **Open Authentik Admin Console**:
   ```bash
   # If not publicly accessible, port-forward:
   kubectl port-forward -n authentik svc/authentik-server 9000:9000
   # Then open: http://localhost:9000
   ```

2. **Navigate to Providers**:
   - Go to **Applications** → **Providers**
   - Find **kubernetes-mcp** in the list
   - Click on it to view details

3. **Copy Client Secret**:
   - Look for **Client Secret** field
   - Click the copy icon or select and copy the value
   - Save this securely (you'll need it in the next step)

### Step 3: Create Sealed Secret

**On your Kubernetes control node** (or wherever you have kubeseal access):

```bash
# Set the client secret from Authentik
CLIENT_SECRET="<paste-your-client-secret-here>"

# Create the sealed secret
kubectl create secret generic kubernetes-mcp-oauth-secret \
  --namespace=kubernetes-mcp-server \
  --from-literal=sts_client_secret="${CLIENT_SECRET}" \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace kubernetes-mcp-server \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/kubernetes-mcp-server/oauth-secret-sealed.yaml

# Verify the file was created
cat gitops/apps/base/kubernetes-mcp-server/oauth-secret-sealed.yaml
```

The sealed secret should look like:
```yaml
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: kubernetes-mcp-oauth-secret
  namespace: kubernetes-mcp-server
spec:
  encryptedData:
    sts_client_secret: AgB...encrypted...data...
  template:
    metadata:
      name: kubernetes-mcp-oauth-secret
      namespace: kubernetes-mcp-server
```

### Step 4: Enable Sealed Secret in Kustomization

Edit `gitops/apps/base/kubernetes-mcp-server/kustomization.yaml`:

```yaml
resources:
  - namespace.yaml
  - rbac.yaml
  - config-configmap.yaml
  - helmrelease.yaml
  - oauth-secret-sealed.yaml  # Uncomment this line
```

### Step 5: Commit and Push

```bash
# Stage all changes
git add gitops/apps/base/authentik/blueprint-kubernetes-mcp.yaml
git add gitops/apps/base/authentik/kustomization.yaml
git add gitops/apps/base/kubernetes-mcp-server/
git add gitops/apps/base/traefik-routes/mcp-routes.yaml
git add gitops/apps/base/traefik-routes/kustomization.yaml

# Commit with proper format
git commit -m "$(cat <<'EOF'
feat: add Kubernetes MCP server with Authentik OIDC

Add Model Context Protocol server for Kubernetes with OIDC authentication:
- Authentik blueprint for OAuth2 provider configuration
- MCP server config with OIDC settings for Authentik
- Traefik ingress at /mcp/kubernetes with prefix stripping
- Read-only RBAC permissions for cluster resources

The /mcp/* path structure allows future MCP servers to be added easily.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"

# Push to trigger Flux reconciliation
git push origin main
```

### Step 6: Deploy with Flux

```bash
# Reconcile the apps kustomization
flux reconcile kustomization apps --with-source

# Watch the deployment
kubectl get pods -n kubernetes-mcp-server -w
```

Expected pods:
```
NAME                                     READY   STATUS    RESTARTS   AGE
kubernetes-mcp-server-xxxxxxxxxx-xxxxx   1/1     Running   0          30s
```

### Step 7: Verify Deployment

#### Check Pod Status

```bash
# Get pod status
kubectl get pods -n kubernetes-mcp-server

# Check pod logs
kubectl logs -n kubernetes-mcp-server -l app.kubernetes.io/name=kubernetes-mcp-server

# Check events
kubectl get events -n kubernetes-mcp-server --sort-by='.lastTimestamp' | tail -20
```

#### Verify Configuration Mounted

```bash
# Check if config is mounted
kubectl exec -n kubernetes-mcp-server deploy/kubernetes-mcp-server -- \
  cat /config/config.toml

# Check if secret is mounted
kubectl exec -n kubernetes-mcp-server deploy/kubernetes-mcp-server -- \
  ls -la /secrets/
```

#### Test OIDC Endpoint

```bash
# Test Authentik OIDC well-known configuration
curl https://authentik.k12n.com/application/o/kubernetes-mcp/.well-known/openid-configuration
```

Should return JSON with endpoints:
```json
{
  "issuer": "https://authentik.k12n.com/application/o/kubernetes-mcp/",
  "authorization_endpoint": "https://authentik.k12n.com/application/o/authorize/",
  "token_endpoint": "https://authentik.k12n.com/application/o/token/",
  ...
}
```

#### Test Ingress Route

```bash
# Test from external
curl -I https://api.k12n.com/mcp/kubernetes/health

# Expected: 401 Unauthorized (auth required) or 200 OK (health endpoint public)
```

#### Test RBAC Permissions

```bash
# Test as ServiceAccount
kubectl auth can-i list pods \
  --as=system:serviceaccount:kubernetes-mcp-server:kubernetes-mcp-server

# Expected: yes

kubectl auth can-i delete pods \
  --as=system:serviceaccount:kubernetes-mcp-server:kubernetes-mcp-server

# Expected: no (read-only)
```

## Testing with Claude Desktop

### 1. Update Claude Desktop MCP Config

Location:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

Add the MCP server:

```json
{
  "mcpServers": {
    "kubernetes": {
      "url": "https://api.k12n.com/mcp/kubernetes",
      "oauth": {
        "client_id": "kubernetes-mcp-server",
        "authorization_url": "https://authentik.k12n.com/application/o/kubernetes-mcp/authorize/",
        "token_url": "https://authentik.k12n.com/application/o/kubernetes-mcp/token/",
        "scopes": ["openid", "profile", "email"]
      }
    }
  }
}
```

### 2. Restart Claude Desktop

After saving the config, restart Claude Desktop to load the new MCP server.

### 3. Authenticate

On first use, Claude will prompt you to authenticate:
1. Browser opens to Authentik login
2. Enter your Authentik credentials
3. Approve the OAuth consent screen
4. Browser redirects back to Claude
5. Claude now has access to the MCP server

### 4. Test Commands

Try asking Claude:

```
"List all pods in the default namespace"
"Show me the deployments in the kubernetes-mcp-server namespace"
"What's the status of the authentik pods?"
"Get logs from the kubernetes-mcp-server pod"
```

## Rollback

If something goes wrong, you can rollback:

```bash
# Delete the sealed secret
kubectl delete secret kubernetes-mcp-oauth-secret -n kubernetes-mcp-server

# Rollback git commits
git revert HEAD
git push origin main

# Reconcile Flux
flux reconcile kustomization apps --with-source

# Or manually delete resources
kubectl delete -k gitops/apps/base/kubernetes-mcp-server/
kubectl delete ingressroute mcp-routes -n traefik
```

## Next Steps

1. **Add more MCP servers** following the `/mcp/<service>` pattern
2. **Configure additional Authentik users** for team access
3. **Set up monitoring** for MCP server health
4. **Create dashboards** in Grafana for MCP usage metrics
5. **Document common Claude queries** for your team

## Troubleshooting

See [README.md](./README.md#troubleshooting) for common issues and solutions.
