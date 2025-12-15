#!/bin/bash

# Test API while watching backend logs
# This helps identify if requests are reaching the backend

echo "=== Testing API with Backend Log Monitoring ==="
echo ""
echo "This script will:"
echo "1. Start watching backend logs"
echo "2. Make an API request"
echo "3. Show you what happens"
echo ""

if [ -z "$1" ]; then
  echo "Usage: $0 <JWT_TOKEN>"
  echo ""
  echo "To get your token:"
  echo "  - Open browser DevTools → Application → Local Storage"
  echo "  - Copy 'heatpump_auth_token' value"
  echo ""
  echo "Or run: ./get-token.sh"
  exit 1
fi

TOKEN="$1"
API_URL="http://localhost:8080"

echo "Token: ${TOKEN:0:20}..."
echo ""
echo "Starting log watcher in background..."
echo ""

# Start log watcher in background
kubectl logs -n redpanda-sink -l app=redpanda-sink -f --tail=0 > /tmp/backend-logs.txt 2>&1 &
LOG_PID=$!

# Wait a moment for log watcher to start
sleep 1

echo "Making API request..."
echo "---"
echo ""

# Make the request with verbose output
RESPONSE=$(curl -s -w "\n\nHTTP_CODE:%{http_code}\nTIME_TOTAL:%{time_total}" \
  --max-time 30 \
  -H "Authorization: Bearer $TOKEN" \
  "$API_URL/api/v1/energy/latest" 2>&1)

# Stop log watcher
kill $LOG_PID 2>/dev/null
wait $LOG_PID 2>/dev/null

# Extract response parts
HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
TIME_TOTAL=$(echo "$RESPONSE" | grep "TIME_TOTAL:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_CODE:/d' | sed '/TIME_TOTAL:/d')

echo "=== Request Results ==="
echo "HTTP Status: ${HTTP_CODE:-'No response'}"
echo "Time Taken: ${TIME_TOTAL:-'N/A'}s"
echo ""

if [ -n "$HTTP_CODE" ]; then
  if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Success!"
    echo "Response: $BODY" | head -c 500
    echo ""
  elif [ "$HTTP_CODE" = "401" ]; then
    echo "❌ Unauthorized - token is invalid or expired"
  elif [ "$HTTP_CODE" = "500" ]; then
    echo "❌ Server Error (500)"
    echo "Response: $BODY"
  else
    echo "⚠️  Status: $HTTP_CODE"
    echo "Response: $BODY"
  fi
else
  echo "❌ No HTTP response - request may have timed out or failed"
  echo "Full response: $RESPONSE"
fi

echo ""
echo "=== Recent Backend Logs ==="
tail -20 /tmp/backend-logs.txt 2>/dev/null || echo "No logs captured"

echo ""
echo "=== Next Steps ==="
if [ -z "$HTTP_CODE" ] || [ "$HTTP_CODE" = "500" ]; then
  echo "1. Check backend logs: kubectl logs -n redpanda-sink -l app=redpanda-sink --tail=100"
  echo "2. Check if database is accessible"
  echo "3. Verify database tables/views exist"
fi
