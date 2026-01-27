# Get Kubernetes MCP Client Secret

## Quick Steps

1. **Open Authentik Admin**:
   ```
   https://authentik.k12n.com/if/admin/
   ```

2. **Navigate to Provider**:
   - Applications → Applications
   - Find "Kubernetes MCP Server"
   - Click on it
   - Click on the provider name "kubernetes-mcp"

3. **Copy Client Secret**:
   - Look for "Client Secret" field
   - Click the copy icon
   - Save this secret (you'll need it in the next step)

## Alternative: Use API

```bash
# Get Authentik API token
AUTHENTIK_TOKEN=$(kubectl exec -n authentik deploy/authentik-server -- ak get_token)

# Get provider details
curl -s -H "Authorization: Bearer $AUTHENTIK_TOKEN" \
  https://authentik.k12n.com/api/v3/providers/oauth2/ \
  | jq '.results[] | select(.client_id == "kubernetes-mcp-server") | .client_secret'
```
