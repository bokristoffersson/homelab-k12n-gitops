#!/bin/bash
# Check Kubernetes iptables chains for port 10250

echo "=== Checking Kubernetes iptables chains ==="
echo ""

echo "1. Checking KUBE-FIREWALL chain:"
sudo iptables -L KUBE-FIREWALL -n -v
echo ""

echo "2. Checking KUBE-ROUTER-INPUT chain:"
sudo iptables -L KUBE-ROUTER-INPUT -n -v | head -30
echo ""

echo "3. Searching for port 10250 in all chains:"
sudo iptables -L -n -v | grep 10250
echo ""

echo "4. Checking if there are any DROP rules:"
sudo iptables -L -n -v | grep DROP | head -10
echo ""

echo "5. Checking KUBE-SERVICES chain (might have kubelet rules):"
sudo iptables -L KUBE-SERVICES -n -v | grep 10250
echo ""

echo "=== Solution ==="
echo "If port 10250 is being blocked, you may need to:"
echo "1. Add an explicit ACCEPT rule before the KUBE chains"
echo "2. Or check if kube-router network policies are blocking it"
echo ""
echo "Try: sudo iptables -I INPUT 1 -p tcp --dport 10250 -j ACCEPT"

