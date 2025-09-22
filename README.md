# Homelab GitOps Setup with K3s and FluxCD

## 🏗️ Infrastructure Overview

- **Control Plane**: Raspberry Pi 4 (`p0.local`)
- **Worker Node**: Raspberry Pi 5 with NVMe HAT
- **GitOps**: FluxCD with GitHub App authentication
- **Repository Structure**: Environment-based (homelab) with Kustomize overlays

## 🚀 Setup Steps Completed

### 1. K3s Cluster Setup

#### Control Plane (Raspberry Pi 4)
```bash
# Install K3s on the control plane node
curl -sfL https://get.k3s.io | sh -

# Get the node token for worker nodes
sudo cat /var/lib/rancher/k3s/server/node-token
```

#### Worker Node (Raspberry Pi 5 with NVMe HAT)
```bash
# Join the worker node to the cluster
curl -sfL https://get.k3s.io | K3S_URL=https://p0.local:6443 K3S_TOKEN=<NODE_TOKEN> sh - --node-name worker-pi5 --data-dir /mnt/data/k3s
# NVMe mounted at /mnt/data
```

#### Verify Cluster
```bash
kubectl get nodes
# Should show both Pi 4 (control-plane) and Pi 5 (worker) as Ready
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