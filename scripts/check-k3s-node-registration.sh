#!/bin/bash
# Check K3s node registration and proxy configuration

echo "=== K3s Node Registration Check ==="
echo ""

echo "1. Node status and addresses:"
kubectl get nodes -o wide
echo ""

echo "2. Detailed node info for p0:"
kubectl get node p0 -o yaml | grep -A 20 "status:" | head -30
echo ""

echo "3. Check if there's a provider ID mismatch:"
kubectl get node p0 -o jsonpath='{.spec.providerID}' && echo ""
kubectl get node p1 -o jsonpath='{.spec.providerID}' && echo ""
echo ""

echo "4. Check node annotations:"
kubectl get node p0 -o jsonpath='{.metadata.annotations.k3s\.io/internal-ip}' && echo ""
kubectl get node p0 -o jsonpath='{.metadata.annotations.flannel\.alpha\.coreos\.com/public-ip}' && echo ""
echo ""

echo "=== Tests ==="
echo ""
echo "5. Test API server proxy with verbose output:"
echo "   kubectl get --raw /api/v1/nodes/p0/proxy/stats/summary -v=8 2>&1 | grep -i 'p0\|proxy\|502\|error'"
echo ""
echo "6. Check K3s server logs on p1:"
echo "   sudo journalctl -u k3s -n 500 --no-pager | grep -i 'p0\|192.168.50.210\|proxy\|kubelet\|502' | tail -30"
echo ""
echo "7. Check if API server can resolve worker node IP:"
echo "   kubectl get node p0 -o jsonpath='{.status.addresses}' | python3 -m json.tool"
echo ""

echo "=== Potential Fix: Re-register Worker Node ==="
echo ""
echo "If nothing else works, try re-registering the worker node:"
echo ""
echo "On p0 (worker node):"
echo "   sudo systemctl stop k3s-agent"
echo "   sudo rm -rf /var/lib/rancher/k3s/agent"
echo "   sudo systemctl start k3s-agent"
echo ""
echo "Wait a few minutes, then verify:"
echo "   kubectl get nodes"
echo "   kubectl get --raw /api/v1/nodes/p0/proxy/stats/summary"

