#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

CLUSTER_NAME="${CLUSTER_NAME:-homelab-local}"
K3D_VERSION="${K3D_VERSION:-v5.7.4}"
FLUX_VERSION="${FLUX_VERSION:-2.4.0}"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking dependencies..."

    local missing_deps=()

    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi

    if ! command -v kubectl &> /dev/null; then
        missing_deps+=("kubectl")
    fi

    if ! command -v k3d &> /dev/null; then
        log_warn "k3d not found. Installation instructions:"
        echo "  macOS: brew install k3d"
        echo "  Linux: curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash"
        missing_deps+=("k3d")
    fi

    if ! command -v flux &> /dev/null; then
        log_warn "flux not found. Installation instructions:"
        echo "  macOS: brew install fluxcd/tap/flux"
        echo "  Linux: curl -s https://fluxcd.io/install.sh | bash"
        missing_deps+=("flux")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi

    log_info "All dependencies satisfied"
}

create_cluster() {
    log_info "Creating k3d cluster: $CLUSTER_NAME"

    # Check if cluster already exists
    if k3d cluster list | grep -q "$CLUSTER_NAME"; then
        log_warn "Cluster $CLUSTER_NAME already exists"
        read -p "Delete and recreate? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            log_info "Deleting existing cluster..."
            k3d cluster delete "$CLUSTER_NAME"
        else
            log_info "Using existing cluster"
            return
        fi
    fi

    # Create cluster with Traefik disabled (we'll install it via GitOps)
    k3d cluster create "$CLUSTER_NAME" \
        --agents 1 \
        --port "80:80@loadbalancer" \
        --port "443:443@loadbalancer" \
        --k3s-arg "--disable=traefik@server:*" \
        --k3s-arg "--disable=servicelb@server:*"

    log_info "Cluster created successfully"

    # Wait for cluster to be ready
    log_info "Waiting for cluster to be ready..."
    kubectl wait --for=condition=Ready nodes --all --timeout=60s
}

setup_flux() {
    log_info "Installing Flux..."

    # Check if flux is already installed
    if kubectl get namespace flux-system &> /dev/null; then
        log_warn "Flux is already installed"
        return
    fi

    # Install Flux
    flux install

    log_info "Waiting for Flux to be ready..."
    kubectl wait --for=condition=Ready pods --all -n flux-system --timeout=120s

    log_info "Flux installed successfully"
}

create_local_secrets() {
    log_info "Creating local secrets..."

    # Create namespace for sealed-secrets controller (even though we won't use it)
    kubectl create namespace sealed-secrets --dry-run=client -o yaml | kubectl apply -f -

    # Create dummy secrets that would normally be sealed
    # Cloudflare tunnel secret
    kubectl create secret generic cloudflare-tunnel-secret \
        -n cloudflare-tunnel \
        --from-literal=tunnel-token="dummy-token-for-local" \
        --dry-run=client -o yaml | kubectl apply -f -

    # Redpanda console auth
    kubectl create secret generic redpanda-console-auth \
        -n redpanda-v2 \
        --from-literal=jwt-signing-key="local-dev-secret-key" \
        --dry-run=client -o yaml | kubectl apply -f -

    log_info "Local secrets created"
}

create_local_secrets() {
    log_info "Creating local development secrets..."

    # Run the secrets creation script
    if [ -f "./scripts/create-local-secrets.sh" ]; then
        ./scripts/create-local-secrets.sh
    else
        log_warn "Secrets script not found, creating manually..."

        # Create namespaces
        kubectl create namespace cloudflare-tunnel --dry-run=client -o yaml | kubectl apply -f -
        kubectl create namespace redpanda-v2 --dry-run=client -o yaml | kubectl apply -f -
        kubectl create namespace cert-manager --dry-run=client -o yaml | kubectl apply -f -

        # Create secrets
        kubectl create secret generic cloudflare-tunnel-secret \
            -n cloudflare-tunnel \
            --from-literal=tunnel-token="local-dev-dummy-token" \
            --dry-run=client -o yaml | kubectl apply -f -

        kubectl create secret generic redpanda-console-auth \
            -n redpanda-v2 \
            --from-literal=jwt-signing-key="local-dev-jwt-secret-key" \
            --dry-run=client -o yaml | kubectl apply -f -

        kubectl create secret generic cloudflare-api-token \
            -n cert-manager \
            --from-literal=api-token="local-dev-dummy-cf-token" \
            --dry-run=client -o yaml | kubectl apply -f -
    fi

    log_info "Local secrets created"
}

apply_gitops() {
    log_info "Applying GitOps configuration..."

    # Create GitRepository
    cat <<EOF | kubectl apply -f -
apiVersion: source.toolkit.fluxcd.io/v1
kind: GitRepository
metadata:
  name: flux-system
  namespace: flux-system
spec:
  interval: 1m
  ref:
    branch: main
  url: https://github.com/bokristoffersson/homelab-k12n-gitops
EOF

    log_info "Waiting for GitRepository to sync..."
    sleep 5

    # Apply infrastructure
    kubectl apply -f ./gitops/clusters/local/infrastructure.yaml

    log_info "Waiting for infrastructure to be ready..."
    kubectl wait --for=condition=Ready kustomization/infrastructure-controllers \
        -n flux-system --timeout=5m || log_warn "Infrastructure not ready yet, check 'flux get kustomizations'"

    # Apply apps
    kubectl apply -f ./gitops/clusters/local/apps.yaml

    log_info "GitOps configuration applied"
}

print_next_steps() {
    cat <<EOF

${GREEN}========================================${NC}
${GREEN}Local Cluster Setup Complete!${NC}
${GREEN}========================================${NC}

Cluster: ${CLUSTER_NAME}
Context: k3d-${CLUSTER_NAME}

${YELLOW}Next Steps:${NC}

1. Verify cluster status:
   ${GREEN}kubectl get nodes${NC}

2. Watch Flux reconciliation:
   ${GREEN}flux get kustomizations --watch${NC}

3. Access Traefik dashboard (once deployed):
   ${GREEN}kubectl port-forward -n traefik svc/traefik 9000:9000${NC}
   Then open: http://localhost:9000/dashboard/

4. To delete the cluster:
   ${GREEN}k3d cluster delete ${CLUSTER_NAME}${NC}

${YELLOW}Development Tips:${NC}

- Fast reconciliation: flux reconcile kustomization <name> --with-source
- View logs: flux logs --all-namespaces --follow
- Suspend auto-sync: flux suspend kustomization <name>
- Resume auto-sync: flux resume kustomization <name>

EOF
}

main() {
    log_info "Starting local cluster setup..."

    check_dependencies
    create_cluster
    setup_flux
    create_local_secrets
    apply_gitops
    print_next_steps

    log_info "Setup complete!"
}

main "$@"
