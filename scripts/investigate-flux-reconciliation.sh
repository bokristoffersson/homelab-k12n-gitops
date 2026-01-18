#!/bin/bash
# Script to investigate why Flux is not reconciling everything

set -e

FLUX_NAMESPACE="flux-system"

echo "🔍 Investigating Flux reconciliation issues"
echo "==========================================="
echo ""

# 1. Check Flux health
echo "1️⃣ Checking Flux health..."
echo "---------------------------"
flux check 2>/dev/null || echo "⚠️  flux CLI not installed, skipping"
echo ""

# 2. Check GitRepository status
echo "2️⃣ Checking GitRepository status..."
echo "------------------------------------"
kubectl get gitrepository -n ${FLUX_NAMESPACE} || echo "⚠️  No GitRepositories found"
echo ""

# 3. Check GitRepository details
echo "3️⃣ GitRepository detailed status..."
echo "-----------------------------------"
GIT_REPO=$(kubectl get gitrepository -n ${FLUX_NAMESPACE} -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
if [ -n "${GIT_REPO}" ]; then
    echo "GitRepository: ${GIT_REPO}"
    kubectl get gitrepository ${GIT_REPO} -n ${FLUX_NAMESPACE} -o yaml | grep -A 30 "status:" || echo "No status found"
else
    echo "⚠️  No GitRepository found"
fi
echo ""

# 4. Check Kustomizations status
echo "4️⃣ Checking Kustomizations status..."
echo "-------------------------------------"
kubectl get kustomizations -n ${FLUX_NAMESPACE} || echo "⚠️  No Kustomizations found"
echo ""

# 5. Check Kustomizations details
echo "5️⃣ Kustomizations detailed status..."
echo "------------------------------------"
kubectl get kustomizations -n ${FLUX_NAMESPACE} -o wide
echo ""

# 6. Check for failed or stuck Kustomizations
echo "6️⃣ Checking for failed/stuck Kustomizations..."
echo "-----------------------------------------------"
kubectl get kustomizations -n ${FLUX_NAMESPACE} -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.conditions[?(@.type=="Ready")].status}{"\t"}{.status.conditions[?(@.type=="Ready")].message}{"\n"}{end}' 2>/dev/null || echo "No status found"
echo ""

# 7. Check infrastructure Kustomization
echo "7️⃣ Checking infrastructure Kustomization..."
echo "--------------------------------------------"
if kubectl get kustomization infrastructure -n ${FLUX_NAMESPACE} 2>/dev/null; then
    echo "Status:"
    kubectl get kustomization infrastructure -n ${FLUX_NAMESPACE} -o yaml | grep -A 30 "status:" || echo "No status found"
    echo ""
    echo "Conditions:"
    kubectl get kustomization infrastructure -n ${FLUX_NAMESPACE} -o jsonpath='{.status.conditions[*]}' 2>/dev/null | jq -r '.' 2>/dev/null || \
    kubectl get kustomization infrastructure -n ${FLUX_NAMESPACE} -o yaml | grep -A 20 "conditions:" || echo "No conditions found"
else
    echo "⚠️  infrastructure Kustomization not found"
fi
echo ""

# 8. Check apps Kustomization
echo "8️⃣ Checking apps Kustomization..."
echo "----------------------------------"
if kubectl get kustomization apps -n ${FLUX_NAMESPACE} 2>/dev/null; then
    echo "Status:"
    kubectl get kustomization apps -n ${FLUX_NAMESPACE} -o yaml | grep -A 30 "status:" || echo "No status found"
    echo ""
    echo "Conditions:"
    kubectl get kustomization apps -n ${FLUX_NAMESPACE} -o jsonpath='{.status.conditions[*]}' 2>/dev/null | jq -r '.' 2>/dev/null || \
    kubectl get kustomization apps -n ${FLUX_NAMESPACE} -o yaml | grep -A 20 "conditions:" || echo "No conditions found"
else
    echo "⚠️  apps Kustomization not found"
fi
echo ""

# 9. Check all Kustomizations for errors
echo "9️⃣ Checking all Kustomizations for errors..."
echo "---------------------------------------------"
for kust in $(kubectl get kustomizations -n ${FLUX_NAMESPACE} -o jsonpath='{.items[*].metadata.name}'); do
    echo "Kustomization: ${kust}"
    STATUS=$(kubectl get kustomization ${kust} -n ${FLUX_NAMESPACE} -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || echo "Unknown")
    MESSAGE=$(kubectl get kustomization ${kust} -n ${FLUX_NAMESPACE} -o jsonpath='{.status.conditions[?(@.type=="Ready")].message}' 2>/dev/null || echo "No message")
    echo "  Status: ${STATUS}"
    echo "  Message: ${MESSAGE}"
    if [ "${STATUS}" != "True" ]; then
        echo "  ⚠️  This Kustomization is NOT ready!"
    fi
    echo ""
done

# 10. Check for stalled resources
echo "🔟 Checking for stalled resources..."
echo "------------------------------------"
for kust in $(kubectl get kustomizations -n ${FLUX_NAMESPACE} -o jsonpath='{.items[*].metadata.name}'); do
    STALLED=$(kubectl get kustomization ${kust} -n ${FLUX_NAMESPACE} -o jsonpath='{.status.inventory.stalled[*].name}' 2>/dev/null || echo "")
    if [ -n "${STALLED}" ]; then
        echo "Kustomization: ${kust} has stalled resources:"
        echo "${STALLED}"
        echo ""
    fi
done

# 11. Check kustomize-controller logs
echo "1️⃣1️⃣ Checking kustomize-controller logs for errors (last 50 lines)..."
echo "-------------------------------------------------------------------"
kubectl logs -n ${FLUX_NAMESPACE} -l app=kustomize-controller --tail=100 2>/dev/null | grep -i "error\|failed\|stalled" | tail -20 || echo "No errors found in recent logs"
echo ""

# 12. Check source-controller logs
echo "1️⃣2️⃣ Checking source-controller logs for errors (last 50 lines)..."
echo "-----------------------------------------------------------------"
kubectl logs -n ${FLUX_NAMESPACE} -l app=source-controller --tail=100 2>/dev/null | grep -i "error\|failed" | tail -20 || echo "No errors found in recent logs"
echo ""

# Summary and recommendations
echo "===================================================================="
echo "📋 Summary & Recommendations"
echo "===================================================================="
echo ""
echo "Common issues to check:"
echo "1. GitRepository not syncing (check step 2 & 3)"
echo "2. Kustomizations stuck or failed (check step 4-9)"
echo "3. Stalled resources (check step 10)"
echo "4. Controller errors (check step 11 & 12)"
echo ""
echo "To force reconciliation:"
echo ""
echo "# Force GitRepository sync:"
echo "flux reconcile source git <git-repo-name> -n ${FLUX_NAMESPACE}"
echo ""
echo "# Force Kustomization reconciliation:"
echo "flux reconcile kustomization infrastructure -n ${FLUX_NAMESPACE}"
echo "flux reconcile kustomization apps -n ${FLUX_NAMESPACE}"
echo ""
echo "# Force all Kustomizations:"
echo "for kust in \$(kubectl get kustomizations -n ${FLUX_NAMESPACE} -o jsonpath='{.items[*].metadata.name}'); do"
echo "  flux reconcile kustomization \$kust -n ${FLUX_NAMESPACE}"
echo "done"
echo ""
echo "# Check Flux status:"
echo "flux get all -n ${FLUX_NAMESPACE}"
echo ""
