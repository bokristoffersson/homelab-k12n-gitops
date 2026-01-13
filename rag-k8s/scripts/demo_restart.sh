#!/usr/bin/env bash
# Demo: Restart deployment using k8s_exec tool

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== K8s Exec Tool Demo - Restart Deployment ==="
echo ""

# Activate venv
source "$REPO_ROOT/venv/bin/activate"

# Create payload
PAYLOAD=$(cat <<EOF
{
  "intent": "restart",
  "resource": "deployment",
  "namespace": "prod",
  "name": "api",
  "constraints": {
    "confirm": false,
    "dryRun": true
  }
}
EOF
)

echo "Payload:"
echo "$PAYLOAD" | jq .
echo ""

# Save to temp file
TEMP_PAYLOAD=$(mktemp)
echo "$PAYLOAD" > "$TEMP_PAYLOAD"

# Execute
echo "Executing k8s_exec..."
python -m agent.tool "$TEMP_PAYLOAD"

# Cleanup
rm -f "$TEMP_PAYLOAD"

echo ""
echo "Demo complete."
