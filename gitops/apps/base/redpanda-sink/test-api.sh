#!/bin/bash

# Test redpanda-sink API from within the cluster

NAMESPACE="redpanda-sink"
SERVICE="redpanda-sink"

echo "=== Testing Health Endpoint ==="
kubectl run -it --rm test-health-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -s -w "\nHTTP Status: %{http_code}\n" http://$SERVICE:8080/health

echo -e "\n=== Testing Login Endpoint ==="
echo "Testing with password: afc1a8737586a28af3360812b7431bbd22f6bd8a5d015336"
kubectl run -it --rm test-login-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -X POST http://$SERVICE:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"admin","password":"afc1a8737586a28af3360812b7431bbd22f6bd8a5d015336"}' \
    -s -w "\nHTTP Status: %{http_code}\n" | head -20

echo -e "\n=== Checking Password Hash in Secret ==="
echo "Stored hash (first 20 chars):"
kubectl get secret auth-secret -n $NAMESPACE -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d | head -c 20
echo "..."

echo -e "\n=== Checking Username in Config ==="
kubectl get configmap redpanda-sink-config -n $NAMESPACE -o yaml | grep -A 1 "username:"
