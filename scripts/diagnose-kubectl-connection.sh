#!/bin/bash
# Script to diagnose kubectl connection issues to K3s cluster

set -e

echo "🔍 Diagnosing kubectl connection issues"
echo "========================================"
echo ""

# Step 1: Check kubectl configuration
echo "1️⃣ Checking kubectl configuration..."
echo "-------------------------------------"
echo "KUBECONFIG environment variable:"
echo "${KUBECONFIG:-~/.kube/config}"
echo ""

if [ -f "${KUBECONFIG:-~/.kube/config}" ]; then
    echo "✅ kubeconfig file exists"
    echo "Location: ${KUBECONFIG:-~/.kube/config}"
    ls -lh "${KUBECONFIG:-~/.kube/config}"
else
    echo "❌ kubeconfig file NOT found"
    echo "Expected location: ${KUBECONFIG:-~/.kube/config}"
fi
echo ""

# Step 2: Check kubectl can find config
echo "2️⃣ Testing kubectl config view..."
echo "----------------------------------"
if kubectl config view >/dev/null 2>&1; then
    echo "✅ kubectl can read config"
    echo ""
    echo "Current context:"
    kubectl config current-context 2>/dev/null || echo "⚠️  No current context"
    echo ""
    echo "Available contexts:"
    kubectl config get-contexts 2>/dev/null || echo "⚠️  No contexts found"
else
    echo "❌ kubectl cannot read config"
    echo "Error:"
    kubectl config view 2>&1 || true
fi
echo ""

# Step 3: Check API server endpoint
echo "3️⃣ Checking API server endpoint..."
echo "-----------------------------------"
if kubectl config view >/dev/null 2>&1; then
    SERVER=$(kubectl config view -o jsonpath='{.clusters[0].cluster.server}' 2>/dev/null || echo "")
    if [ -n "${SERVER}" ]; then
        echo "API Server: ${SERVER}"
        echo ""
        echo "Testing connectivity..."
        # Extract host and port
        HOST=$(echo "${SERVER}" | sed 's|https\?://||' | cut -d: -f1)
        PORT=$(echo "${SERVER}" | sed 's|https\?://||' | cut -d: -f2 | cut -d/ -f1)
        
        if [ -n "${HOST}" ] && [ -n "${PORT}" ]; then
            echo "Testing connection to ${HOST}:${PORT}..."
            if timeout 5 bash -c "echo >/dev/tcp/${HOST}/${PORT}" 2>/dev/null; then
                echo "✅ Port ${PORT} is reachable on ${HOST}"
            else
                echo "❌ Cannot connect to ${HOST}:${PORT}"
                echo "This could mean:"
                echo "  - API server is down"
                echo "  - Network connectivity issue"
                echo "  - Firewall blocking port"
                echo "  - DNS resolution issue"
            fi
        else
            echo "⚠️  Could not parse host/port from ${SERVER}"
        fi
    else
        echo "⚠️  Could not get API server endpoint"
    fi
else
    echo "⚠️  Cannot read kubectl config"
fi
echo ""

# Step 4: Check if cluster is reachable (actual kubectl command)
echo "4️⃣ Testing kubectl cluster connection..."
echo "-----------------------------------------"
if kubectl cluster-info >/dev/null 2>&1; then
    echo "✅ kubectl can reach cluster"
    kubectl cluster-info 2>/dev/null || true
else
    echo "❌ kubectl cannot reach cluster"
    echo ""
    echo "Error details:"
    kubectl cluster-info 2>&1 || true
    echo ""
fi
echo ""

# Step 5: Check certificate expiration
echo "5️⃣ Checking certificate expiration..."
echo "--------------------------------------"
if kubectl config view >/dev/null 2>&1; then
    CERT_FILE=$(kubectl config view --raw -o jsonpath='{.users[0].user.client-certificate-data}' 2>/dev/null | base64 -d 2>/dev/null || echo "")
    if [ -n "${CERT_FILE}" ]; then
        # Try to check cert expiration (if openssl available)
        if command -v openssl >/dev/null 2>&1; then
            echo "${CERT_FILE}" | openssl x509 -noout -enddate 2>/dev/null || echo "⚠️  Could not parse certificate"
        else
            echo "⚠️  openssl not available, cannot check certificate"
        fi
    else
        # Try to check from kubeconfig file
        CLIENT_CERT=$(kubectl config view -o jsonpath='{.users[0].user.client-certificate}' 2>/dev/null || echo "")
        if [ -n "${CLIENT_CERT}" ] && [ -f "${CLIENT_CERT}" ]; then
            if command -v openssl >/dev/null 2>&1; then
                echo "Certificate expiration:"
                openssl x509 -in "${CLIENT_CERT}" -noout -enddate 2>/dev/null || echo "⚠️  Could not read certificate"
            fi
        else
            echo "⚠️  Could not find certificate file"
        fi
    fi
else
    echo "⚠️  Cannot read kubectl config"
fi
echo ""

# Step 6: Check if it's a K3s cluster (based on README)
echo "6️⃣ Checking for K3s-specific issues..."
echo "----------------------------------------"
echo "If this is a K3s cluster (based on your setup), check:"
echo ""
echo "On the control plane node, check K3s service:"
echo "  sudo systemctl status k3s"
echo ""
echo "Check K3s logs:"
echo "  sudo journalctl -u k3s -n 50"
echo ""
echo "Check if K3s is listening:"
echo "  sudo netstat -tlnp | grep 6443"
echo "  # or"
echo "  sudo ss -tlnp | grep 6443"
echo ""

# Step 7: Network connectivity checks
echo "7️⃣ Network connectivity suggestions..."
echo "---------------------------------------"
if kubectl config view >/dev/null 2>&1; then
    SERVER=$(kubectl config view -o jsonpath='{.clusters[0].cluster.server}' 2>/dev/null || echo "")
    if [ -n "${SERVER}" ]; then
        HOST=$(echo "${SERVER}" | sed 's|https\?://||' | cut -d: -f1)
        PORT=$(echo "${SERVER}" | sed 's|https\?://||' | cut -d: -f2 | cut -d/ -f1)
        
        echo "Try these connectivity tests:"
        echo ""
        echo "# Ping test:"
        echo "ping -c 3 ${HOST}"
        echo ""
        echo "# DNS resolution:"
        echo "nslookup ${HOST} || dig ${HOST}"
        echo ""
        echo "# Port connectivity:"
        echo "nc -zv ${HOST} ${PORT} || telnet ${HOST} ${PORT}"
        echo ""
        echo "# HTTPS test:"
        echo "curl -k ${SERVER}/healthz || curl -k ${SERVER}/api/v1"
    fi
fi
echo ""

# Step 8: Common solutions
echo "===================================================================="
echo "📋 Common Issues and Solutions"
echo "===================================================================="
echo ""
echo "Issue 1: Kubeconfig file missing or incorrect"
echo "  Solution: Copy kubeconfig from control plane node"
echo "  ssh user@control-plane-node"
echo "  sudo cat /etc/rancher/k3s/k3s.yaml > ~/.kube/config"
echo "  # Edit server URL if needed"
echo ""
echo "Issue 2: K3s service stopped"
echo "  Solution: Restart K3s on control plane"
echo "  sudo systemctl restart k3s"
echo "  sudo systemctl status k3s"
echo ""
echo "Issue 3: Network connectivity issue"
echo "  Solution: Check network/firewall/DNS"
echo "  - Verify you can reach the control plane node"
echo "  - Check firewall rules (port 6443)"
echo "  - Verify DNS resolution"
echo ""
echo "Issue 4: Certificate expired"
echo "  Solution: Regenerate certificates or restart K3s"
echo "  sudo systemctl restart k3s"
echo "  # Then copy new kubeconfig"
echo ""
echo "Issue 5: Wrong context selected"
echo "  Solution: Switch to correct context"
echo "  kubectl config get-contexts"
echo "  kubectl config use-context <correct-context>"
echo ""
echo "Issue 6: API server crashed"
echo "  Solution: Check K3s logs and restart"
echo "  sudo journalctl -u k3s -n 100"
echo "  sudo systemctl restart k3s"
echo ""
