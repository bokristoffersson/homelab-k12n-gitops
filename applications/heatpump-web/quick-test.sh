#!/bin/bash

# Quick test script - shows exactly what's happening

if [ -z "$1" ]; then
  echo "Usage: $0 <JWT_TOKEN>"
  echo ""
  echo "Quick test - will show HTTP status and response time"
  echo ""
  echo "Get token from browser:"
  echo "  DevTools → Application → Local Storage → heatpump_auth_token"
  exit 1
fi

TOKEN="$1"

echo "Testing API endpoint..."
echo "Token: ${TOKEN:0:30}..."
echo ""

# Test with full output
RESPONSE=$(curl -s -w "\n\n===STATS===\nHTTP_CODE:%{http_code}\nTIME:%{time_total}s\nSIZE:%{size_download} bytes\n" \
  --max-time 30 \
  -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/energy/latest)

# Extract stats
HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
TIME=$(echo "$RESPONSE" | grep "^TIME:" | cut -d: -f2)
SIZE=$(echo "$RESPONSE" | grep "SIZE:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed '/===STATS===/,$d')

echo "=== Results ==="
echo "HTTP Status: ${HTTP_CODE:-'No response'}"
echo "Time: ${TIME:-'N/A'}s"
echo "Response Size: ${SIZE:-'0'} bytes"
echo ""

if [ -n "$HTTP_CODE" ]; then
  if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ SUCCESS!"
    echo ""
    echo "Response body:"
    echo "$BODY" | head -20
  elif [ "$HTTP_CODE" = "401" ]; then
    echo "❌ Unauthorized (401)"
    echo "Token is invalid or expired. Try logging in again."
  elif [ "$HTTP_CODE" = "500" ]; then
    echo "❌ Server Error (500)"
    echo ""
    echo "Response body:"
    echo "$BODY"
    echo ""
    echo "This is a backend/database issue."
  else
    echo "⚠️  Status: $HTTP_CODE"
    echo "Response: $BODY"
  fi
else
  echo "❌ No response received"
  echo "Request may have timed out or connection failed"
  echo ""
  echo "Full output:"
  echo "$RESPONSE"
fi
