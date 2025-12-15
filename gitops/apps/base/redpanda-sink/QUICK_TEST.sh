#!/bin/bash

# Quick test script for redpanda-sink API

NAMESPACE="redpanda-sink"
SERVICE="redpanda-sink"

echo "=== Testing Health Endpoint (from within cluster) ==="
kubectl run -it --rm test-health-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -s -w "\nHTTP Status: %{http_code}\n" http://$SERVICE:8080/health

echo -e "\n=== Testing Login Endpoint ==="
echo "Enter your password when prompted, or modify the script to include it"
read -sp "Password: " PASSWORD
echo ""

kubectl run -it --rm test-login-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -X POST http://$SERVICE:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"admin\",\"password\":\"$PASSWORD\"}" \
    -s -w "\nHTTP Status: %{http_code}\n"

echo -e "\n=== Recent Application Logs ==="
kubectl logs -n $NAMESPACE deployment/$SERVICE --tail=20
