#!/usr/bin/env bash
set -euo pipefail

# Simple local cluster setup WITHOUT Flux
# For rapid development using kubectl apply -k

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

CLUSTER_NAME="${CLUSTER_NAME:-homelab-local}"

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
        log_warn "k3d not found. Install with: brew install k3d"
        missing_deps+=("k3d")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi

    log_info "All dependencies satisfied"
}

create_cluster() {
    log_info "Creating k3d cluster: $CLUSTER_NAME"

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

    # Create minimal cluster with Traefik disabled (we'll install via manifests)
    k3d cluster create "$CLUSTER_NAME" \
        --agents 1 \
        --port "80:80@loadbalancer" \
        --port "443:443@loadbalancer" \
        --k3s-arg "--disable=traefik@server:*" \
        --k3s-arg "--disable=servicelb@server:*"

    log_info "Cluster created successfully"
    kubectl wait --for=condition=Ready nodes --all --timeout=60s
}

create_namespaces() {
    log_info "Creating namespaces..."

    kubectl create namespace flux-system --dry-run=client -o yaml | kubectl apply -f -
    kubectl create namespace cert-manager --dry-run=client -o yaml | kubectl apply -f -
    kubectl create namespace traefik --dry-run=client -o yaml | kubectl apply -f -
    kubectl create namespace redpanda-v2 --dry-run=client -o yaml | kubectl apply -f -
    kubectl create namespace monitoring --dry-run=client -o yaml | kubectl apply -f -
    kubectl create namespace cloudflare-tunnel --dry-run=client -o yaml | kubectl apply -f -

    log_info "Namespaces created"
}

create_secrets() {
    log_info "Creating development secrets..."

    # Redpanda console auth
    kubectl create secret generic redpanda-console-auth \
        -n redpanda-v2 \
        --from-literal=jwt-signing-key="local-dev-jwt-secret-key" \
        --dry-run=client -o yaml | kubectl apply -f -

    # Cloudflare (dummy for apps that reference it)
    kubectl create secret generic cloudflare-tunnel-secret \
        -n cloudflare-tunnel \
        --from-literal=tunnel-token="local-dev-dummy-token" \
        --dry-run=client -o yaml | kubectl apply -f -

    kubectl create secret generic cloudflare-api-token \
        -n cert-manager \
        --from-literal=api-token="local-dev-dummy-token" \
        --dry-run=client -o yaml | kubectl apply -f -

    log_info "Secrets created"
}

print_usage() {
    cat <<EOF

${GREEN}========================================${NC}
${GREEN}Local Cluster Ready!${NC}
${GREEN}========================================${NC}

Cluster: ${CLUSTER_NAME}
Context: k3d-${CLUSTER_NAME}

${YELLOW}Quick Development Workflow:${NC}

1. Apply infrastructure directly:
   ${GREEN}kubectl apply -k gitops/infrastructure/controllers-local${NC}

2. Apply applications directly:
   ${GREEN}kubectl apply -k gitops/apps/local/redpanda-v2${NC}
   ${GREEN}kubectl apply -k gitops/apps/local/monitoring${NC}

3. Make changes and re-apply:
   ${GREEN}kubectl apply -k gitops/apps/local/redpanda-v2${NC}

4. Watch resources:
   ${GREEN}kubectl get pods -n redpanda-v2 --watch${NC}

${YELLOW}Using the Makefile:${NC}

   ${GREEN}make dev-apply-infra${NC}    # Apply infrastructure
   ${GREEN}make dev-apply-apps${NC}     # Apply all apps
   ${GREEN}make dev-apply-redpanda${NC}  # Apply just Redpanda

${YELLOW}Access Services:${NC}

   ${GREEN}make port-redpanda${NC}       # http://localhost:8080
   ${GREEN}make port-traefik${NC}        # http://localhost:9000/dashboard/

${YELLOW}Cleanup:${NC}

   ${GREEN}k3d cluster delete ${CLUSTER_NAME}${NC}

EOF
}

main() {
    log_info "Starting simple local cluster setup (no Flux)..."

    check_dependencies
    create_cluster
    create_namespaces
    create_secrets
    print_usage

    log_info "Setup complete! Use 'kubectl apply -k' to deploy resources."
}

main "$@"
