#!/bin/bash
# Script to fix failed redpanda-operator HelmRelease that's blocking Kustomization health check

set -e

HELMRELEASE_NAME="redpanda-operator"
NAMESPACE="redpanda-system"
KUSTOMIZATION_NAME="infrastructure-controllers"
KUSTOMIZATION_NAMESPACE="flux-system"

echo "🔧 Fixing failed redpanda-operator HelmRelease"
echo "==============================================="
echo ""
echo "The Kustomization '${KUSTOMIZATION_NAME}' is failing because"
echo "HelmRelease '${HELMRELEASE_NAME}' has status 'Failed'"
echo ""

# Step 1: Check HelmRelease status
echo "1️⃣ Checking HelmRelease status..."
echo "----------------------------------"
kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} || {
    echo "❌ HelmRelease not found"
    exit 1
}
echo ""

# Step 2: Get detailed status and error
echo "2️⃣ Getting detailed error information..."
echo "-----------------------------------------"
echo "HelmRelease conditions:"
kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} -o yaml | grep -A 30 "conditions:" || echo "No conditions found"
echo ""

# Step 3: Check for recent errors in logs
echo "3️⃣ Checking helm-controller logs for errors..."
echo "-----------------------------------------------"
kubectl logs -n flux-system -l app=helm-controller --tail=100 2>/dev/null | grep -A 5 -B 5 "${HELMRELEASE_NAME}" | tail -20 || echo "No logs found"
echo ""

# Step 4: Check HelmRepository status
echo "4️⃣ Checking HelmRepository status..."
echo "-------------------------------------"
kubectl get helmrepository redpanda-operator -n flux-system || echo "⚠️  HelmRepository not found"
echo ""

# Step 5: Options to fix
echo "===================================================================="
echo "📋 Options to Fix"
echo "===================================================================="
echo ""
echo "Option 1: Suspend and Resume (try first)"
echo "-----------------------------------------"
echo "This will reset the HelmRelease state and trigger reconciliation:"
echo ""
echo "  flux suspend helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
echo "  sleep 10"
echo "  flux resume helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
echo ""

echo "Option 2: Delete and Recreate (if suspend/resume doesn't work)"
echo "---------------------------------------------------------------"
echo "This will delete the HelmRelease and let Flux recreate it from git:"
echo ""
echo "  kubectl delete helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} --wait=false"
echo "  # Wait for Flux to recreate it (or force reconcile)"
echo "  flux reconcile kustomization ${KUSTOMIZATION_NAME} -n ${KUSTOMIZATION_NAMESPACE}"
echo ""

echo "Option 3: Force Reconciliation"
echo "-------------------------------"
echo "Force the HelmRelease to reconcile:"
echo ""
echo "  flux reconcile helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
echo ""

echo "Option 4: Check if values are incorrect (if still failing after recreate)"
echo "--------------------------------------------------------------------------"
echo "The HelmRelease might be using old values. Check:"
echo ""
echo "  kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} -o yaml | grep -A 50 'values:'"
echo ""
echo "Make sure values match the fixed configuration in git"
echo ""

# Step 6: Recommended fix
echo "===================================================================="
echo "🎯 Recommended Fix (Run these commands)"
echo "===================================================================="
echo ""
echo "# Step 1: Suspend to stop current reconciliation"
echo "flux suspend helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
echo ""
echo "# Step 2: Wait a moment"
echo "sleep 10"
echo ""
echo "# Step 3: Resume to trigger fresh reconciliation"
echo "flux resume helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
echo ""
echo "# Step 4: Watch status"
echo "kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} -w"
echo ""
echo "# If that doesn't work, try delete and recreate:"
echo "kubectl delete helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} --wait=false"
echo "flux reconcile kustomization ${KUSTOMIZATION_NAME} -n ${KUSTOMIZATION_NAMESPACE}"
echo ""
