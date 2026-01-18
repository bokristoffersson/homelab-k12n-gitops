#!/bin/bash
# Script to troubleshoot and fix stuck Helm release deletion
# Usage: ./scripts/troubleshoot-stuck-helm-release.sh <release-name> <namespace>

set -e

RELEASE_NAME="${1:-kube-prometheus-stack}"
NAMESPACE="${2:-monitoring}"

echo "🔍 Troubleshooting stuck Helm release: ${RELEASE_NAME} in namespace ${NAMESPACE}"
echo ""

# 1. Check HelmRelease status
echo "1️⃣ Checking HelmRelease status..."
kubectl get helmrelease ${RELEASE_NAME} -n ${NAMESPACE} || echo "⚠️  HelmRelease not found"

# 2. Check Helm release status
echo ""
echo "2️⃣ Checking Helm release status..."
helm list -n ${NAMESPACE} | grep ${RELEASE_NAME} || echo "⚠️  Helm release not found in Helm"

# 3. Check for stuck resources
echo ""
echo "3️⃣ Checking for stuck resources (StatefulSets, Deployments, PVCs)..."
kubectl get statefulsets -n ${NAMESPACE} 2>/dev/null || echo "No StatefulSets found"
kubectl get deployments -n ${NAMESPACE} 2>/dev/null || echo "No Deployments found"
kubectl get pvc -n ${NAMESPACE} 2>/dev/null || echo "No PVCs found"

# 4. Check for finalizers
echo ""
echo "4️⃣ Checking for resources with finalizers that might prevent deletion..."
kubectl get all -n ${NAMESPACE} -o json 2>/dev/null | grep -i finalizer || echo "No finalizers found"

# 5. Options
echo ""
echo "📋 Options to fix:"
echo ""
echo "Option A: Manually uninstall with Helm (longer timeout)"
echo "   helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --timeout 30m"
echo ""
echo "Option B: Delete HelmRelease and let Flux handle it"
echo "   kubectl delete helmrelease ${RELEASE_NAME} -n ${NAMESPACE}"
echo "   # Then reconcile: flux reconcile kustomization <kustomization-name> -n flux-system"
echo ""
echo "Option C: Patch HelmRelease to remove finalizer (if stuck)"
echo "   kubectl patch helmrelease ${RELEASE_NAME} -n ${NAMESPACE} -p '{\"metadata\":{\"finalizers\":[]}}' --type=merge"
echo ""
echo "Option D: Force delete resources (use with caution)"
echo "   kubectl delete all --all -n ${NAMESPACE} --force --grace-period=0"
echo "   helm uninstall ${RELEASE_NAME} -n ${NAMESPACE} --no-hooks"
echo ""
