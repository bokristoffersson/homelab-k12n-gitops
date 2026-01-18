#!/bin/bash
# Script to fix stuck kube-prometheus-stack Helm release deletion
# This handles the timeout issue when Flux tries to delete the release

set -e

RELEASE_NAME="kube-prometheus-stack"
NAMESPACE="monitoring"

echo "🔧 Fixing stuck kube-prometheus-stack Helm release deletion"
echo "============================================================"
echo ""
echo "⚠️  WARNING: This script will delete resources!"
echo "Press Ctrl+C to cancel, or Enter to continue..."
read

# Step 1: Check current state
echo "1️⃣ Checking current state..."
echo "----------------------------"
kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null || echo "⚠️  HelmRelease not found"
helm list -n ${NAMESPACE} | grep ${RELEASE_NAME} || echo "⚠️  Helm release not found"
echo ""

# Step 2: Delete the HelmRelease resource first
echo "2️⃣ Deleting HelmRelease resource (this tells Flux to stop trying to delete)..."
echo "----------------------------------------------------------------------------"
kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE} --wait=false 2>/dev/null || echo "⚠️  HelmRelease already deleted or doesn't exist"
echo ""

# Step 3: Wait a moment for Flux to stop
echo "3️⃣ Waiting 10 seconds for Flux to stop trying to delete..."
echo "-----------------------------------------------------------"
sleep 10
echo ""

# Step 4: Try manual Helm uninstall with longer timeout
echo "4️⃣ Attempting manual Helm uninstall with 30-minute timeout..."
echo "--------------------------------------------------------------"
if helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --timeout 30m 2>/dev/null; then
    echo "✅ Helm uninstall succeeded"
else
    echo "⚠️  Helm uninstall failed or already uninstalled, trying with --no-hooks..."
    helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --timeout 30m --no-hooks 2>/dev/null || echo "⚠️  Helm uninstall failed (may already be uninstalled)"
fi
echo ""

# Step 5: Check for remaining resources
echo "5️⃣ Checking for remaining resources..."
echo "--------------------------------------"
echo "StatefulSets:"
kubectl get statefulsets -n ${NAMESPACE} 2>/dev/null || echo "No StatefulSets"
echo ""
echo "Deployments:"
kubectl get deployments -n ${NAMESPACE} 2>/dev/null || echo "No Deployments"
echo ""
echo "PVCs (these may need manual deletion):"
kubectl get pvc -n ${NAMESPACE} 2>/dev/null || echo "No PVCs"
echo ""

# Step 6: Option to manually delete remaining resources
echo "6️⃣ If resources remain, you may need to delete them manually..."
echo "--------------------------------------------------------------"
echo ""
echo "⚠️  If Helm release still exists in Helm but resources are gone:"
echo "   You may need to manually clean up Helm metadata:"
echo "   kubectl delete secret -n ${NAMESPACE} -l owner=helm,name=${RELEASE_NAME}"
echo ""
echo "⚠️  If PVCs remain and are stuck:"
echo "   kubectl delete pvc -n ${NAMESPACE} --all"
echo ""
echo "✅ Cleanup attempt complete!"
echo ""
echo "📋 Next steps:"
echo "1. If the HelmRelease still exists in git, Flux will recreate it"
echo "2. Check HelmRelease status: kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE}"
echo "3. To force recreation: flux reconcile kustomization <kustomization-name> -n flux-system"
echo ""
