#!/bin/bash
# Fix API server proxy issue for kubelet

echo "=== Fixing API Server Proxy Issue ==="
echo ""

# Get node IPs
P0_IP=$(kubectl get node p0 -o jsonpath='{.status.addresses[?(@.type=="InternalIP")].address}' 2>/dev/null || echo "192.168.50.210")
P1_IP=$(kubectl get node p1 -o jsonpath='{.status.addresses[?(@.type=="InternalIP")].address}' 2>/dev/null || echo "192.168.50.211")

echo "Worker node (p0) IP: $P0_IP"
echo "Control plane (p1) IP: $P1_IP"
echo ""

echo "=== On WORKER NODE (p0) ==="
echo ""
echo "1. Verify iptables rule is still there:"
echo "   sudo iptables -L INPUT -n -v --line-numbers | grep 10250"
echo ""
echo "2. If rule is missing, add it again:"
echo "   sudo iptables -I INPUT 1 -p tcp --dport 10250 -j ACCEPT"
echo ""
echo "3. Check if rule is being processed:"
echo "   sudo iptables -L INPUT -n -v | head -5"
echo ""
echo "4. Try allowing from control plane specifically:"
echo "   sudo iptables -I INPUT 1 -p tcp -s $P1_IP --dport 10250 -j ACCEPT"
echo ""
echo "=== On CONTROL PLANE (p1) ==="
echo ""
echo "1. Check K3s server logs for detailed proxy errors:"
echo "   sudo journalctl -u k3s -n 200 --no-pager | grep -A 5 -B 5 'p0\|$P0_IP\|proxy\|502'"
echo ""
echo "2. Check if API server can resolve worker node:"
echo "   kubectl get node p0 -o yaml | grep -A 3 addresses"
echo ""
echo "3. Try restarting K3s server (if needed):"
echo "   sudo systemctl restart k3s"
echo "   sudo systemctl status k3s"
echo ""
echo "=== Alternative: Check K3s Configuration ==="
echo ""
echo "On p0, check if there's a kubelet-arg configuration issue:"
echo "   sudo cat /etc/rancher/k3s/config.yaml"
echo ""
echo "On p1, check API server configuration:"
echo "   sudo cat /etc/rancher/k3s/config.yaml"
echo "   ps aux | grep k3s | grep -v grep | head -1"

