# Local Development Environment

This guide will help you set up a local Kubernetes cluster for faster development and testing of your GitOps configurations.

## Why Local Development?

- **Faster iteration**: Test changes locally before pushing to production
- **Resource efficient**: Reduced resource requirements compared to production
- **No cloud costs**: Runs entirely on your machine
- **Simplified setup**: No sealed secrets, no external dependencies

## Prerequisites

### Required Tools

#### 1. Docker

**macOS:**
```bash
brew install --cask docker
# Or download from: https://www.docker.com/products/docker-desktop
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install docker.io
sudo usermod -aG docker $USER

# Fedora/RHEL
sudo dnf install docker
sudo systemctl start docker
sudo systemctl enable docker
sudo usermod -aG docker $USER
```

#### 2. kubectl

**macOS:**
```bash
brew install kubectl
```

**Linux:**
```bash
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
```

**Windows:**
```powershell
choco install kubernetes-cli
```

#### 3. Helm (optional but recommended)

**macOS:**
```bash
brew install helm
```

**Linux:**
```bash
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
```

**Windows:**
```powershell
choco install kubernetes-helm
```

#### 4. k3d

**macOS:**
```bash
brew install k3d
```

**Linux:**
```bash
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash
```

**Windows:**
```powershell
choco install k3d
```

#### 5. Flux CLI

**macOS:**
```bash
brew install fluxcd/tap/flux
```

**Linux:**
```bash
curl -s https://fluxcd.io/install.sh | bash
```

**Windows:**
```powershell
choco install flux
```

### Verify Installation

```bash
docker --version
kubectl version --client
helm version
k3d version
flux version
```

## Quick Start

### 1. Create the Local Cluster

From the repository root:

```bash
./scripts/setup-local-cluster.sh
```

This script will:
- Create a k3d cluster named `homelab-local`
- Install Flux CD
- Set up Traefik as ingress controller
- Create necessary namespaces and secrets
- Bootstrap GitOps

### 2. Verify the Cluster

```bash
# Check cluster nodes
kubectl get nodes

# Check Flux components
flux check

# Watch GitOps reconciliation
flux get kustomizations --watch
```

### 3. Access Services

#### Traefik Dashboard

```bash
kubectl port-forward -n traefik svc/traefik 9000:9000
```

Open http://localhost:9000/dashboard/

#### Redpanda Console

```bash
kubectl port-forward -n redpanda-v2 svc/redpanda-v2-console 8080:8080
```

Open http://localhost:8080

#### Prometheus

```bash
kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090
```

Open http://localhost:9090

## Development Workflow

### Testing Configuration Changes

1. **Make changes** to your GitOps files locally

2. **Commit and push** to your branch:
   ```bash
   git add .
   git commit -m "feat: test new configuration"
   git push
   ```

3. **Trigger reconciliation** (if you don't want to wait):
   ```bash
   flux reconcile source git flux-system
   flux reconcile kustomization infrastructure-controllers
   flux reconcile kustomization redpanda-v2
   ```

4. **Watch logs**:
   ```bash
   flux logs --all-namespaces --follow
   ```

### Working with Helm Releases

```bash
# List all Helm releases managed by Flux
flux get helmreleases --all-namespaces

# View Helm values for a release
helm get values -n redpanda-v2 redpanda-v2

# Suspend auto-reconciliation (for debugging)
flux suspend helmrelease -n redpanda-v2 redpanda-v2

# Resume auto-reconciliation
flux resume helmrelease -n redpanda-v2 redpanda-v2
```

### Debugging

```bash
# Check events
kubectl get events -n <namespace> --sort-by='.lastTimestamp'

# Describe resources
kubectl describe helmrelease -n redpanda-v2 redpanda-v2

# View pod logs
kubectl logs -n <namespace> <pod-name> --follow

# Get shell in pod
kubectl exec -it -n <namespace> <pod-name> -- /bin/sh
```

## Local vs Production Differences

| Feature | Production | Local |
|---------|-----------|-------|
| Sealed Secrets | ✅ Enabled | ❌ Regular secrets |
| Cloudflare Tunnel | ✅ Enabled | ❌ Disabled |
| Storage Class | `longhorn` | `local-path` |
| Resources | Full allocation | Reduced (50%) |
| Ingress | Cloudflare + Cert | Traefik + HTTP |
| PVC Sizes | 50Gi+ | 5-10Gi |

## Resource Requirements

**Minimum:**
- CPU: 4 cores
- RAM: 8 GB
- Disk: 20 GB free

**Recommended:**
- CPU: 6+ cores
- RAM: 16 GB
- Disk: 50 GB free

## Customizing Your Local Environment

### Adding More Apps

Edit `gitops/clusters/local/apps.yaml` to add more applications:

```yaml
---
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: my-app
  namespace: flux-system
spec:
  interval: 5m
  sourceRef:
    kind: GitRepository
    name: flux-system
  path: ./gitops/apps/local/my-app
  prune: true
  wait: true
```

### Adjusting Resources

Create overlays in `gitops/apps/local/<app-name>/` to patch resource requirements.

Example (see `gitops/apps/local/redpanda-v2/kustomization.yaml`):

```yaml
patches:
  - target:
      kind: HelmRelease
      name: my-app
    patch: |-
      - op: replace
        path: /spec/values/resources/requests/memory
        value: 256Mi
```

### Switching Cluster Contexts

```bash
# List contexts
kubectl config get-contexts

# Switch to local
kubectl config use-context k3d-homelab-local

# Switch to production
kubectl config use-context homelab-production
```

## Cleanup

### Delete the Local Cluster

```bash
k3d cluster delete homelab-local
```

### Clean Docker Resources

```bash
docker system prune -a --volumes
```

## Troubleshooting

### Cluster Won't Start

```bash
# Check Docker
docker ps

# Recreate cluster
k3d cluster delete homelab-local
./scripts/setup-local-cluster.sh
```

### Flux Reconciliation Fails

```bash
# Check Flux logs
flux logs --all-namespaces --level=error

# Force reconciliation
flux reconcile kustomization flux-system --with-source
```

### Out of Resources

```bash
# Check resource usage
kubectl top nodes
kubectl top pods --all-namespaces

# Reduce replicas or remove apps from local/apps.yaml
```

### Port Already in Use

```bash
# Find process using port 80 or 443
sudo lsof -i :80
sudo lsof -i :443

# Kill the process or change k3d port mappings in setup script
```

## Tips & Tricks

### Fast Reconciliation

```bash
# Reconcile everything
flux reconcile kustomization flux-system --with-source

# Reconcile specific app
flux reconcile helmrelease -n redpanda-v2 redpanda-v2
```

### Development Aliases

Add to your `~/.bashrc` or `~/.zshrc`:

```bash
alias k='kubectl'
alias kgp='kubectl get pods'
alias kgs='kubectl get svc'
alias kgn='kubectl get nodes'
alias fl='flux'
alias flg='flux get all --all-namespaces'
alias flr='flux reconcile kustomization flux-system --with-source'
```

### Using Local Registry

For testing custom container images:

```bash
# Create cluster with registry
k3d cluster create homelab-local \
  --registry-create homelab-local-registry:0.0.0.0:5001

# Tag and push image
docker tag myapp:latest localhost:5001/myapp:latest
docker push localhost:5001/myapp:latest

# Use in manifests as: homelab-local-registry:5001/myapp:latest
```

## Next Steps

- Review the [GitOps structure](../README.md)
- Learn about [Flux concepts](https://fluxcd.io/flux/concepts/)
- Explore [Kustomize overlays](https://kubectl.docs.kubernetes.io/references/kustomize/kustomization/)

## Support

If you encounter issues:
1. Check the [troubleshooting section](#troubleshooting)
2. Review Flux logs: `flux logs --all-namespaces --follow`
3. Check cluster events: `kubectl get events --all-namespaces`
