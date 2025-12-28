# Kong with OIDC Plugin

Custom Kong image with the `kong-oidc` plugin installed for OAuth2/OpenID Connect authentication with Authentik.

## Base Image

- **Base**: `kong:3.8`
- **Plugin**: `kong-oidc` v1.3.0 (https://github.com/revomatico/kong-oidc)
- **Dependencies**: `lua-resty-openidc`

## Plugin Information

The `kong-oidc` plugin provides:
- OAuth2/OpenID Connect authentication
- Token introspection support
- Bearer token validation
- Integration with external identity providers (Authentik, Keycloak, etc.)

**Plugin Name**: `oidc` (not `openid-connect`)

## Building Locally

```bash
cd applications/kong-oidc
docker build -t kong-oidc:latest .

# Test locally
docker run --rm kong-oidc:latest kong version
```

## CI/CD

The image is automatically built and pushed to GitHub Container Registry via GitHub Actions:

- **Workflow**: `.github/workflows/kong-oidc.yml`
- **Trigger**: Push to main branch or tags matching `kong-oidc-v*`
- **Registry**: `ghcr.io/bokristoffersson/kong-oidc`
- **Tags**:
  - `main` - Latest from main branch
  - `v1.0.0` - Semantic version tags
  - `latest` - Latest stable release

## Usage in Kubernetes

The Kong deployment in `gitops/apps/base/kong/deployment.yaml` uses this custom image:

```yaml
spec:
  containers:
    - name: kong
      image: ghcr.io/bokristoffersson/kong-oidc:latest
```

## Plugin Configuration

Configure the OIDC plugin in `gitops/apps/base/kong/config.yaml`:

```yaml
plugins:
  - name: oidc
    config:
      client_id: $(KONG_OIDC_CLIENT_ID)
      client_secret: $(KONG_OIDC_CLIENT_SECRET)
      discovery: http://authentik-server.authentik.svc.cluster.local:9000/application/o/api-gateway/.well-known/openid-configuration
      introspection_endpoint: http://authentik-server.authentik.svc.cluster.local:9000/application/o/introspect/
      bearer_only: "yes"
      realm: kong
      introspection_endpoint_auth_method: client_secret_post
```

## References

- Kong OIDC Plugin: https://github.com/revomatico/kong-oidc
- lua-resty-openidc: https://github.com/zmartzone/lua-resty-openidc
- Kong Plugin Development: https://docs.konghq.com/gateway/latest/plugin-development/
