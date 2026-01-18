#!/bin/bash
# Script to fix Flux reconciliation issues and force reconciliation

set -e

FLUX_NAMESPACE="flux-system"

echo "🔧 Fixing Flux reconciliation issues"
echo "===================================="
echo ""

# Step 1: Check GitRepository sync
echo "1️⃣ Checking GitRepository sync..."
echo "-----------------------------------"
GIT_REPO=$(kubectl get gitrepository -n ${FLUX_NAMESPACE} -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
if [ -n "${GIT_REPO}" ]; then
    echo "GitRepository: ${GIT_REPO}"
    kubectl get gitrepository ${GIT_REPO} -n ${FLUX_NAMESPACE}
    echo ""
    echo "Forcing GitRepository sync..."
    flux reconcile source git ${GIT_REPO} -n ${FLUX_NAMESPACE} 2>/dev/null || echo "⚠️  flux CLI not installed, skipping"
    sleep 5
else
    echo "⚠️  No GitRepository found"
fi
echo ""

# Step 2: Force reconciliation of all Kustomizations
echo "2️⃣ Forcing reconciliation of all Kustomizations..."
echo "--------------------------------------------------"
KUSTOMIZATIONS=$(kubectl get kustomizations -n ${FLUX_NAMESPACE} -o jsonpath='{.items[*].metadata.name}' 2>/dev/null || echo "")
if [ -n "${KUSTOMIZATIONS}" ]; then
    echo "Found Kustomizations:"
    for kust in ${KUSTOMIZATIONS}; do
        echo "  - ${kust}"
    done
    echo ""
    echo "Forcing reconciliation..."
    for kust in ${KUSTOMIZATIONS}; do
        echo "Reconciling ${kust}..."
        flux reconcile kustomization ${kust} -n ${FLUX_NAMESPACE} 2>/dev/null || \
        kubectl annotate kustomization ${kust} -n ${FLUX_NAMESPACE} reconcile.fluxcd.io/requestedAt="$(date +%s)" --overwrite 2>/dev/null || echo "⚠️  Failed to reconcile ${kust}"
        sleep 2
    done
else
    echo "⚠️  No Kustomizations found"
fi
echo ""

# Step 3: Check status after reconciliation
echo "3️⃣ Checking Kustomizations status..."
echo "-------------------------------------"
kubectl get kustomizations -n ${FLUX_NAMESPACE}
echo ""

# Step 4: Check for failed Kustomizations
echo "4️⃣ Checking for failed Kustomizations..."
echo "-----------------------------------------"
for kust in $(kubectl get kustomizations -n ${FLUX_NAMESPACE} -o jsonpath='{.items[*].metadata.name}'); do
    STATUS=$(kubectl get kustomization ${kust} -n ${FLUX_NAMESPACE} -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || echo "Unknown")
    if [ "${STATUS}" != "True" ]; then
        echo "⚠️  ${kust} is not ready (Status: ${STATUS})"
        MESSAGE=$(kubectl get kustomization ${kust} -n ${FLUX_NAMESPACE} -o jsonpath='{.status.conditions[?(@.type=="Ready")].message}' 2>/dev/null || echo "")
        echo "   Message: ${MESSAGE}"
    fi
done
echo ""

# Summary
echo "===================================================================="
echo "📋 Summary"
echo "===================================================================="
echo ""
echo "✅ Reconciliation triggered for all Kustomizations"
echo ""
echo "Next steps:"
echo "1. Monitor Kustomizations status:"
echo "   kubectl get kustomizations -n ${FLUX_NAMESPACE} -w"
echo ""
echo "2. Check specific Kustomization:"
echo "   kubectl get kustomization <name> -n ${FLUX_NAMESPACE} -o yaml"
echo ""
echo "3. Check for errors:"
echo "   kubectl logs -n ${FLUX_NAMESPACE} -l app=kustomize-controller --tail=100"
echo ""
echo "4. If Kustomizations are still failing, run the investigation script:"
echo "   ./scripts/investigate-flux-reconciliation.sh"
echo ""
