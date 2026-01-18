#!/bin/bash
# Script to force delete a stuck HelmRelease when kubectl delete is hanging
# This handles the case where finalizers are waiting for Helm uninstall to complete

set -e

RELEASE_NAME="${1:-kube-prometheus-stack}"
NAMESPACE="${2:-monitoring}"

echo "🔧 Force deleting stuck HelmRelease: ${RELEASE_NAME} in ${NAMESPACE}"
echo "===================================================================="
echo ""
echo "⚠️  WARNING: This will force delete the HelmRelease by removing finalizers!"
echo "This should only be done if kubectl delete is hanging."
echo ""
echo "Press Ctrl+C to cancel, or Enter to continue..."
read

# Step 1: Check current state
echo "1️⃣ Checking current HelmRelease state..."
echo "----------------------------------------"
kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null || {
    echo "✅ HelmRelease not found (may already be deleted)"
    exit 0
}
echo ""

# Step 2: Check if it has a deletion timestamp (already being deleted)
echo "2️⃣ Checking if HelmRelease has deletion timestamp..."
echo "-----------------------------------------------------"
DELETION_TIMESTAMP=$(kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} -o jsonpath='{.metadata.deletionTimestamp}' 2>/dev/null || echo "")

if [ -n "${DELETION_TIMESTAMP}" ]; then
    echo "⚠️  HelmRelease is already marked for deletion but stuck"
    echo "   Deletion timestamp: ${DELETION_TIMESTAMP}"
else
    echo "ℹ️  HelmRelease is not yet marked for deletion"
fi
echo ""

# Step 3: Check finalizers
echo "3️⃣ Checking finalizers..."
echo "-------------------------"
FINALIZERS=$(kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} -o jsonpath='{.metadata.finalizers[*]}' 2>/dev/null || echo "")
if [ -n "${FINALIZERS}" ]; then
    echo "Finalizers found: ${FINALIZERS}"
    echo "⚠️  These finalizers are preventing deletion"
else
    echo "No finalizers found"
fi
echo ""

# Step 4: Try graceful deletion with --wait=false first
echo "4️⃣ Attempting graceful deletion with --wait=false..."
echo "-----------------------------------------------------"
echo "This won't wait for finalizers to complete"
kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE} --wait=false 2>/dev/null || echo "⚠️  Delete command failed or resource doesn't exist"
echo ""

# Step 5: Wait a moment
echo "5️⃣ Waiting 5 seconds..."
echo "------------------------"
sleep 5
echo ""

# Step 6: Check if it's still there
echo "6️⃣ Checking if HelmRelease still exists..."
echo "-------------------------------------------"
if kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null; then
    echo "⚠️  HelmRelease still exists, will remove finalizers"
    echo ""
    
    # Step 7: Patch to remove finalizers (force delete)
    echo "7️⃣ Removing finalizers to force deletion..."
    echo "--------------------------------------------"
    echo "⚠️  This will force delete the HelmRelease even if Helm uninstall hasn't completed"
    kubectl patch helmrelease ${RELEASE_NAME} -n ${NAMESPACE} -p '{"metadata":{"finalizers":[]}}' --type=merge
    
    echo ""
    echo "✅ Finalizers removed"
    
    # Step 8: Wait and verify deletion
    echo ""
    echo "8️⃣ Waiting for deletion to complete..."
    echo "--------------------------------------"
    sleep 5
    
    if kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null; then
        echo "⚠️  HelmRelease still exists, trying again..."
        kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE} --wait=false --grace-period=0 --force 2>/dev/null || true
        sleep 3
    fi
    
    if kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null; then
        echo "❌ HelmRelease still exists after removing finalizers"
        echo "You may need to delete it directly with:"
        echo "  kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE} --grace-period=0 --force"
    else
        echo "✅ HelmRelease deleted successfully"
    fi
else
    echo "✅ HelmRelease deleted successfully (with --wait=false)"
fi
echo ""

# Step 9: Check Helm release status
echo "9️⃣ Checking Helm release status..."
echo "-----------------------------------"
if helm list -n ${NAMESPACE} | grep -q ${RELEASE_NAME}; then
    echo "⚠️  Helm release still exists (HelmRelease was force deleted)"
    echo "   You may need to manually uninstall:"
    echo "   helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --timeout 30m"
else
    echo "✅ Helm release not found (clean state)"
fi
echo ""

# Summary
echo "===================================================================="
echo "📋 Summary"
echo "===================================================================="
echo ""
echo "Next steps:"
echo "1. If Helm release still exists, manually uninstall it:"
echo "   helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --timeout 30m --no-hooks"
echo ""
echo "2. If the HelmRelease still exists in git, Flux will recreate it"
echo "   Check: kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE}"
echo ""
echo "3. To prevent recreation, remove the HelmRelease from git first"
echo ""
echo "4. Clean up any remaining resources if needed:"
echo "   kubectl get all -n ${NAMESPACE}"
echo "   kubectl get pvc -n ${NAMESPACE}"
echo ""
