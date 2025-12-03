#!/bin/bash
# Fix kubelet access by adding iptables rule

echo "=== Fixing Kubelet Access ==="
echo ""

# Get control plane IP
CONTROL_PLANE_IP=$(kubectl get node p1 -o jsonpath='{.status.addresses[?(@.type=="InternalIP")].address}' 2>/dev/null || echo "192.168.50.211")

echo "Control plane IP: $CONTROL_PLANE_IP"
echo ""

echo "Adding iptables rule to allow kubelet port 10250..."
echo ""

# Add rule to allow kubelet port from control plane
sudo iptables -I INPUT 1 -p tcp -s $CONTROL_PLANE_IP --dport 10250 -j ACCEPT

# Also allow from entire subnet (in case IP changes)
sudo iptables -I INPUT 2 -p tcp -s 192.168.50.0/24 --dport 10250 -j ACCEPT

echo "Rules added. Verifying..."
sudo iptables -L INPUT -n -v | head -5

echo ""
echo "=== Testing ==="
echo "Now test from your local machine:"
echo "  kubectl get --raw /api/v1/nodes/p0/proxy/stats/summary | head -20"
echo ""
echo "To make rules persistent, run:"
echo "  sudo netfilter-persistent save"
echo "  (or install iptables-persistent first if needed)"

