#!/bin/bash
# Script to rerun postgres migrations
# This is needed because Kubernetes Jobs are immutable and cannot be updated

set -e

NAMESPACE="${NAMESPACE:-heatpump-settings}"
JOB_NAME="${JOB_NAME:-postgres-migration}"

echo "🔄 Rerunning postgres migrations in namespace: $NAMESPACE"

# Check if Job exists
if kubectl get job "$JOB_NAME" -n "$NAMESPACE" &>/dev/null; then
    echo "📋 Found existing migration Job: $JOB_NAME"
    echo "🗑️  Deleting existing Job (required because Jobs are immutable)..."
    kubectl delete job "$JOB_NAME" -n "$NAMESPACE" --wait=true
    
    # Wait a moment for cleanup
    sleep 2
    echo "✅ Job deleted"
else
    echo "ℹ️  No existing Job found, will create new one"
fi

echo ""
echo "⏳ Waiting for Flux to recreate the Job..."
echo "   (This may take a few seconds)"
echo ""

# Wait for Flux to recreate the Job (with timeout)
MAX_WAIT=30
ELAPSED=0
while ! kubectl get job "$JOB_NAME" -n "$NAMESPACE" &>/dev/null; do
    if [ $ELAPSED -ge $MAX_WAIT ]; then
        echo "❌ Timeout waiting for Flux to recreate Job"
        echo "   You may need to trigger a Flux sync manually:"
        echo "   flux reconcile source git flux-system"
        exit 1
    fi
    sleep 2
    ELAPSED=$((ELAPSED + 2))
    echo -n "."
done

echo ""
echo "✅ Job recreated by Flux"
echo ""
echo "📊 Waiting for migration to complete..."
echo "   (You can watch progress with: kubectl logs -f job/$JOB_NAME -n $NAMESPACE)"
echo ""

# Wait for Job to complete
kubectl wait --for=condition=complete --timeout=300s job/$JOB_NAME -n "$NAMESPACE" 2>/dev/null && {
    echo ""
    echo "✅ Migration completed successfully!"
    kubectl logs job/$JOB_NAME -n "$NAMESPACE" --tail=50
} || {
    echo ""
    echo "❌ Migration failed or timed out"
    echo "   Check logs with: kubectl logs job/$JOB_NAME -n $NAMESPACE"
    exit 1
}
