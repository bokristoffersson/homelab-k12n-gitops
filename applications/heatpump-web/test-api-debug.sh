#!/bin/bash

# Debug script to test API endpoints
# This will help identify why requests are hanging or returning nothing

echo "=== API Connection Test ==="
echo ""

# Check if port-forward is active
echo "1. Checking port-forward..."
if lsof -i :8080 | grep -q kubectl; then
  echo "   ✅ Port-forward is active"
else
  echo "   ❌ Port-forward is NOT active"
  echo "   Run: kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080"
  exit 1
fi

# Test health endpoint (no auth required, but might return 401)
echo ""
echo "2. Testing health endpoint (no auth)..."
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" --max-time 5 http://localhost:8080/api/v1/health 2>&1)
HEALTH_CODE=$(echo "$HEALTH_RESPONSE" | tail -n1)
HEALTH_BODY=$(echo "$HEALTH_RESPONSE" | sed '$d')

if [ -n "$HEALTH_CODE" ]; then
  echo "   Status: $HEALTH_CODE"
  if [ "$HEALTH_CODE" = "401" ]; then
    echo "   ✅ Backend is responding (401 = auth required, which is expected)"
  elif [ "$HEALTH_CODE" = "200" ]; then
    echo "   ✅ Backend is responding (200 = OK)"
  else
    echo "   ⚠️  Unexpected status: $HEALTH_CODE"
  fi
else
  echo "   ❌ No response - connection failed or timed out"
  exit 1
fi

# Check if token is provided
if [ -z "$1" ]; then
  echo ""
  echo "3. ⚠️  No token provided"
  echo ""
  echo "Usage: $0 <JWT_TOKEN>"
  echo ""
  echo "To get your token:"
  echo "  1. Open browser DevTools (F12)"
  echo "  2. Go to Application → Local Storage → http://localhost:3000"
  echo "  3. Copy the value of 'heatpump_auth_token'"
  echo ""
  echo "Or login via curl:"
  echo "  curl -X POST http://localhost:8080/api/v1/auth/login \\"
  echo "    -H 'Content-Type: application/json' \\"
  echo "    -d '{\"username\":\"admin\",\"password\":\"your-password\"}'"
  exit 1
fi

TOKEN="$1"
echo ""
echo "3. Testing with JWT token..."
echo "   Token: ${TOKEN:0:20}..."

# Test energy latest endpoint
echo ""
echo "4. Testing /api/v1/energy/latest..."
echo "   (This may take a few seconds if database query is slow)"
RESPONSE=$(curl -s -w "\n%{http_code}\n%{time_total}" --max-time 30 \
  -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/energy/latest 2>&1)

HTTP_CODE=$(echo "$RESPONSE" | tail -n2 | head -n1)
TIME_TOTAL=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d' | sed '$d')

echo "   HTTP Status: $HTTP_CODE"
echo "   Response Time: ${TIME_TOTAL}s"

if [ "$HTTP_CODE" = "200" ]; then
  echo "   ✅ Success!"
  echo "   Response: $BODY" | head -c 200
  echo ""
elif [ "$HTTP_CODE" = "401" ]; then
  echo "   ❌ Unauthorized - token is invalid or expired"
  echo "   Try logging in again to get a new token"
elif [ "$HTTP_CODE" = "500" ]; then
  echo "   ❌ Server Error (500)"
  echo "   Response body: $BODY"
  echo ""
  echo "   This indicates a backend/database issue."
  echo "   Check backend logs: kubectl logs -n redpanda-sink -l app=redpanda-sink --tail=50"
elif [ -z "$HTTP_CODE" ]; then
  echo "   ❌ No response - request timed out or connection failed"
  echo "   Check:"
  echo "     - Is port-forward still active?"
  echo "     - Is backend pod running? (kubectl get pods -n redpanda-sink)"
  echo "     - Are there errors in backend logs?"
else
  echo "   ⚠️  Unexpected status: $HTTP_CODE"
  echo "   Response: $BODY"
fi

echo ""
echo "=== Test Complete ==="
