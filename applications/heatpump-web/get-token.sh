#!/bin/bash

# Script to get a fresh JWT token by logging in

API_URL="${VITE_API_URL:-http://localhost:8080}"

echo "=== Get JWT Token ==="
echo ""
echo "Enter your credentials:"
read -p "Username: " USERNAME
read -sp "Password: " PASSWORD
echo ""

echo ""
echo "Logging in..."
RESPONSE=$(curl -s -w "\n%{http_code}" \
  -X POST "$API_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
  TOKEN=$(echo "$BODY" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
  if [ -n "$TOKEN" ]; then
    echo "✅ Login successful!"
    echo ""
    echo "Your JWT token:"
    echo "$TOKEN"
    echo ""
    echo "To use it:"
    echo "  export TOKEN=\"$TOKEN\""
    echo "  curl -H \"Authorization: Bearer \$TOKEN\" http://localhost:8080/api/v1/energy/latest"
    echo ""
    echo "Or test with the debug script:"
    echo "  ./test-api-debug.sh \"$TOKEN\""
  else
    echo "❌ Could not extract token from response"
    echo "Response: $BODY"
  fi
else
  echo "❌ Login failed (Status: $HTTP_CODE)"
  echo "Response: $BODY"
  exit 1
fi
