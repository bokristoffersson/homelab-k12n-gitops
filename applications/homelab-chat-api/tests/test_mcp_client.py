"""MCP JSON-RPC client tests backed by respx."""

from __future__ import annotations

import httpx
import pytest
import respx

from homelab_chat.auth import AgentTokenProvider
from homelab_chat.mcp_client import MCPClient, MCPError, ToolRouter

TOKEN_URL = "https://authentik.example/application/o/token/"
MCP_URL_A = "http://homelab-api.test/mcp"
MCP_URL_B = "http://homelab-settings-api.test/mcp"


async def _token_provider(http_client: httpx.AsyncClient) -> AgentTokenProvider:
    return AgentTokenProvider(
        token_url=TOKEN_URL,
        client_id="agent",
        client_secret="secret",
        http_client=http_client,
    )


@pytest.mark.asyncio
async def test_list_tools_parses_response_and_sends_bearer_auth() -> None:
    async with httpx.AsyncClient() as http_client, respx.mock() as mock:
        mock.post(TOKEN_URL).mock(
            return_value=httpx.Response(
                200,
                json={"access_token": "abc123", "expires_in": 3600, "token_type": "bearer"},
            )
        )
        mcp_route = mock.post(MCP_URL_A).mock(
            return_value=httpx.Response(
                200,
                json={
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "tools": [
                            {
                                "name": "energy_latest",
                                "description": "Latest energy reading",
                                "inputSchema": {"type": "object", "properties": {}},
                            }
                        ]
                    },
                },
            )
        )

        provider = await _token_provider(http_client)
        client = MCPClient(
            url=MCP_URL_A, http_client=http_client, token_provider=provider
        )
        tools = await client.list_tools()

        assert len(tools) == 1
        assert tools[0].name == "energy_latest"
        assert tools[0].source_url == MCP_URL_A

        call = mcp_route.calls.last
        assert call.request.headers["Authorization"] == "Bearer abc123"
        assert call.request.headers["Content-Type"] == "application/json"
        body = call.request.content.decode()
        assert '"method":"tools/list"' in body.replace(" ", "")


@pytest.mark.asyncio
async def test_call_tool_raises_mcp_error_on_rpc_error() -> None:
    async with httpx.AsyncClient() as http_client, respx.mock() as mock:
        mock.post(TOKEN_URL).mock(
            return_value=httpx.Response(
                200,
                json={"access_token": "tok", "expires_in": 3600, "token_type": "bearer"},
            )
        )
        mock.post(MCP_URL_A).mock(
            return_value=httpx.Response(
                200,
                json={
                    "jsonrpc": "2.0",
                    "id": 1,
                    "error": {"code": -32601, "message": "Tool not found"},
                },
            )
        )

        provider = await _token_provider(http_client)
        client = MCPClient(
            url=MCP_URL_A, http_client=http_client, token_provider=provider
        )

        with pytest.raises(MCPError) as exc_info:
            await client.call_tool("missing", {})

        assert exc_info.value.code == -32601


@pytest.mark.asyncio
async def test_tool_router_routes_calls_to_correct_client() -> None:
    async with httpx.AsyncClient() as http_client, respx.mock() as mock:
        mock.post(TOKEN_URL).mock(
            return_value=httpx.Response(
                200,
                json={"access_token": "tok", "expires_in": 3600, "token_type": "bearer"},
            )
        )
        mock.post(MCP_URL_A).mock(
            side_effect=[
                httpx.Response(
                    200,
                    json={
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": {
                            "tools": [
                                {
                                    "name": "energy_latest",
                                    "description": "",
                                    "inputSchema": {},
                                }
                            ]
                        },
                    },
                ),
                httpx.Response(
                    200,
                    json={"jsonrpc": "2.0", "id": 2, "result": {"value": 1.0}},
                ),
            ]
        )
        b_route = mock.post(MCP_URL_B).mock(
            return_value=httpx.Response(
                200,
                json={
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "tools": [
                            {
                                "name": "list_plugs",
                                "description": "",
                                "inputSchema": {},
                            }
                        ]
                    },
                },
            )
        )

        provider = await _token_provider(http_client)
        client_a = MCPClient(
            url=MCP_URL_A, http_client=http_client, token_provider=provider
        )
        client_b = MCPClient(
            url=MCP_URL_B, http_client=http_client, token_provider=provider
        )
        router = ToolRouter([client_a, client_b])

        tools = await router.load()
        assert {t.name for t in tools} == {"energy_latest", "list_plugs"}

        result = await router.call("energy_latest", {})
        assert result == {"value": 1.0}

        # list_plugs lookup hits client_b only for the tools/list call above.
        assert b_route.call_count == 1
