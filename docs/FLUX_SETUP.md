# Flux Setup for Production

This guide covers installing and configuring Flux CD on your **production cluster only**.

> ⚠️ **Never install Flux on localhost** - Local development uses `kubectl apply -k` to avoid the risk of committing secrets to git.

## Prerequisites

- Production Kubernetes cluster (K3s, K8s, etc.)
- kubectl configured for production cluster
- Flux CLI installed
- GitHub repository with GitOps manifests

## Install Flux CLI

**macOS:**
```bash
brew install fluxcd/tap/flux
```

**Linux:**
```bash
curl -s https://fluxcd.io/install.sh | bash
```

Verify:
```bash
flux version
```

## Bootstrap Flux on Production

### Option 1: GitHub App (Recommended)

Using a GitHub App is more secure than personal tokens.

#### 1. Create GitHub App

1. Go to GitHub Settings → Developer settings → GitHub Apps
2. Create new GitHub App:
   - **Name**: `homelab-flux`
   - **Repository permissions**:
     - Contents: Read
     - Metadata: Read
   - **Webhook**: Disabled
3. Install the app on your GitOps repository
4. Note the **App ID** and **Installation ID**
5. Generate and download private key (.pem file)

#### 2. Create GitHub App Secret

```bash
# Switch to production cluster context
kubectl config use-context <production-context>

# Create Flux namespace
kubectl create namespace flux-system

# Create GitHub App secret
flux create secret githubapp flux-system \
  --app-id=YOUR_APP_ID \
  --app-installation-id=YOUR_INSTALLATION_ID \
  --app-private-key=./path/to/private-key.pem
```

#### 3. Bootstrap Flux

Create `flux-instance.yaml`:

```yaml
apiVersion: fluxcd.controlplane.io/v1
kind: FluxInstance
metadata:
  name: flux
  namespace: flux-system
spec:
  distribution:
    version: "2.x"
    registry: "ghcr.io/fluxcd"
  components:
    - source-controller
    - kustomize-controller
    - helm-controller
    - notification-controller
  cluster:
    type: kubernetes
    multitenant: false
    networkPolicy: true
    domain: "cluster.local"
  sync:
    kind: GitRepository
    provider: github
    url: "https://github.com/YOUR_USERNAME/YOUR_REPO.git"
    ref: "refs/heads/main"
    path: "clusters/homelab"  # Path to your cluster config
    pullSecret: "flux-system"
```

Apply:
```bash
kubectl apply -f flux-instance.yaml
```

### Option 2: Personal Access Token

If you don't want to use a GitHub App:

```bash
flux bootstrap github \
  --owner=YOUR_USERNAME \
  --repository=YOUR_REPO \
  --branch=main \
  --path=clusters/homelab \
  --personal
```

This will:
1. Create Flux namespace
2. Install Flux controllers
3. Create deploy key
4. Configure sync with your repository

## Verify Flux Installation

```bash
# Check Flux components
flux check

# View all Flux resources
flux get all

# Watch reconciliation
flux get kustomizations --watch

# View logs
flux logs --all-namespaces --follow
```

## Production Cluster Structure

Your production cluster should reference:

```
clusters/homelab/
├── infrastructure.yaml  # Points to gitops/infrastructure/controllers
└── apps.yaml           # Points to gitops/apps/homelab
```

These use **sealed secrets** (not plain secrets like localhost).

## Managing Secrets in Production

Production uses sealed-secrets controller:

```bash
# Install kubeseal CLI
KUBESEAL_VERSION='0.32.2'
curl -OL "https://github.com/bitnami-labs/sealed-secrets/releases/download/v${KUBESEAL_VERSION}/kubeseal-${KUBESEAL_VERSION}-linux-amd64.tar.gz"
tar -xvzf kubeseal-${KUBESEAL_VERSION}-linux-amd64.tar.gz kubeseal
sudo install -m 755 kubeseal /usr/local/bin/kubeseal

# Fetch the public key from your cluster
kubeseal --fetch-cert > pub-sealed-secrets.pem

# Create a sealed secret
kubectl create secret generic my-secret \
  --from-literal=password=supersecret \
  --dry-run=client -o yaml | \
  kubeseal --cert pub-sealed-secrets.pem -o yaml > my-sealed-secret.yaml

# Commit the sealed secret (safe to commit!)
git add my-sealed-secret.yaml
git commit -m "Add sealed secret"
git push
```

Flux will decrypt it on the cluster using the private key.

## Troubleshooting

### Flux Not Syncing

```bash
# Check source
flux get sources git

# Force reconciliation
flux reconcile source git flux-system
flux reconcile kustomization flux-system

# Check for errors
flux logs --level=error
```

### Reconciliation Failures

```bash
# Describe the resource
flux describe kustomization infrastructure-controllers

# Check events
kubectl get events -n flux-system --sort-by='.lastTimestamp'
```

### Suspend/Resume Sync

```bash
# Suspend (for maintenance)
flux suspend kustomization infrastructure-controllers

# Resume
flux resume kustomization infrastructure-controllers
```

## Best Practices

1. **Never commit plain secrets** - Always use sealed-secrets
2. **Use GitHub Apps** instead of personal tokens
3. **Monitor Flux logs** regularly
4. **Test locally first** with `kubectl apply -k`
5. **Use branches** for testing major changes
6. **Set up notifications** (Slack, Discord, etc.)

## References

- [Flux Documentation](https://fluxcd.io/flux/)
- [Bootstrap with GitHub App](https://fluxcd.io/blog/2025/04/flux-operator-github-app-bootstrap/)
- [Sealed Secrets](https://github.com/bitnami-labs/sealed-secrets)
