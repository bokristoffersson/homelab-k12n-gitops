# Setup Notes - Kubernetes MCP Server

## Important Note on Helm Values

The HelmRelease configuration includes custom values for mounting ConfigMaps and Secrets. The exact field names depend on the kubernetes-mcp-server Helm chart structure.

**If the deployment fails**, you may need to adjust the `values` section in `helmrelease.yaml` to match the chart's actual schema. Common alternatives:

### Option 1: volumeMounts and volumes (most common)

```yaml
values:
  volumeMounts:
    - name: config
      mountPath: /config
    - name: oauth-secret
      mountPath: /secrets
  
  volumes:
    - name: config
      configMap:
        name: kubernetes-mcp-server-config
    - name: oauth-secret
      secret:
        secretName: kubernetes-mcp-oauth-secret
  
  env:
    - name: CONFIG_FILE
      value: /config/config.toml
    - name: STS_CLIENT_SECRET_FILE
      value: /secrets/sts_client_secret
```

### Option 2: extraVolumeMounts and extraVolumes

```yaml
values:
  extraVolumeMounts:
    - name: config
      mountPath: /config
    - name: oauth-secret
      mountPath: /secrets
  
  extraVolumes:
    - name: config
      configMap:
        name: kubernetes-mcp-server-config
    - name: oauth-secret
      secret:
        secretName: kubernetes-mcp-oauth-secret
```

### Option 3: Direct pod spec override

```yaml
values:
  podSpec:
    volumes:
      - name: config
        configMap:
          name: kubernetes-mcp-server-config
      - name: oauth-secret
        secret:
          secretName: kubernetes-mcp-oauth-secret
    
    containers:
      - name: kubernetes-mcp-server
        volumeMounts:
          - name: config
            mountPath: /config
          - name: oauth-secret
            mountPath: /secrets
        env:
          - name: CONFIG_FILE
            value: /config/config.toml
          - name: STS_CLIENT_SECRET_FILE
            value: /secrets/sts_client_secret
```

## Checking the Chart Structure

To see the available values in the Helm chart:

```bash
# Pull the chart
helm pull oci://ghcr.io/containers/charts/kubernetes-mcp-server --version 0.1.0 --untar

# View values.yaml
cat kubernetes-mcp-server/values.yaml

# Or use helm show
helm show values oci://ghcr.io/containers/charts/kubernetes-mcp-server --version 0.1.0
```

## Alternative Configuration Approach

If the Helm chart doesn't support easy ConfigMap/Secret mounting, you can:

1. **Use environment variables directly** (if supported):
   ```yaml
   values:
     env:
       - name: REQUIRE_OAUTH
         value: "true"
       - name: OAUTH_AUDIENCE
         value: "kubernetes-mcp-server"
       - name: AUTHORIZATION_URL
         value: "https://authentik.k12n.com/application/o/kubernetes-mcp/"
       - name: STS_CLIENT_ID
         value: "kubernetes-mcp-server"
       - name: STS_CLIENT_SECRET
         valueFrom:
           secretKeyRef:
             name: kubernetes-mcp-oauth-secret
             key: sts_client_secret
   ```

2. **Create a custom deployment** (bypass Helm chart entirely):
   - Copy the generated Helm manifests
   - Convert to plain Kubernetes manifests
   - Add ConfigMap and Secret mounts directly
   - Remove the HelmRelease and use plain Deployment

## Server Configuration Options

The MCP server may support different configuration methods:

1. **Config file** (`config.toml`): Full configuration in TOML format
2. **Environment variables**: Individual settings as env vars
3. **Command-line flags**: Arguments passed to the server binary

Check the kubernetes-mcp-server documentation for supported configuration methods:
- [https://github.com/containers/kubernetes-mcp-server](https://github.com/containers/kubernetes-mcp-server)

## Testing After Deployment

Once deployed, verify the configuration was loaded correctly:

```bash
# Check environment variables
kubectl exec -n kubernetes-mcp-server deploy/kubernetes-mcp-server -- env | grep -E 'OAUTH|CONFIG|STS'

# Check mounted files
kubectl exec -n kubernetes-mcp-server deploy/kubernetes-mcp-server -- ls -la /config/
kubectl exec -n kubernetes-mcp-server deploy/kubernetes-mcp-server -- cat /config/config.toml

# Check server logs for config loading
kubectl logs -n kubernetes-mcp-server -l app.kubernetes.io/name=kubernetes-mcp-server | grep -i config
```

## Fallback Plan

If the Helm chart is too restrictive, we can create a simple Deployment manifest instead:

```bash
# Generate manifests from Helm
helm template kubernetes-mcp-server oci://ghcr.io/containers/charts/kubernetes-mcp-server \
  --version 0.1.0 \
  --namespace kubernetes-mcp-server \
  > kubernetes-mcp-server-manifests.yaml

# Edit the deployment to add ConfigMap/Secret mounts
# Then use plain manifests instead of HelmRelease
```
