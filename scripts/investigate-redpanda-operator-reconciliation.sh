#!/bin/bash
# Script to investigate why redpanda-operator HelmRelease is failing Flux reconciliation

set -e

NAMESPACE="redpanda-system"
HELMRELEASE_NAME="redpanda-operator"
HELMREPOSITORY_NAME="redpanda-operator"
HELMREPOSITORY_NAMESPACE="flux-system"

echo "🔍 Investigating redpanda-operator HelmRelease reconciliation failure"
echo "=================================================================="
echo ""

# 1. Check HelmRelease status
echo "1️⃣ Checking HelmRelease status..."
echo "-----------------------------------"
kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} || echo "❌ HelmRelease not found"
echo ""

# 2. Get detailed HelmRelease information
echo "2️⃣ Detailed HelmRelease status..."
echo "-----------------------------------"
kubectl describe helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} 2>/dev/null || echo "❌ HelmRelease not found"
echo ""

# 3. Check HelmRelease conditions
echo "3️⃣ HelmRelease conditions (checking for errors)..."
echo "--------------------------------------------------"
kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} -o jsonpath='{.status.conditions[*]}' 2>/dev/null | jq -r '.' 2>/dev/null || \
kubectl get helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE} -o yaml 2>/dev/null | grep -A 10 "conditions:" || \
echo "⚠️  No conditions found or HelmRelease doesn't exist"
echo ""

# 4. Check HelmRepository status
echo "4️⃣ Checking HelmRepository status..."
echo "------------------------------------"
kubectl get helmrepository ${HELMREPOSITORY_NAME} -n ${HELMREPOSITORY_NAMESPACE} || echo "❌ HelmRepository not found"
echo ""

# 5. Check HelmRepository conditions
echo "5️⃣ HelmRepository conditions..."
echo "-------------------------------"
kubectl describe helmrepository ${HELMREPOSITORY_NAME} -n ${HELMREPOSITORY_NAMESPACE} 2>/dev/null | grep -A 20 "Conditions:" || echo "⚠️  No conditions found"
echo ""

# 6. Check if namespace exists
echo "6️⃣ Checking if namespace exists..."
echo "----------------------------------"
kubectl get namespace ${NAMESPACE} || echo "❌ Namespace not found"
echo ""

# 7. Check Helm release status (if installed)
echo "7️⃣ Checking Helm release status (if installed)..."
echo "-------------------------------------------------"
helm list -n ${NAMESPACE} 2>/dev/null | grep ${HELMRELEASE_NAME} || echo "⚠️  Helm release not installed"
echo ""

# 8. Check for pods
echo "8️⃣ Checking for operator pods..."
echo "--------------------------------"
kubectl get pods -n ${NAMESPACE} 2>/dev/null || echo "⚠️  No pods found"
echo ""

# 9. Check helm-controller logs for errors
echo "9️⃣ Checking helm-controller logs for redpanda-operator errors (last 50 lines)..."
echo "--------------------------------------------------------------------------------"
kubectl logs -n flux-system -l app=helm-controller --tail=200 2>/dev/null | grep -i "${HELMRELEASE_NAME}" | tail -20 || echo "⚠️  No logs found for ${HELMRELEASE_NAME}"
echo ""

# 10. Check recent helm-controller errors
echo "🔟 Checking recent helm-controller errors..."
echo "-------------------------------------------"
kubectl logs -n flux-system -l app=helm-controller --tail=100 2>/dev/null | grep -i "error\|failed\|reconciler error" | tail -10 || echo "⚠️  No recent errors found"
echo ""

# 11. Verify chart version exists
echo "1️⃣1️⃣ Verifying chart version 25.3.1 exists..."
echo "---------------------------------------------"
helm repo add redpanda-temp https://charts.redpanda.com/ 2>/dev/null || true
helm repo update redpanda-temp 2>/dev/null || true
if helm search repo redpanda-temp/operator --version 25.3.1 --versions 2>/dev/null | grep -q "25.3.1"; then
    echo "✅ Chart version 25.3.1 exists"
else
    echo "❌ Chart version 25.3.1 NOT found"
fi
helm repo remove redpanda-temp 2>/dev/null || true
echo ""

# 12. Check HelmRelease configuration in git
echo "1️⃣2️⃣ Checking HelmRelease configuration..."
echo "------------------------------------------"
echo "Chart version: 25.3.1"
echo "HelmRepository: ${HELMREPOSITORY_NAME} in ${HELMREPOSITORY_NAMESPACE}"
echo "Values check:"
echo "  - monitoring.enabled: false (should be set)"
echo "  - enableHelmControllers: should NOT be present"
echo "  - monitoring.deployPrometheusKubeStack: should NOT be present"
echo ""

# Summary
echo "=================================================================="
echo "📋 Summary & Next Steps"
echo "=================================================================="
echo ""
echo "Common issues to check:"
echo "1. HelmRepository not ready (check step 4 & 5)"
echo "2. Chart version not found (check step 11)"
echo "3. Values validation errors (check step 2 - describe output)"
echo "4. Namespace doesn't exist (check step 6)"
echo "5. CRD issues (HelmRelease has crds: Skip)"
echo ""
echo "To force reconciliation:"
echo "  flux reconcile helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
echo ""
echo "To check HelmRepository reconciliation:"
echo "  flux reconcile source helm ${HELMREPOSITORY_NAME} -n ${HELMREPOSITORY_NAMESPACE}"
echo ""
echo "To view detailed error messages:"
echo "  kubectl describe helmrelease ${HELMRELEASE_NAME} -n ${NAMESPACE}"
echo "  kubectl logs -n flux-system -l app=helm-controller --tail=200 | grep ${HELMRELEASE_NAME}"
echo ""
