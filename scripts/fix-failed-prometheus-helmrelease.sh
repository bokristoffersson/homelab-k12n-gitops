#!/bin/bash
# Script to fix failed kube-prometheus-stack HelmRelease that's stuck in failed uninstall state

set -e

RELEASE_NAME="kube-prometheus-stack"
NAMESPACE="monitoring"

echo "🔧 Fixing failed kube-prometheus-stack HelmRelease"
echo "=================================================="
echo ""
echo "Current status shows: Helm uninstall failed"
echo "This script will:"
echo "1. Delete the HelmRelease resource"
echo "2. Manually uninstall the Helm release"
echo "3. Clean up any remaining resources if needed"
echo ""
echo "Press Ctrl+C to cancel, or Enter to continue..."
read

# Step 1: Check current state
echo "1️⃣ Checking current state..."
echo "----------------------------"
kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE}
echo ""

# Step 2: Check if Helm release exists
echo "2️⃣ Checking if Helm release exists..."
echo "-------------------------------------"
helm list -n ${NAMESPACE} | grep ${RELEASE_NAME} || echo "⚠️  Helm release not found"
echo ""

# Step 3: Delete HelmRelease with --wait=false
echo "3️⃣ Deleting HelmRelease (without waiting)..."
echo "---------------------------------------------"
kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE} --wait=false 2>/dev/null || echo "⚠️  Delete command failed"
echo ""

# Step 4: Wait a moment
echo "4️⃣ Waiting 5 seconds..."
echo "------------------------"
sleep 5
echo ""

# Step 5: Check if it's still there (might have finalizers)
echo "5️⃣ Checking if HelmRelease still exists..."
echo "-------------------------------------------"
if kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null; then
    echo "⚠️  HelmRelease still exists (likely has finalizers)"
    echo "Removing finalizers..."
    kubectl patch helmrelease ${RELEASE_NAME} -n ${NAMESPACE} -p '{"metadata":{"finalizers":[]}}' --type=merge 2>/dev/null || echo "⚠️  Patch failed"
    sleep 3
    
    # Try delete again
    kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE} --wait=false --grace-period=0 2>/dev/null || true
    sleep 2
fi

# Step 6: Verify HelmRelease is gone
echo "6️⃣ Verifying HelmRelease is deleted..."
echo "--------------------------------------"
if kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null; then
    echo "❌ HelmRelease still exists - manual intervention may be needed"
    echo "Try: kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE} --grace-period=0 --force"
else
    echo "✅ HelmRelease deleted successfully"
fi
echo ""

# Step 7: Check if Helm release still exists and uninstall it
echo "7️⃣ Checking if Helm release needs manual uninstall..."
echo "-----------------------------------------------------"
if helm list -n ${NAMESPACE} | grep -q ${RELEASE_NAME}; then
    echo "⚠️  Helm release still exists - attempting manual uninstall..."
    echo ""
    
    # Try with normal timeout
    if helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --timeout 30m 2>/dev/null; then
        echo "✅ Helm release uninstalled successfully"
    else
        echo "⚠️  Normal uninstall failed, trying with --no-hooks..."
        helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --timeout 30m --no-hooks 2>/dev/null || {
            echo "❌ Helm uninstall failed completely"
            echo ""
            echo "⚠️  Helm release may need manual cleanup"
            echo "You may need to:"
            echo "1. Delete resources manually"
            echo "2. Delete Helm metadata secrets/configmaps"
            echo ""
            echo "See troubleshooting guide for manual cleanup steps"
        }
    fi
else
    echo "✅ Helm release already uninstalled (clean state)"
fi
echo ""

# Step 8: Final status check
echo "8️⃣ Final status check..."
echo "------------------------"
echo "HelmRelease status:"
kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} 2>/dev/null || echo "✅ HelmRelease deleted"
echo ""
echo "Helm release status:"
helm list -n ${NAMESPACE} | grep ${RELEASE_NAME} || echo "✅ Helm release uninstalled"
echo ""

# Summary
echo "===================================================================="
echo "📋 Summary"
echo "===================================================================="
echo ""
echo "✅ Cleanup complete!"
echo ""
echo "📝 Next steps:"
echo ""
echo "1. If the HelmRelease still exists in git, Flux will recreate it"
echo "   To check: kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} -w"
echo ""
echo "2. If you want to prevent recreation, remove the HelmRelease from git first"
echo "   Location: gitops/apps/base/monitoring/helmrelease.yaml"
echo ""
echo "3. If you want to keep it but it was failing, the issue may be:"
echo "   - Resource constraints (not enough resources for Prometheus)"
echo "   - Storage issues (PVCs can't be deleted)"
echo "   - Dependency issues (resources depending on Prometheus)"
echo ""
echo "4. To reinstall fresh (if keeping it):"
echo "   - Remove from git temporarily"
echo "   - Let Flux delete it"
echo "   - Add it back to git"
echo "   - Let Flux reinstall"
echo ""
