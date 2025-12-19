#!/usr/bin/env bash
set -euo pipefail

# This script creates local development secrets that replace sealed secrets in production

GREEN='\033[0;32m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

create_namespace_if_not_exists() {
    local namespace=$1
    if ! kubectl get namespace "$namespace" &> /dev/null; then
        log_info "Creating namespace: $namespace"
        kubectl create namespace "$namespace"
    fi
}

log_info "Creating local development secrets..."

# Cloudflare tunnel namespace and secret
create_namespace_if_not_exists "cloudflare-tunnel"
kubectl create secret generic cloudflare-tunnel-secret \
    -n cloudflare-tunnel \
    --from-literal=tunnel-token="local-dev-dummy-token" \
    --dry-run=client -o yaml | kubectl apply -f -
log_info "Created cloudflare-tunnel-secret"

# Redpanda console auth
create_namespace_if_not_exists "redpanda-v2"
kubectl create secret generic redpanda-console-auth \
    -n redpanda-v2 \
    --from-literal=jwt-signing-key="local-dev-jwt-secret-key-12345" \
    --dry-run=client -o yaml | kubectl apply -f -
log_info "Created redpanda-console-auth"

# Cert-manager cloudflare credentials (if using cert-manager)
create_namespace_if_not_exists "cert-manager"
kubectl create secret generic cloudflare-api-token \
    -n cert-manager \
    --from-literal=api-token="local-dev-dummy-cf-token" \
    --dry-run=client -o yaml | kubectl apply -f -
log_info "Created cloudflare-api-token for cert-manager"

log_info "All local secrets created successfully!"
