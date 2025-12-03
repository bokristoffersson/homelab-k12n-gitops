#!/bin/bash
# Script to run on worker node (p0) to check kubelet configuration

echo "=== Kubelet Configuration Check ==="
echo ""

# Check if kubelet is listening
echo "1. Checking if kubelet is listening on port 10250..."
if command -v ss &> /dev/null; then
    sudo ss -tlnp | grep 10250 || echo "   ✗ Kubelet not listening on port 10250"
else
    sudo netstat -tlnp | grep 10250 || echo "   ✗ Kubelet not listening on port 10250"
fi
echo ""

# Check k3s-agent service status
echo "2. Checking k3s-agent service status..."
sudo systemctl status k3s-agent --no-pager -l | head -20
echo ""

# Check k3s-agent logs for errors
echo "3. Recent k3s-agent logs (last 30 lines)..."
sudo journalctl -u k3s-agent -n 30 --no-pager | tail -30
echo ""

# Check iptables rules (even if ufw is inactive)
echo "4. Checking iptables rules for port 10250..."
sudo iptables -L -n | grep 10250 || echo "   No specific iptables rules for 10250"
echo ""

# Check k3s config
echo "5. Checking K3s configuration..."
if [ -f /etc/rancher/k3s/config.yaml ]; then
    echo "   Config file exists:"
    sudo cat /etc/rancher/k3s/config.yaml
else
    echo "   No custom config file found (using defaults)"
fi
echo ""

# Check network interfaces
echo "6. Network interfaces..."
ip addr show | grep -E "^[0-9]+:|inet " | head -10
echo ""

# Check if kubelet is bound to correct interface
echo "7. Testing if kubelet responds locally..."
curl -k https://localhost:10250/healthz 2>&1 | head -5 || echo "   ✗ Cannot reach kubelet locally"
echo ""

echo "=== Next Steps ==="
echo "If kubelet is not listening on 0.0.0.0:10250, you may need to:"
echo "1. Check K3s agent configuration"
echo "2. Restart k3s-agent: sudo systemctl restart k3s-agent"
echo "3. Check if there are any network policies blocking traffic"

