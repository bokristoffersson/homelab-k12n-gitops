#!/bin/bash
# MCP Proxy for Claude Desktop
# This script acts as a stdio bridge between Claude Desktop and the remote homelab MCP server

set -e

# Configuration
MCP_URL="https://api.k12n.com/mcp"
TOKEN_URL="https://authentik.k12n.com/application/o/token/"
CLIENT_ID="claude-mcp-client"
CLIENT_SECRET="${HOMELAB_MCP_CLIENT_SECRET}"

if [ -z "$CLIENT_SECRET" ]; then
  echo '{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"HOMELAB_MCP_CLIENT_SECRET environment variable not set"}}' >&2
  exit 1
fi

# Get OAuth2 token
get_token() {
  curl -s -X POST "$TOKEN_URL" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=client_credentials" \
    -d "client_id=$CLIENT_ID" \
    -d "client_secret=$CLIENT_SECRET" | jq -r '.access_token'
}

TOKEN=$(get_token)

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo '{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Failed to obtain OAuth2 token"}}' >&2
  exit 1
fi

# Read JSON-RPC request from stdin
while IFS= read -r line; do
  # Forward request to remote MCP server with Bearer token
  response=$(curl -s -X POST "$MCP_URL" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "$line")

  # Return response to stdout
  echo "$response"
done
