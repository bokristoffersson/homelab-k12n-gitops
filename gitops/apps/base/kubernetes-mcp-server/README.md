# Kubernetes MCP Server

A Model Context Protocol (MCP) server that provides AI assistants like Claude with read-only access to your Kubernetes cluster.

## Architecture

```
┌─────────────────────────────────────────────────┐
│           Claude Desktop / MCP Client            │
│  - OAuth2 PKCE flow with Authentik              │
│  - Exchanges auth code for access token         │
└──────────────────┬──────────────────────────────┘
                   │ HTTPS (OAuth2)
                   │
┌──────────────────▼──────────────────────────────┐
│              Traefik Ingress                     │
│  Route: /mcp/kubernetes (strip prefix)          │
│  Middlewares: CORS, security headers            │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│         Kubernetes MCP Server                    │
│  - OIDC authentication via Authentik            │
│  - Token validation                              │
│  - K8s API client (ServiceAccount)              │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│          Kubernetes API Server                   │
│  - Read-only access (viewer RBAC)               │
│  - Pods, Deployments, Services, Logs, etc.      │
└─────────────────────────────────────────────────┘
```

## Features

- **OIDC Authentication**: Uses Authentik for secure OAuth2/OIDC authentication
- **Read-Only Access**: Limited to viewing resources, no write operations
- **Comprehensive RBAC**: ClusterRole with read-only permissions for most resources
- **MCP Path Structure**: Deployed under `/mcp/kubernetes` for future MCP servers

## Access URL

The MCP server is accessible at:

```
https://api.k12n.com/mcp/kubernetes
```

## Deployment Steps

### Step 1: Apply Authentik Blueprint

The blueprint creates an OAuth2 provider and application in Authentik:

```bash
# Apply the blueprint
kubectl apply -f gitops/apps/base/authentik/blueprint-kubernetes-mcp.yaml

# Restart Authentik to load the blueprint
kubectl rollout restart deployment/authentik-server -n authentik

# Wait for restart
kubectl rollout status deployment/authentik-server -n authentik

# Verify blueprint was applied (check logs)
kubectl logs -n authentik -l app=authentik-server --tail=100 | grep kubernetes-mcp
```

### Step 2: Get OAuth Client Secret

1. Open Authentik admin console: `https://authentik.k12n.com`
2. Navigate to **Applications** → **Providers**
3. Find **kubernetes-mcp** provider
4. Copy the **Client Secret** value

### Step 3: Create Sealed Secret

On your Kubernetes control node:

```bash
# Set the client secret from Authentik
CLIENT_SECRET="<paste-client-secret-here>"

# Create sealed secret
kubectl create secret generic kubernetes-mcp-oauth-secret \
  --namespace=kubernetes-mcp-server \
  --from-literal=sts_client_secret="${CLIENT_SECRET}" \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace kubernetes-mcp-server \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/kubernetes-mcp-server/oauth-secret-sealed.yaml
```

### Step 4: Enable Secret in Kustomization

Uncomment the sealed secret in `kustomization.yaml`:

```yaml
resources:
  - namespace.yaml
  - rbac.yaml
  - config-configmap.yaml
  - helmrelease.yaml
  - oauth-secret-sealed.yaml  # Uncomment this line
```

### Step 5: Deploy

```bash
# Commit changes
git add gitops/apps/base/
git commit -m "$(cat <<'EOF'
feat: add Kubernetes MCP server with Authentik OIDC

- Create Authentik blueprint for OAuth2 provider
- Configure MCP server with OIDC authentication
- Add Traefik ingress route at /mcp/kubernetes
- Set up RBAC with read-only cluster access

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"

git push origin main

# Apply with Flux
flux reconcile kustomization apps --with-source
```

### Step 6: Verify Deployment

```bash
# Check pods
kubectl get pods -n kubernetes-mcp-server

# Check logs
kubectl logs -n kubernetes-mcp-server -l app.kubernetes.io/name=kubernetes-mcp-server

# Test endpoint (should require auth)
curl https://api.k12n.com/mcp/kubernetes/health
```

## Configuration

### OIDC Settings

The MCP server is configured via ConfigMap at `config-configmap.yaml`:

- **Authorization URL**: `https://authentik.k12n.com/application/o/kubernetes-mcp/`
- **Client ID**: `kubernetes-mcp-server`
- **OAuth Scopes**: `openid`, `profile`, `email`
- **Token Validation**: Disabled (validated by K8s API server)

### RBAC Permissions

The `kubernetes-mcp-server-viewer` ClusterRole provides read-only access to:

- **Core Resources**: Pods, Services, ConfigMaps, Secrets (metadata only), PVCs, Nodes, Events
- **Apps**: Deployments, StatefulSets, DaemonSets, ReplicaSets
- **Batch**: Jobs, CronJobs
- **Networking**: Ingresses, NetworkPolicies
- **Flux**: HelmReleases, Kustomizations, GitRepositories
- **Logs**: Pod logs via `pods/log` subresource

## Claude Desktop Integration

To use this MCP server with Claude Desktop, add to your MCP configuration:

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

## Troubleshooting

### Server won't start

```bash
# Check pod status
kubectl get pods -n kubernetes-mcp-server

# Check events
kubectl get events -n kubernetes-mcp-server --sort-by='.lastTimestamp'

# Check logs
kubectl logs -n kubernetes-mcp-server -l app.kubernetes.io/name=kubernetes-mcp-server
```

**Common issues:**
- Missing OAuth secret: Verify sealed secret is created and applied
- Config not mounted: Check ConfigMap is listed in HelmRelease values
- RBAC issues: Verify ServiceAccount and ClusterRoleBinding exist

### OIDC authentication failing

```bash
# Test Authentik provider
curl https://authentik.k12n.com/application/o/kubernetes-mcp/.well-known/openid-configuration

# Check MCP server logs for auth errors
kubectl logs -n kubernetes-mcp-server -l app.kubernetes.io/name=kubernetes-mcp-server | grep -i oauth

# Verify client secret is correct
kubectl get secret kubernetes-mcp-oauth-secret -n kubernetes-mcp-server -o yaml
```

### Can't access K8s resources

```bash
# Verify RBAC
kubectl auth can-i list pods --as=system:serviceaccount:kubernetes-mcp-server:kubernetes-mcp-server

# Check ClusterRoleBinding
kubectl get clusterrolebinding kubernetes-mcp-server-viewer -o yaml

# Test API access from pod
kubectl exec -n kubernetes-mcp-server -it deploy/kubernetes-mcp-server -- \
  kubectl get pods -n default
```

### Ingress not working

```bash
# Check IngressRoute
kubectl get ingressroute -n traefik mcp-routes -o yaml

# Check Traefik logs
kubectl logs -n traefik -l app.kubernetes.io/name=traefik | grep mcp

# Test from within cluster
kubectl run curl-test --image=curlimages/curl -it --rm -- \
  curl http://kubernetes-mcp-server.kubernetes-mcp-server.svc.cluster.local:8080/health
```

## Security Considerations

1. **Read-Only**: MCP server has read-only access, cannot modify cluster state
2. **OIDC Authentication**: All requests require valid Authentik OAuth2 token
3. **ServiceAccount**: Uses dedicated ServiceAccount with minimal RBAC permissions
4. **HTTPS**: All traffic encrypted via Traefik with TLS
5. **Token Expiry**: Access tokens expire after 1 hour, refresh tokens after 7 days
6. **No ForwardAuth**: OIDC handled by MCP server, not oauth2-proxy

## Future MCP Servers

This setup establishes the `/mcp/` root path for all MCP servers. Future servers should follow the pattern:

- `/mcp/kubernetes` - This server (cluster operations)
- `/mcp/grafana` - Grafana MCP server (metrics/dashboards)
- `/mcp/authentik` - Authentik MCP server (identity management)
- `/mcp/<service>` - Additional MCP servers

Each MCP server will:
1. Have its own Authentik blueprint
2. Use its own OAuth2 client
3. Share the `/mcp/` path prefix
4. Have dedicated RBAC permissions

## References

- [Kubernetes MCP Server GitHub](https://github.com/containers/kubernetes-mcp-server)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [Authentik OAuth2 Provider](https://docs.goauthentik.io/docs/providers/oauth2/)
- [Traefik IngressRoute](https://doc.traefik.io/traefik/routing/providers/kubernetes-crd/)
