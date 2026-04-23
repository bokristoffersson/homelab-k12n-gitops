"""Async JSON-RPC 2.0 client for MCP servers reachable over HTTP."""

from __future__ import annotations

import itertools
import logging
from dataclasses import dataclass
from typing import Any

import httpx

from homelab_chat.auth import AgentTokenProvider

logger = logging.getLogger(__name__)


class MCPError(RuntimeError):
    """Raised when an MCP server returns a JSON-RPC error payload."""

    def __init__(self, code: int, message: str, data: Any | None = None) -> None:
        super().__init__(f"MCP error {code}: {message}")
        self.code = code
        self.message = message
        self.data = data


@dataclass(frozen=True, slots=True)
class ToolDefinition:
    """A single tool as advertised by an MCP server."""

    name: str
    description: str
    input_schema: dict[str, Any]
    source_url: str


class MCPClient:
    """Thin JSON-RPC 2.0 client bound to a single MCP endpoint URL."""

    def __init__(
        self,
        *,
        url: str,
        http_client: httpx.AsyncClient,
        token_provider: AgentTokenProvider,
    ) -> None:
        self._url = url
        self._http = http_client
        self._token_provider = token_provider
        self._id_counter = itertools.count(1)

    @property
    def url(self) -> str:
        """The endpoint URL this client is bound to."""
        return self._url

    async def list_tools(self) -> list[ToolDefinition]:
        """Return the tools advertised by this MCP server."""
        result = await self._call("tools/list", {})
        raw_tools = result.get("tools", []) if isinstance(result, dict) else []

        tools: list[ToolDefinition] = []
        for raw in raw_tools:
            if not isinstance(raw, dict):
                continue
            name = raw.get("name")
            description = raw.get("description", "")
            input_schema = raw.get("inputSchema", {})
            if not isinstance(name, str) or not isinstance(input_schema, dict):
                logger.warning("skipping malformed tool entry: %r", raw)
                continue
            tools.append(
                ToolDefinition(
                    name=name,
                    description=str(description),
                    input_schema=input_schema,
                    source_url=self._url,
                )
            )
        return tools

    async def call_tool(self, name: str, arguments: dict[str, Any]) -> Any:
        """Invoke ``tools/call`` and return the result payload."""
        return await self._call("tools/call", {"name": name, "arguments": arguments})

    async def _call(self, method: str, params: dict[str, Any]) -> Any:
        request_id = next(self._id_counter)
        token = await self._token_provider.get_token()
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        }
        response = await self._http.post(
            self._url,
            json=payload,
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
                "Accept": "application/json",
            },
        )
        if response.status_code >= 400:
            raise MCPError(
                code=response.status_code,
                message=f"MCP HTTP error: {response.text}",
            )
        body = response.json()
        if not isinstance(body, dict):
            raise MCPError(code=-32603, message="MCP response was not a JSON object")

        if "error" in body and body["error"] is not None:
            err = body["error"]
            if isinstance(err, dict):
                raise MCPError(
                    code=int(err.get("code", -32603)),
                    message=str(err.get("message", "unknown error")),
                    data=err.get("data"),
                )
            raise MCPError(code=-32603, message=f"MCP error: {err}")

        return body.get("result")


class ToolRouter:
    """Merges tools from multiple MCP clients and routes calls back to their source."""

    def __init__(self, clients: list[MCPClient]) -> None:
        self._clients = clients
        self._tools: list[ToolDefinition] = []
        self._name_to_client: dict[str, MCPClient] = {}
        self._loaded = False

    async def load(self) -> list[ToolDefinition]:
        """Fetch ``tools/list`` from every client and build the routing table."""
        combined: list[ToolDefinition] = []
        name_to_client: dict[str, MCPClient] = {}

        for client in self._clients:
            try:
                tools = await client.list_tools()
            except (MCPError, httpx.HTTPError) as exc:
                logger.warning("tools/list failed for %s: %s", client.url, exc)
                continue

            for tool in tools:
                if tool.name in name_to_client:
                    logger.warning(
                        "duplicate tool name %s advertised by %s; keeping first from %s",
                        tool.name,
                        client.url,
                        name_to_client[tool.name].url,
                    )
                    continue
                combined.append(tool)
                name_to_client[tool.name] = client

        self._tools = combined
        self._name_to_client = name_to_client
        self._loaded = True
        return combined

    async def ensure_loaded(self) -> list[ToolDefinition]:
        """Load the routing table on first access, then return the cached result."""
        if not self._loaded:
            await self.load()
        return self._tools

    async def call(self, name: str, arguments: dict[str, Any]) -> Any:
        """Route a tool call to the MCP server that advertised the tool."""
        await self.ensure_loaded()
        client = self._name_to_client.get(name)
        if client is None:
            raise MCPError(
                code=-32601,
                message=f"tool {name!r} is not advertised by any configured MCP server",
            )
        return await client.call_tool(name, arguments)
