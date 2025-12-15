#!/bin/bash

# Test API endpoints to debug 500 errors
# Usage: ./test-api.sh <JWT_TOKEN>

TOKEN="${1:-}"

if [ -z "$TOKEN" ]; then
  echo "Usage: ./test-api.sh <JWT_TOKEN>"
  echo ""
  echo "To get your token:"
  echo "1. Open browser DevTools → Application → Local Storage"
  echo "2. Copy the value of 'heatpump_auth_token'"
  echo ""
  echo "Or login via curl:"
  echo "curl -X POST http://localhost:8080/api/v1/auth/login \\"
  echo "  -H 'Content-Type: application/json' \\"
  echo "  -d '{\"username\":\"admin\",\"password\":\"your-password\"}'"
  exit 1
fi

API_URL="${VITE_API_URL:-http://localhost:8080}"

echo "Testing API endpoints at $API_URL"
echo "=================================="
echo ""

test_endpoint() {
  local endpoint=$1
  local name=$2
  
  echo "Testing $name..."
  echo "  Endpoint: $endpoint"
  
  response=$(curl -s -w "\n%{http_code}" \
    -H "Authorization: Bearer $TOKEN" \
    "$API_URL$endpoint")
  
  http_code=$(echo "$response" | tail -n1)
  body=$(echo "$response" | sed '$d')
  
  if [ "$http_code" = "200" ]; then
    echo "  ✅ Status: $http_code"
    echo "  Response: $(echo "$body" | head -c 100)..."
  else
    echo "  ❌ Status: $http_code"
    echo "  Response: $body"
  fi
  echo ""
}

test_endpoint "/api/v1/energy/latest" "Energy Latest"
test_endpoint "/api/v1/energy/hourly-total" "Hourly Total"
test_endpoint "/api/v1/heatpump/latest" "Heatpump Latest"
test_endpoint "/api/v1/energy/history?from=$(date -u -v-1d +%Y-%m-%dT%H:%M:%SZ)&to=$(date -u +%Y-%m-%dT%H:%M:%SZ)" "Energy History"

echo "=================================="
echo "Done!"
