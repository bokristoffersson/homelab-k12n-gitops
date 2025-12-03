#!/bin/bash
# Test connectivity between control plane and worker node

CONTROL_PLANE_IP="192.168.50.211"
WORKER_NODE_IP="192.168.50.210"

echo "=== Testing Network Connectivity ==="
echo ""

echo "1. Testing ping from control plane to worker node..."
echo "   (Run this on p1: ping -c 3 $WORKER_NODE_IP)"
echo ""

echo "2. Testing if control plane can reach worker kubelet..."
echo "   (Run this on p1: curl -k https://$WORKER_NODE_IP:10250/healthz)"
echo ""

echo "3. Checking if there are any iptables rules blocking..."
echo "   (Run this on p0: sudo iptables -L -n -v | grep 10250)"
echo ""

echo "4. Checking routing table..."
echo "   (Run this on p1: ip route get $WORKER_NODE_IP)"
echo ""

echo "5. Testing from control plane using kubectl proxy..."
echo "   kubectl proxy --port=8001 &"
echo "   curl http://localhost:8001/api/v1/nodes/p0/proxy/stats/summary"
echo ""

