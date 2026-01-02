# Deployment Guide

## Kubernetes Deployment

Heatpump Web is deployed to Kubernetes via GitOps with Flux CD.

### Manifest Structure

```
gitops/apps/base/heatpump-web/
├── namespace.yaml
├── deployment.yaml
├── service.yaml
├── configmap.yaml
└── kustomization.yaml
```

### Environment Configuration

Configuration is provided via ConfigMap and injected at container startup:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: heatpump-web-config
  namespace: heatpump-web
data:
  VITE_API_URL: "https://heatpump.k12n.com"
  VITE_AUTHENTIK_URL: "https://authentik.k12n.com"
  VITE_OAUTH_CLIENT_ID: "heatpump-web"
  VITE_OAUTH_REDIRECT_URI: "https://heatpump.k12n.com/auth/callback"
```

### Deployment Resource

- **Image**: `ghcr.io/bokristoffersson/heatpump-web:latest`
- **Replicas**: 1
- **Resources**:
  - Requests: 50m CPU, 64Mi memory
  - Limits: 200m CPU, 128Mi memory
- **Probes**: HTTP readiness and liveness checks on port 80

### Service

- **Type**: ClusterIP
- **Port**: 80
- **Protocol**: TCP

### Ingress Routing

Traefik IngressRoute configuration:

```yaml
# Frontend (public)
- match: Host(`heatpump.k12n.com`)
  kind: Rule
  middlewares:
    - name: https-scheme
      namespace: traefik
  services:
    - name: heatpump-web
      namespace: heatpump-web
      port: 80

# API (protected by oauth2-proxy)
- match: Host(`heatpump.k12n.com`) && PathPrefix(`/api`)
  kind: Rule
  priority: 90
  middlewares:
    - name: https-scheme
      namespace: traefik
    - name: oauth2-proxy-auth
      namespace: traefik
  services:
    - name: homelab-api
      namespace: homelab-api
      port: 8080
```

### Cloudflare Tunnel

External access is provided via Cloudflare Tunnel:

```yaml
ingress:
  - hostname: heatpump.k12n.com
    service: http://traefik.traefik.svc.cluster.local:80
```

## Deployment Process

1. **Update code**: Push changes to main branch
2. **GitHub Actions**: Automatically builds and pushes Docker image
3. **Flux CD**: Detects new image and updates deployment
4. **Kubernetes**: Performs rolling update

## Manual Deployment

Force Flux to reconcile immediately:

```bash
flux reconcile kustomization heatpump-web --with-source
```

## Monitoring

Check deployment status:

```bash
kubectl get pods -n heatpump-web
kubectl logs -n heatpump-web -l app=heatpump-web
```

View in Backstage:
- Navigate to https://backstage.k12n.com
- Open the heatpump-web component
- Click "Kubernetes" tab to see live pod status
