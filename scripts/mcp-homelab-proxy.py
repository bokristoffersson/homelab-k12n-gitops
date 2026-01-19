#!/Users/bo/Development/homelab/Cursor_Workspace/homelab-k12n-gitops/scripts/.venv-mcp/bin/python3
"""
MCP Proxy for Claude Desktop
Acts as a stdio bridge between Claude Desktop and the remote homelab MCP server
"""

import os
import sys
import json
import requests
from typing import Optional

# Configuration
MCP_URL = "https://api.k12n.com/mcp"
TOKEN_URL = "https://authentik.k12n.com/application/o/token/"
CLIENT_ID = "claude-mcp-client"
CLIENT_SECRET = os.environ.get("HOMELAB_MCP_CLIENT_SECRET")


def log_error(message: str):
    """Log error to stderr"""
    sys.stderr.write(f"{message}\n")
    sys.stderr.flush()


def get_oauth_token() -> Optional[str]:
    """Get OAuth2 access token using client credentials"""
    if not CLIENT_SECRET:
        log_error("ERROR: HOMELAB_MCP_CLIENT_SECRET environment variable not set")
        return None

    try:
        response = requests.post(
            TOKEN_URL,
            data={
                "grant_type": "client_credentials",
                "client_id": CLIENT_ID,
                "client_secret": CLIENT_SECRET,
            },
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            timeout=10,
        )
        response.raise_for_status()
        data = response.json()
        return data.get("access_token")
    except requests.exceptions.HTTPError as e:
        log_error(f"ERROR: OAuth2 HTTP error {e.response.status_code}: {e.response.text}")
        return None
    except Exception as e:
        log_error(f"ERROR: Failed to obtain OAuth2 token: {e}")
        return None


def send_request(token: str, request_data: dict) -> dict:
    """Send JSON-RPC request to remote MCP server"""
    try:
        response = requests.post(
            MCP_URL,
            json=request_data,
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
            },
            timeout=30,
        )
        response.raise_for_status()
        return response.json()
    except Exception as e:
        log_error(f"ERROR: Failed to send request: {e}")
        return {
            "jsonrpc": "2.0",
            "id": request_data.get("id", 0),
            "error": {
                "code": -32603,
                "message": f"Internal error: {str(e)}",
            },
        }


def main():
    """Main loop: read from stdin, forward to remote server, write to stdout"""
    # Get OAuth token (lazily, so we can respond to first request with proper ID)
    token = None
    token_error = None

    # Process JSON-RPC requests from stdin
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            request_data = json.loads(line)
            request_id = request_data.get("id", 0)

            # Lazy token fetch on first request
            if token is None and token_error is None:
                token = get_oauth_token()
                if not token:
                    token_error = "Failed to obtain OAuth2 token"

            # If we have a token error, return it with the proper request ID
            if token_error:
                error_response = {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": -32603,
                        "message": token_error,
                    },
                }
                print(json.dumps(error_response))
                sys.stdout.flush()
                continue

            # Forward request to remote MCP server
            response = send_request(token, request_data)
            print(json.dumps(response))
            sys.stdout.flush()
        except json.JSONDecodeError as e:
            log_error(f"ERROR: Invalid JSON: {e}")
            error_response = {
                "jsonrpc": "2.0",
                "id": 0,  # Use 0 as default if we can't parse the request
                "error": {
                    "code": -32700,
                    "message": "Parse error",
                },
            }
            print(json.dumps(error_response))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
