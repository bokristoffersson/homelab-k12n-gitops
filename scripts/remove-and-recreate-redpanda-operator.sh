#!/bin/bash
# Script to remove redpanda-operator HelmRelease and let Flux recreate it
# This is useful for troubleshooting reconciliation issues

set -e

NAMESPACE="redpanda-system"
HELMRELEASE_NAME="redpanda-operator"

echo "🔍 Checking current state..."
kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} || echo "⚠️  HelmRelease not found (may already be deleted)"

echo ""
echo "📦 Checking if Helm release exists..."
helm list -n ${NAMESPACE} | grep ${HELMRELEASE_NAME} || echo "⚠️  Helm release not found"

echo ""
echo "🗑️  Deleting HelmRelease resource..."
kubectl delete helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} --wait=true || echo "⚠️  HelmRelease already deleted or doesn't exist"

echo ""
echo "⏳ Waiting 5 seconds for resources to be cleaned up..."
sleep 5

echo ""
echo "📋 Checking if Helm release still exists (it should be uninstalled automatically)..."
if helm list -n ${NAMESPACE} | grep -q ${HELMRELEASE_NAME}; then
    echo "⚠️  Helm release still exists, manually uninstalling..."
    helm uninstall ${HELMRELEASE_NAME} -n ${NAMESPACE} || echo "⚠️  Failed to uninstall (may already be uninstalled)"
else
    echo "✅ Helm release already removed"
fi

echo ""
echo "✅ Cleanup complete!"
echo ""
echo "🔄 Flux will now recreate the HelmRelease from git..."
echo "   Monitor with: kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} -w"
echo ""
echo "💡 To force immediate reconciliation after Flux recreates it:"
echo "   flux reconcile helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
