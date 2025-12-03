#!/bin/bash
# Restart K3s agent and verify re-registration

echo "=== Restarting K3s Agent ==="
echo ""

# Start the agent
echo "Starting k3s-agent service..."
sudo systemctl start k3s-agent

# Wait a moment
sleep 3

# Check status
echo ""
echo "Service status:"
sudo systemctl status k3s-agent --no-pager -l | head -20

echo ""
echo "=== Waiting for node to re-register (this may take 1-2 minutes) ==="
echo ""

# Wait and check node status
for i in {1..12}; do
    echo "Checking node status... ($i/12)"
    if kubectl get node p0 &>/dev/null; then
        STATUS=$(kubectl get node p0 -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null)
        if [ "$STATUS" = "True" ]; then
            echo "✓ Node p0 is Ready!"
            break
        else
            echo "  Node found but not Ready yet (status: $STATUS)"
        fi
    else
        echo "  Node not found yet..."
    fi
    sleep 10
done

echo ""
echo "=== Final Status ==="
kubectl get nodes

echo ""
echo "=== Test Proxy ==="
echo "Run this command to test:"
echo "  kubectl get --raw /api/v1/nodes/p0/proxy/stats/summary | head -20"

