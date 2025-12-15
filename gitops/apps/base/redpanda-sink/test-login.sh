#!/bin/bash

# Test login endpoint and show full output

NAMESPACE="redpanda-sink"
SERVICE="redpanda-sink"
PASSWORD="afc1a8737586a28af3360812b7431bbd22f6bd8a5d015336"

echo "=== Testing Login Endpoint ==="
echo "Username: admin"
echo "Password: $PASSWORD"
echo ""

kubectl run --rm -i test-login-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -X POST http://$SERVICE:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"admin\",\"password\":\"$PASSWORD\"}" \
    -w "\n\nHTTP Status: %{http_code}\n" \
    -s

echo ""
echo "=== Checking Configuration ==="
echo "Username in config:"
kubectl get configmap redpanda-sink-config -n $NAMESPACE -o yaml | grep -A 1 "username:" | head -2

echo ""
echo "Password hash format (first 30 chars):"
kubectl get secret auth-secret -n $NAMESPACE -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d | head -c 30
echo "..."
