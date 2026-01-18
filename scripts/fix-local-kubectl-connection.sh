#!/bin/bash
# Script to fix kubectl connection from local machine when it works on control plane

set -e

echo "🔧 Fixing kubectl connection from local machine"
echo "================================================"
echo ""
echo "Since kubectl works on the control plane node, K3s is running fine."
echo "The issue is likely with kubeconfig on your local machine or network connectivity."
echo ""

# Step 1: Get control plane connection info
echo "1️⃣ Getting control plane connection information..."
echo "--------------------------------------------------"
read -p "Enter control plane node hostname or IP (e.g., p0.local or 192.168.x.x): " CONTROL_PLANE
read -p "Enter SSH user (default: pi): " SSH_USER
SSH_USER=${SSH_USER:-pi}

echo ""
echo "Control plane: ${SSH_USER}@${CONTROL_PLANE}"
echo ""

# Step 2: Test connectivity
echo "2️⃣ Testing connectivity to control plane..."
echo "--------------------------------------------"
if ping -c 1 ${CONTROL_PLANE} >/dev/null 2>&1; then
    echo "✅ Can ping ${CONTROL_PLANE}"
else
    echo "❌ Cannot ping ${CONTROL_PLANE}"
    echo "Check network connectivity or DNS resolution"
    exit 1
fi

# Test port 6443
echo ""
echo "Testing port 6443 connectivity..."
if timeout 5 bash -c "echo >/dev/tcp/${CONTROL_PLANE}/6443" 2>/dev/null || nc -zv ${CONTROL_PLANE} 6443 2>/dev/null; then
    echo "✅ Port 6443 is reachable on ${CONTROL_PLANE}"
else
    echo "❌ Cannot reach port 6443 on ${CONTROL_PLANE}"
    echo ""
    echo "⚠️  Possible issues:"
    echo "  1. Firewall blocking port 6443"
    echo "  2. K3s API server not bound to external interface"
    echo "  3. Network routing issue"
    echo ""
    echo "Check K3s binding on control plane:"
    echo "  ssh ${SSH_USER}@${CONTROL_PLANE} 'sudo ss -tlnp | grep 6443'"
    echo ""
    read -p "Continue anyway? (y/n): " CONTINUE
    if [ "${CONTINUE}" != "y" ]; then
        exit 1
    fi
fi
echo ""

# Step 3: Backup current kubeconfig
echo "3️⃣ Backing up current kubeconfig..."
echo "-------------------------------------"
if [ -f ~/.kube/config ]; then
    BACKUP_FILE=~/.kube/config.backup.$(date +%Y%m%d_%H%M%S)
    cp ~/.kube/config "${BACKUP_FILE}"
    echo "✅ Backed up to: ${BACKUP_FILE}"
else
    echo "ℹ️  No existing kubeconfig to backup"
fi
echo ""

# Step 4: Get fresh kubeconfig from control plane
echo "4️⃣ Getting fresh kubeconfig from control plane..."
echo "--------------------------------------------------"
echo "SSH-ing to ${SSH_USER}@${CONTROL_PLANE} to get kubeconfig..."

# Ensure .kube directory exists
mkdir -p ~/.kube

# Get kubeconfig from control plane
if ssh ${SSH_USER}@${CONTROL_PLANE} "sudo k3s kubectl config view --raw" > ~/.kube/config.new 2>/dev/null; then
    echo "✅ Retrieved kubeconfig from control plane"
    
    # Check what server URL is in the config
    SERVER_URL=$(grep "server:" ~/.kube/config.new | head -1 | awk '{print $2}' || echo "")
    echo ""
    echo "Server URL in kubeconfig: ${SERVER_URL}"
    echo ""
    
    # Check if server URL needs to be updated
    if [[ "${SERVER_URL}" == *"127.0.0.1"* ]] || [[ "${SERVER_URL}" == *"localhost"* ]]; then
        echo "⚠️  Server URL is localhost/127.0.0.1, need to update to ${CONTROL_PLANE}"
        echo ""
        
        # Update server URL
        if [[ "${CONTROL_PLANE}" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            # It's an IP address
            NEW_URL="https://${CONTROL_PLANE}:6443"
        else
            # It's a hostname
            CONTROL_IP=$(getent hosts ${CONTROL_PLANE} 2>/dev/null | awk '{print $1}' | head -1 || echo "")
            if [ -n "${CONTROL_IP}" ]; then
                echo "Resolved ${CONTROL_PLANE} to ${CONTROL_IP}"
                NEW_URL="https://${CONTROL_IP}:6443"
            else
                NEW_URL="https://${CONTROL_PLANE}:6443"
            fi
        fi
        
        echo "Updating server URL to: ${NEW_URL}"
        sed "s|server:.*|server: ${NEW_URL}|" ~/.kube/config.new > ~/.kube/config.new2
        mv ~/.kube/config.new2 ~/.kube/config.new
    fi
    
    # Set permissions
    chmod 600 ~/.kube/config.new
    
    # Move to final location
    mv ~/.kube/config.new ~/.kube/config
    echo ""
    echo "✅ kubeconfig saved to ~/.kube/config"
else
    echo "❌ Failed to get kubeconfig from control plane"
    echo ""
    echo "Try manually:"
    echo "  ssh ${SSH_USER}@${CONTROL_PLANE}"
    echo "  sudo k3s kubectl config view --raw > ~/k3s-config"
    echo "  chmod 600 ~/k3s-config"
    echo "  # Then copy it to your local machine"
    exit 1
fi
echo ""

# Step 5: Test connection
echo "5️⃣ Testing kubectl connection..."
echo "----------------------------------"
echo "Current context:"
kubectl config current-context 2>/dev/null || echo "⚠️  No current context"
echo ""

echo "Testing cluster connection..."
if kubectl cluster-info >/dev/null 2>&1; then
    echo "✅ kubectl connection successful!"
    echo ""
    kubectl cluster-info
    echo ""
    echo "Testing nodes..."
    kubectl get nodes
else
    echo "❌ kubectl connection failed"
    echo ""
    echo "Error details:"
    kubectl cluster-info 2>&1 || true
    echo ""
    echo "Troubleshooting steps:"
    echo "1. Check if K3s is bound to external interface:"
    echo "   ssh ${SSH_USER}@${CONTROL_PLANE} 'sudo ss -tlnp | grep 6443'"
    echo ""
    echo "2. Check firewall rules on control plane:"
    echo "   ssh ${SSH_USER}@${CONTROL_PLANE} 'sudo iptables -L -n | grep 6443'"
    echo ""
    echo "3. Try using IP address instead of hostname:"
    echo "   kubectl config set-cluster default --server=https://<control-plane-ip>:6443"
    exit 1
fi
echo ""

# Summary
echo "===================================================================="
echo "📋 Summary"
echo "===================================================================="
echo ""
echo "✅ kubeconfig updated successfully!"
echo ""
echo "If connection still fails, check:"
echo ""
echo "1. K3s API server binding:"
echo "   ssh ${SSH_USER}@${CONTROL_PLANE} 'sudo ss -tlnp | grep 6443'"
echo "   Should show: 0.0.0.0:6443 or <external-ip>:6443"
echo "   If it shows 127.0.0.1:6443, K3s is only listening on localhost"
echo ""
echo "2. Firewall on control plane:"
echo "   ssh ${SSH_USER}@${CONTROL_PLANE} 'sudo ufw status'"
echo "   Or: ssh ${SSH_USER}@${CONTROL_PLANE} 'sudo iptables -L -n | grep 6443'"
echo ""
echo "3. Network connectivity:"
echo "   ping ${CONTROL_PLANE}"
echo "   nc -zv ${CONTROL_PLANE} 6443"
echo ""
echo "4. If K3s is only listening on localhost, you may need to:"
echo "   - Use SSH tunnel: ssh -L 6443:localhost:6443 ${SSH_USER}@${CONTROL_PLANE}"
echo "   - Or reconfigure K3s to bind to external interface"
echo ""
