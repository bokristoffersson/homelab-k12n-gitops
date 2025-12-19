#!/usr/bin/env bash
set -euo pipefail

echo "Installing Redpanda (simple, no operator)..."

# Add Redpanda Helm repo if not already added
if ! helm repo list | grep -q "^redpanda"; then
    echo "Adding Redpanda Helm repository..."
    helm repo add redpanda https://charts.redpanda.com
fi

helm repo update

# Install or upgrade Redpanda
helm upgrade --install redpanda-v2 redpanda/redpanda \
    --namespace redpanda-v2 \
    --create-namespace \
    --values values.yaml \
    --wait \
    --timeout 5m

echo ""
echo "✅ Redpanda installed successfully!"
echo ""
echo "Next steps:"
echo "1. Wait for pod to be ready:"
echo "   kubectl wait --for=condition=Ready pod/redpanda-v2-0 -n redpanda-v2 --timeout=120s"
echo ""
echo "2. Create topics using rpk:"
echo "   make rpk-create-topics"
echo ""
echo "3. Access console:"
echo "   make port-redpanda"
echo "   http://localhost:8080"
