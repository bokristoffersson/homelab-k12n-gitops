# Quick Start - Local Development

Get up and running with a local Kubernetes cluster in minutes!

## Prerequisites

Install these tools (see [LOCAL_DEVELOPMENT.md](./LOCAL_DEVELOPMENT.md#prerequisites) for details):

- Docker
- kubectl
- k3d
- flux CLI

**One-liner for macOS:**
```bash
brew install docker kubectl k3d fluxcd/tap/flux
```

## Setup (3 Steps)

### 1. Clone the Repository

```bash
git clone https://github.com/bokristoffersson/homelab-k12n-gitops.git
cd homelab-k12n-gitops
```

### 2. Choose Your Workflow

**Fast Development (Recommended):**
```bash
./scripts/setup-local-cluster-simple.sh
# Or
make local-up-simple
```

**Full GitOps Testing:**
```bash
./scripts/setup-local-cluster.sh
# Or
make local-up
```

### 3. Deploy Your Apps

**Simple cluster (direct apply):**
```bash
# Apply infrastructure
make dev-apply-infra

# Apply apps
make dev-apply-redpanda

# Watch deployment
make dev-watch
```

**GitOps cluster (Flux auto-sync):**
```bash
# Check Flux status
flux get kustomizations

# Watch reconciliation
flux logs --all-namespaces --follow
```

## Access Your Services

### Redpanda Console

```bash
kubectl port-forward -n redpanda-v2 svc/redpanda-v2-console 8080:8080
```
→ http://localhost:8080

### Traefik Dashboard

```bash
kubectl port-forward -n traefik svc/traefik 9000:9000
```
→ http://localhost:9000/dashboard/

### Prometheus

```bash
kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090
```
→ http://localhost:9090

## Development Workflow

### Fast Iteration (Simple Cluster)

1. **Edit** your GitOps files locally
2. **Apply** directly:
   ```bash
   make dev-apply-redpanda
   # Or
   kubectl apply -k gitops/apps/local/redpanda-v2
   ```
3. **Watch** the changes:
   ```bash
   make dev-watch
   ```

**No git commits needed!** Perfect for experimenting.

### GitOps Testing (Full Cluster)

1. **Edit** your GitOps files
2. **Commit & Push**:
   ```bash
   git add .
   git commit -m "feat: your changes"
   git push
   ```
3. **Reconcile** (or wait for auto-sync):
   ```bash
   make flux-reconcile
   ```

## Useful Commands

```bash
# Watch reconciliation
flux get kustomizations --watch

# View logs
flux logs --all-namespaces --follow

# Restart a deployment
kubectl rollout restart deployment -n <namespace> <name>

# Delete cluster
k3d cluster delete homelab-local
```

## Troubleshooting

**Cluster won't start?**
```bash
docker ps  # Check Docker is running
k3d cluster delete homelab-local
./scripts/setup-local-cluster.sh  # Try again
```

**Flux errors?**
```bash
flux logs --all-namespaces --level=error
```

**Out of memory?**
- Close other applications
- Increase Docker memory (Settings → Resources)
- Remove apps from `gitops/clusters/local/apps.yaml`

## Next Steps

- 📖 Read the full [Local Development Guide](./LOCAL_DEVELOPMENT.md)
- 🔧 Customize apps in `gitops/apps/local/`
- 🚀 Deploy to production using the homelab cluster config

## Need Help?

Check the [troubleshooting section](./LOCAL_DEVELOPMENT.md#troubleshooting) in the full guide.
