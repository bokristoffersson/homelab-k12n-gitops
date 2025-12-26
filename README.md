# Homelab GitOps Setup with K3s and FluxCD

## 🚀 Quick Start

### Local Development

Test changes safely on your machine before pushing to production. See the [Quick Start Guide](docs/QUICK_START.md)!

```bash
# Create local k3d cluster (no Flux, no git commits needed)
./scripts/setup-local-cluster.sh

# Apply your changes directly
make dev-apply-redpanda

# Access services
make port-redpanda
```

**Safe**: Secrets never leave your machine. No risk of committing secrets to git.

📖 **Documentation**:
- [Local Development Guide](docs/LOCAL_DEVELOPMENT.md)
- [Architecture & Principles](docs/ARCHITECTURE.md)
- [Flux Setup (Production)](docs/FLUX_SETUP.md)
- [Database Backup Setup](docs/BACKUP_SETUP.md)

---

## 🏗️ Infrastructure Overview

- **Control Plane**: Raspberry Pi 4 (`p0.local`)
- **Worker Node**: Raspberry Pi 5 with NVMe HAT
- **GitOps**: FluxCD with GitHub App authentication
- **Repository Structure**: Environment-based (homelab) with Kustomize overlays

## 🚀 Setup Steps Completed

### 1. K3s Cluster Setup

```
# On server node prepare data directory for k3s
mkdir /mnt/data/k3s
sudo chown root:root /mnt/data/k3s
sudo chmod 755 /mnt/data/k3s
```

#### Control Plane (Raspberry Pi 5 with NVMe HAT)
```bash
# Install K3s on the control plane node, specify DNS name for Control plane for to use kubectl outside cluster
curl -sfL https://get.k3s.io | sh -s - --data-dir /mnt/data/k3s --tls-san <CONTROL_PLANE_DNS_NAME>

# FIxing Permission error see link: https://dev.to/olymahmud/resolving-the-k3s-config-file-permission-denied-error-27e5
export KUBECONFIG=~/.kube/config
mkdir -p ~/.kube
sudo k3s kubectl config view --raw > "$KUBECONFIG"
chmod 600 "$KUBECONFIG"
nano ~/.profile
# Add export KUBECONFIG=~/.kube/config
ource ~/.profile

# Get the node token for worker nodes
sudo cat /mnt/data/k3s/server/node-token
```

```bash
# On each agent node
export K3S_NOE_NAME=agent001
```

#### Worker Node (Raspberry Pi 4)
```bash
# Join the worker node to the cluster
curl -sfL https://get.k3s.io | K3S_URL=https://<CONTROL_PLANE_IP>:6443 K3S_TOKEN=<NODE_TOKEN> sh - 
# NVMe mounted at /mnt/data
```

#### Verify Cluster
```bash
kubectl get nodes
# Should show both Pi 5 (control-plane) and Pi 4 (agent001) as Ready
```

Add to ~/.profile
```bash
# Set alias
alias k=kubectl
```
#### Setup kubectl on development computer
```bash
Copy ~/.kube/config from control-node to development computer.
```

### 2. GitOps Repository Structure

Created a GitHub repository with the following structure:
```
homelab-k12n-gitops/
├── clusters/
│   ├── homelab/
│   │   ├── infrastructure.yaml   # Points to infrastructure configs
│   │   └── apps.yaml            # Points to test app configs
├── infrastructure/
│   ├── sources/                # Git repos, Helm repos
│   ├── crds/                   # Custom Resource Definitions
│   ├── controllers/            # Controllers/Operators
│   └── configs/                # ConfigMaps, policies, etc.
└── apps/
    ├── base/                   # Base Kustomize configurations
    ├── homelab/                # Homelab environment overlays

### 3. GitHub App Setup

#### Create GitHub App
1. Navigate to GitHub Settings → Developer settings → GitHub Apps
2. Create new GitHub App with:
   - **App name**: `homelab-k12n-gitops`
   - **Repository permissions**:
     - Contents: Read
     - Metadata: Read
     - Pull requests: Read and write
   - **Webhook**: Disabled

#### Install GitHub App
1. Install the app on your GitOps repository
2. Note the **App ID** (numeric, e.g., `123456`)
3. Note the **Installation ID** (from URL after installation)
4. Generate and download private key (.pem file)
```

### 4. Flux Installation and Bootstrap

#### Install Prerequisites on Control Plane
```bash
# Install Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Install Flux CLI
curl -s https://fluxcd.io/install.sh | sudo bash
```

#### Install Flux Operator
```bash
helm install flux-operator oci://ghcr.io/controlplaneio-fluxcd/charts/flux-operator \
  --namespace flux-system \
  --create-namespace
```

#### Create GitHub App Secret
```bash
flux create secret githubapp flux-system \
  --app-id=YOUR_APP_ID \
  --app-installation-id=YOUR_INSTALLATION_ID \
  --app-private-key=./path/to/private-key.pem
```

#### Bootstrap Flux with FluxInstance
[Bootstrap Flux with github app](https://fluxcd.io/blog/2025/04/flux-operator-github-app-bootstrap/)

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
    - image-reflector-controller
    - image-automation-controller
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
    path: "clusters/test"
    pullSecret: "flux-system"
```

```
kubectl apply -f flux.yaml    # file above
```

### 5. Sealing secrets

[Install kubeseal](https://github.com/bitnami-labs/sealed-secrets?tab=readme-ov-file#linux)
```
KUBESEAL_VERSION='0.32.2'
curl -OL "https://github.com/bitnami-labs/sealed-secrets/releases/download/v${KUBESEAL_VERSION:?}/kubeseal-${KUBESEAL_VERSION:?}-linux-amd64.tar.gz"
tar -xvzf kubeseal-${KUBESEAL_VERSION:?}-linux-amd64.tar.gz kubeseal
sudo install -m 755 kubeseal /usr/local/bin/kubeseal
```

### 5. Test Application Deployment

#### Created Simple Whoami Test App

**Base Configuration** (`apps/base/whoami/`):
- `deployment.yaml`: 2 replicas of traefik/whoami
- `service.yaml`: ClusterIP service
- `kustomization.yaml`: Base Kustomize config

**Homelab Overlay** (`apps/homelab/whoami/`):
- `kustomization.yaml`: Reduces replicas to 1, adds homelab- prefix

#### Verification
```bash
# Check deployment
kubectl get deployment homelab-whoami -n default

# Test the application
kubectl port-forward service/homelab-whoami 8080:80 -n default
curl http://localhost:8080
```

## ✅ Success Verification

### Flux Health Check
```bash
# Overall Flux status
flux check
flux get all

# Check specific resources
kubectl get fluxinstance -n flux-system
kubectl get gitrepository -n flux-system
kubectl get kustomization -n flux-system
```

### Application Status
```bash
# Check whoami deployment
kubectl get all -l app=whoami -n default

# View application response
kubectl port-forward service/homelab-whoami 8080:80 -n default

# And in another terminal
curl http://localhost:8080
```

## 🔧 Key Benefits Achieved

1. **Declarative Infrastructure**: All cluster state managed through Git
2. **Secure Authentication**: GitHub App eliminates user-tied credentials
3. **Environment Separation**: Clean test/prod separation with overlays
4. **Automated Deployments**: Changes in Git automatically sync to cluster
5. **Scalable Architecture**: Easy to add more applications and environments

## 📚 Key Components

- **K3s**: Lightweight Kubernetes distribution perfect for homelab
- **FluxCD**: GitOps operator for continuous deployment
- **Kustomize**: Configuration management with base + overlays
- **GitHub Apps**: Secure, organization-level Git authentication
- **Helm**: Package manager for Kubernetes applications

## 🚀 Next Steps

- [ ] Add monitoring stack (Prometheus/Grafana)
- [ ] Set up production environment
- [ ] Add SSL/TLS with cert-manager
- [ ] Implement image automation
- [ ] Add more applications to the stack

## 🎉 Result

Successfully deployed a production-ready GitOps workflow on a Raspberry Pi homelab cluster! The whoami application serves as proof that the entire pipeline (Git → Flux → Kubernetes → Application) is working correctly.

---
*Setup completed on a Raspberry Pi 4 + Pi 5 homelab cluster with GitOps automation via FluxCD* 🏠⚙️