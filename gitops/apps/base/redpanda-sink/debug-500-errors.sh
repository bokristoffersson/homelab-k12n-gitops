#!/bin/bash

# Script to debug 500 errors by monitoring logs while making API requests

NAMESPACE="${NAMESPACE:-redpanda-sink}"
DEPLOYMENT="${DEPLOYMENT:-redpanda-sink}"

echo "=== Debugging 500 Errors ==="
echo ""
echo "This script will:"
echo "1. Show recent logs"
echo "2. Start monitoring logs in real-time"
echo "3. Make an API request"
echo "4. Show the error details"
echo ""

# Check if token is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <JWT_TOKEN> [API_ENDPOINT]"
  echo ""
  echo "Example:"
  echo "  $0 \"your-token\" \"/api/v1/energy/latest\""
  echo ""
  echo "Get token from:"
  echo "  curl -X POST https://api.k12n.com/api/v1/auth/login \\"
  echo "    -H 'Content-Type: application/json' \\"
  echo "    -d '{\"username\":\"admin\",\"password\":\"your-password\"}'"
  exit 1
fi

TOKEN="$1"
ENDPOINT="${2:-/api/v1/energy/latest}"
API_URL="${API_URL:-https://api.k12n.com}"

echo "Token: ${TOKEN:0:30}..."
echo "Endpoint: $ENDPOINT"
echo "API URL: $API_URL"
echo ""

# Show recent logs
echo "=== Recent Logs (last 20 lines) ==="
kubectl logs -n $NAMESPACE deployment/$DEPLOYMENT --tail=20
echo ""

# Start log monitoring in background
echo "=== Starting Real-time Log Monitoring ==="
echo "Press Ctrl+C to stop monitoring"
echo ""

# Create a temporary log file
LOG_FILE=$(mktemp)
kubectl logs -n $NAMESPACE deployment/$DEPLOYMENT -f --tail=0 > "$LOG_FILE" 2>&1 &
LOG_PID=$!

# Wait a moment for log watcher to start
sleep 2

# Make the API request
echo "=== Making API Request ==="
echo "Request: GET $API_URL$ENDPOINT"
echo ""

RESPONSE=$(curl -s -w "\n\nHTTP_CODE:%{http_code}\nTIME:%{time_total}s" \
  --max-time 30 \
  -H "Authorization: Bearer $TOKEN" \
  "$API_URL$ENDPOINT" 2>&1)

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
TIME=$(echo "$RESPONSE" | grep "^TIME:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_CODE:/d' | sed '/TIME:/d')

# Stop log monitoring
sleep 2
kill $LOG_PID 2>/dev/null
wait $LOG_PID 2>/dev/null

echo "=== API Response ==="
echo "HTTP Status: ${HTTP_CODE:-'No response'}"
echo "Time: ${TIME:-'N/A'}s"
echo ""

if [ "$HTTP_CODE" = "500" ]; then
  echo "❌ Server Error (500)"
  echo ""
  echo "Response body:"
  echo "$BODY" | head -50
  echo ""
elif [ "$HTTP_CODE" = "200" ]; then
  echo "✅ Success!"
  echo "Response: $BODY" | head -20
else
  echo "Status: $HTTP_CODE"
  echo "Response: $BODY"
fi

echo ""
echo "=== Logs Captured During Request ==="
if [ -f "$LOG_FILE" ]; then
  cat "$LOG_FILE"
  rm "$LOG_FILE"
else
  echo "No logs captured"
fi

echo ""
echo "=== Filtered Error Logs ==="
kubectl logs -n $NAMESPACE deployment/$DEPLOYMENT --tail=100 | grep -i "error\|panic\|failed\|500\|database\|query" | tail -20

echo ""
echo "=== Next Steps ==="
if [ "$HTTP_CODE" = "500" ]; then
  echo "1. Check the logs above for error messages"
  echo "2. Look for database-related errors (missing tables, connection issues)"
  echo "3. Check if database migrations have been run"
  echo "4. Verify database tables/views exist"
fi
