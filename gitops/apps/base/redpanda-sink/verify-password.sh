#!/bin/bash

# Verify password hash and test login

NAMESPACE="redpanda-sink"
PASSWORD="afc1a8737586a28af3360812b7431bbd22f6bd8a5d015336"

echo "=== Step 1: Check Secret Value ==="
echo "Password hash stored in secret (first 50 chars):"
STORED_HASH=$(kubectl get secret auth-secret -n $NAMESPACE -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d)
echo "$STORED_HASH" | head -c 50
echo "..."
echo ""

echo "Hash format check (should start with \$2b\$):"
echo "$STORED_HASH" | head -c 10
echo ""
if echo "$STORED_HASH" | grep -q '^\$2[ab]\$'; then
    echo "✓ Valid bcrypt hash format"
else
    echo "✗ Invalid hash format - should start with \$2b\$"
fi
echo ""

echo "=== Step 2: Check Environment Variable in Pod ==="
echo "ADMIN_PASSWORD_HASH env var (first 50 chars):"
POD_NAME=$(kubectl get pods -n $NAMESPACE -l app=redpanda-sink -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n $NAMESPACE $POD_NAME -- env | grep ADMIN_PASSWORD_HASH | head -c 50
echo "..."
echo ""

echo "=== Step 3: Test Login ==="
kubectl run --rm -i test-login-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -X POST http://redpanda-sink:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"admin\",\"password\":\"$PASSWORD\"}" \
    -w "\n\nHTTP Status: %{http_code}\n" \
    -s

echo ""
echo "=== Step 4: Check Application Logs for Auth Errors ==="
kubectl logs -n $NAMESPACE deployment/redpanda-sink --tail=10 | grep -i "auth\|login\|password\|unauthorized" || echo "No auth-related log entries found"
