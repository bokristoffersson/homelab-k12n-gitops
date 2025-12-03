#!/bin/bash
# Diagnostic script for K3s node connectivity issues

set -e

echo "=== K3s Node Connectivity Diagnostics ==="
echo ""

# Check node status
echo "1. Checking node status..."
kubectl get nodes -o wide
echo ""

# Check if we can reach the kubelet API directly
echo "2. Testing kubelet API connectivity..."
NODE_IP=$(kubectl get node p0 -o jsonpath='{.status.addresses[?(@.type=="InternalIP")].address}')
CONTROL_PLANE_IP=$(kubectl get node p1 -o jsonpath='{.status.addresses[?(@.type=="InternalIP")].address}')

echo "Worker node (p0) IP: $NODE_IP"
echo "Control plane (p1) IP: $CONTROL_PLANE_IP"
echo ""

# Test basic connectivity
echo "3. Testing network connectivity from control plane to worker..."
echo "   (Run this on p1: ping -c 3 $NODE_IP)"
echo ""

# Check kubelet proxy
echo "4. Testing kubelet proxy access..."
if kubectl get --raw /api/v1/nodes/p0/proxy/stats/summary > /dev/null 2>&1; then
    echo "   ✓ Kubelet proxy works"
else
    echo "   ✗ Kubelet proxy failed (this is the issue)"
fi
echo ""

# Check firewall status
echo "5. Firewall check instructions:"
echo "   On p0 (worker node), run:"
echo "   sudo ufw status"
echo "   sudo iptables -L -n | grep 10250"
echo ""

# Check kubelet service
echo "6. Kubelet service check instructions:"
echo "   On p0 (worker node), run:"
echo "   sudo systemctl status k3s-agent"
echo "   sudo journalctl -u k3s-agent -n 50 --no-pager"
echo ""

echo "=== Common Fixes ==="
echo ""
echo "1. Ensure port 10250 is open on worker node (p0):"
echo "   sudo ufw allow 10250/tcp"
echo "   sudo ufw reload"
echo ""
echo "2. Check kubelet is listening on correct interface:"
echo "   sudo netstat -tlnp | grep 10250"
echo "   or"
echo "   sudo ss -tlnp | grep 10250"
echo ""
echo "3. Restart k3s-agent on worker node if needed:"
echo "   sudo systemctl restart k3s-agent"
echo ""

