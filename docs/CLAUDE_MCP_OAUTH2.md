# Claude MCP OAuth2 Client Credentials Setup

This document explains how to set up OAuth2 client credentials flow for Claude Desktop to access the homelab-api MCP server.

## Architecture

```
Claude Desktop
    ↓ (1) POST /application/o/token/ (client credentials grant)
Authentik
    ↓ (2) Returns access_token (JWT)
Claude Desktop
    ↓ (3) GET/POST https://api.k12n.com/mcp (Authorization: Bearer <token>)
Traefik
    ↓ (4) Forward request (currently public - add JWT auth later if needed)
homelab-api
    ↓ (5) Process MCP request
TimescaleDB
```

## Components

1. **Authentik OAuth2 Provider**: Configured with client credentials grant type
2. **homelab-api**: Ready for JWT validation (can be added later)
3. **Traefik**: Routes MCP traffic (currently public endpoint)
4. **Claude Desktop**: OAuth2 client using client credentials flow

**Note**: This setup creates the OAuth2 provider and gives you the credentials. JWT token validation in homelab-api can be added later if you want to enforce authentication.

## Deployment Steps

### 1. Deploy Authentik Blueprint

The blueprint creates:
- Service account user `claude-mcp-service`
- OAuth2 provider `claude-mcp` with client credentials support
- Application `Claude MCP`

```bash
# Changes already staged - commit and push
git add -A
git commit -m "$(cat <<'EOF'
feat(authentik): add Claude MCP OAuth2 client credentials provider

- Create service account for Claude MCP client
- Configure OAuth2 provider with client credentials grant
- Remove ForwardAuth from MCP endpoint (use Bearer tokens instead)
- Add JWT validation to MCP handlers using existing JWKS support
- Update homelab-api config to use claude-mcp JWKS endpoint

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"

git push
```

### 2. Wait for FluxCD to Apply Changes

```python
import sys
sys.path.insert(0, '/Users/bo/Development/homelab/Cursor Workspace/homelab-k12n-gitops/rag-k8s')
from agent.tool import k8s_exec

# Reconcile Authentik
k8s_exec({
  "intent": "flux-reconcile",
  "resource": "kustomization",
  "namespace": "flux-system",
  "name": "authentik",
  "constraints": {"dryRun": False}
})

# Reconcile homelab-api
k8s_exec({
  "intent": "flux-reconcile",
  "resource": "kustomization",
  "namespace": "flux-system",
  "name": "homelab-api",
  "constraints": {"dryRun": False}
})

# Reconcile traefik-routes
k8s_exec({
  "intent": "flux-reconcile",
  "resource": "kustomization",
  "namespace": "flux-system",
  "name": "traefik-routes",
  "constraints": {"dryRun": False}
})
```

### 3. Retrieve Client Credentials from Authentik

**Option A: Via Authentik UI** (Recommended)

1. Navigate to https://authentik.k12n.com/if/admin/
2. Go to Applications → Applications
3. Find "Claude MCP" application
4. Click on the provider (claude-mcp)
5. Copy the `Client ID` (should be `claude-mcp-client`)
6. Click "Generate Secret" or view existing secret
7. Copy the `Client Secret` (save this securely!)

**Option B: Via kubectl**

```bash
# Get Authentik server pod
kubectl get pods -n authentik -l app=authentik-server

# Execute into the pod (replace POD_NAME)
kubectl exec -it -n authentik <POD_NAME> -- /bin/bash

# Use authentik CLI to get provider details
python manage.py shell

# In Python shell:
from authentik.providers.oauth2.models import OAuth2Provider
provider = OAuth2Provider.objects.get(name='claude-mcp')
print(f"Client ID: {provider.client_id}")
print(f"Client Secret: {provider.client_secret}")
exit()
```

### 4. Test Token Endpoint

Once deployed, test the OAuth2 token endpoint:

```bash
# Replace with your actual client_id and client_secret
CLIENT_ID="claude-mcp-client"
CLIENT_SECRET="your-secret-here"

curl -X POST https://authentik.k12n.com/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=${CLIENT_ID}" \
  -d "client_secret=${CLIENT_SECRET}"

# Expected response:
# {
#   "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6IjQ4ZDhkMjE5In0...",
#   "token_type": "Bearer",
#   "expires_in": 86400
# }
```

### 5. Test MCP Endpoint with Bearer Token

```bash
# Use the access_token from step 4
ACCESS_TOKEN="eyJ0eXAiOiJKV1Qi..."

# Test MCP endpoint
curl -X POST https://api.k12n.com/mcp \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize"
  }'

# Expected response:
# {
#   "jsonrpc": "2.0",
#   "id": 1,
#   "result": {
#     "protocolVersion": "2024-11-05",
#     "capabilities": {"tools": {}},
#     "serverInfo": {"name": "homelab-api", "version": "..."}
#   }
# }
```

## Claude Desktop Configuration

### MCP Settings JSON

Add this to your Claude Desktop MCP configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "homelab": {
      "url": "https://api.k12n.com/mcp",
      "transport": {
        "type": "sse"
      },
      "authorization": {
        "type": "oauth2",
        "tokenUrl": "https://authentik.k12n.com/application/o/token/",
        "clientId": "claude-mcp-client",
        "clientSecret": "YOUR_CLIENT_SECRET_HERE",
        "grantType": "client_credentials"
      }
    }
  }
}
```

### Restart Claude Desktop

After updating the configuration, restart Claude Desktop to load the new MCP server.

## Available MCP Tools

Once configured, Claude will have access to:

1. **energy_hourly_consumption** - Get hourly energy consumption data
2. **energy_peak_hour_day** - Find peak energy usage hour for a specific day
3. **heatpump_daily_summary** - Get daily heatpump operation summaries

## Troubleshooting

### "Missing Authorization header"

The client is not sending the Bearer token. Check:
- Claude Desktop configuration is correct
- clientId and clientSecret are valid
- tokenUrl is accessible from Claude Desktop

### "Invalid token"

The JWT validation failed. Check:
- JWKS URL is accessible from homelab-api pod
- Issuer matches between Authentik and homelab-api config
- Token hasn't expired (default: 24 hours)

### "JWT validator not configured"

The homelab-api couldn't initialize the JWT validator. Check:
- ConfigMap has correct jwks_url and issuer
- Authentik is accessible from the cluster
- homelab-api pod logs for initialization errors

```python
from agent.tool import k8s_exec
k8s_exec({
  "intent": "logs",
  "resource": "deployment",
  "namespace": "homelab-api",
  "name": "homelab-api",
  "constraints": {"dryRun": False}
})
```

### Check Authentik Logs

```python
from agent.tool import k8s_exec
k8s_exec({
  "intent": "logs",
  "resource": "deployment",
  "namespace": "authentik",
  "name": "authentik-server",
  "constraints": {"dryRun": False}
})
```

## Security Considerations

1. **Client Secret Storage**: Store the client secret securely. Never commit it to version control.

2. **Token Expiration**: Access tokens expire after 24 hours. Claude Desktop should automatically refresh.

3. **JWKS Caching**: The JWT validator caches JWKS keys. If you rotate Authentik keys, restart homelab-api.

4. **Network Security**: All traffic uses HTTPS with TLS termination at Cloudflare Tunnel.

5. **Service Account**: The claude-mcp-service account has no interactive login capabilities.

## Updating Configuration

### Update Token Validity

Edit the Authentik blueprint and change:

```yaml
access_token_validity: hours=48  # Default: hours=24
```

Then reconcile:

```python
from agent.tool import k8s_exec
k8s_exec({
  "intent": "flux-reconcile",
  "resource": "kustomization",
  "namespace": "flux-system",
  "name": "authentik",
  "constraints": {"dryRun": False}
})
```

### Regenerate Client Secret

1. Log into Authentik admin UI
2. Navigate to the claude-mcp provider
3. Click "Generate Secret"
4. Update Claude Desktop configuration with new secret
5. Restart Claude Desktop

## References

- [Authentik OAuth2 Provider Docs](https://goauthentik.io/docs/providers/oauth2/)
- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [OAuth2 Client Credentials Grant](https://oauth.net/2/grant-types/client-credentials/)
