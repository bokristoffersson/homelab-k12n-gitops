#!/bin/bash

# Debug login 500 error

NAMESPACE="redpanda-sink"
PASSWORD="afc1a8737586a28af3360812b7431bbd22f6bd8a5d015336"

echo "=== Step 1: Check Recent Logs (last 30 lines) ==="
kubectl logs -n $NAMESPACE deployment/redpanda-sink --tail=30

echo -e "\n=== Step 2: Check for Errors ==="
kubectl logs -n $NAMESPACE deployment/redpanda-sink | grep -i "error\|panic\|failed" | tail -10

echo -e "\n=== Step 3: Test Login and Watch Logs ==="
echo "Attempting login (watch logs in another terminal with: kubectl logs -n $NAMESPACE -f deployment/redpanda-sink)"
kubectl run --rm -i test-login-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -X POST http://redpanda-sink:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"admin\",\"password\":\"$PASSWORD\"}" \
    -v 2>&1

echo -e "\n=== Step 4: Check Logs After Login Attempt ==="
sleep 1
kubectl logs -n $NAMESPACE deployment/redpanda-sink --tail=10

echo -e "\n=== Step 5: Verify Secret Values ==="
echo "JWT_SECRET exists:"
kubectl get secret auth-secret -n $NAMESPACE -o jsonpath='{.data.JWT_SECRET}' | base64 -d | head -c 20
echo "..."

echo -e "\nADMIN_PASSWORD_HASH exists:"
kubectl get secret auth-secret -n $NAMESPACE -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d | head -c 20
echo "..."

echo -e "\n=== Step 6: Check Config in Pod ==="
POD_NAME=$(kubectl get pods -n $NAMESPACE -l app=redpanda-sink -o jsonpath='{.items[0].metadata.name}')
echo "Checking if ADMIN_PASSWORD_HASH env var is set:"
kubectl exec -n $NAMESPACE $POD_NAME -- env | grep ADMIN_PASSWORD_HASH | head -c 50
echo "..."
