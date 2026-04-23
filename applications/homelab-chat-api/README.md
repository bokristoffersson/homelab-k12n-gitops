# homelab-chat-api

LLM-backed chat agent for the homelab. Exposes a single SSE endpoint that accepts a
conversation and streams back tokens, tool invocations, and tool results.

## Architecture

- **FastAPI** HTTP service (Python 3.13).
- **LLM providers** (runtime-selected by `LLM_PROVIDER`):
  - `mlx` — OpenAI-compatible client pointed at a local MLX server on the LAN.
  - `anthropic` — Anthropic API using the official SDK.
  - If `mlx` is selected and the endpoint fails with a connection/timeout/5xx, the
    service falls back to Anthropic for the duration of that request only.
- **MCP clients** — async JSON-RPC 2.0 over HTTP against the homelab-api and
  homelab-settings-api MCP endpoints. Tools from both servers are merged; calls are
  routed back to the server that advertised the tool.
- **Agent identity** — client_credentials OAuth2 token obtained from Authentik,
  cached in-process until 5 minutes before expiry.

## Endpoints

- `GET /health` — liveness probe; returns plain text `OK`.
- `POST /api/v1/chat` — body `{"messages": [{"role": "user"|"assistant", "content": "..."}]}`,
  returns `text/event-stream` with event types:
  - `token` — incremental assistant text.
  - `tool_call` — `{"name": ..., "arguments": ...}`.
  - `tool_result` — `{"name": ..., "result": ...}` or `{"name": ..., "error": ...}`.
  - `done` — terminal event.

## Running locally

```bash
uv sync
uv run uvicorn homelab_chat.main:app --host 0.0.0.0 --port 8080 --reload
```

Required environment variables:

| Variable | Purpose |
|----------|---------|
| `AUTHENTIK_TOKEN_URL` | Authentik token endpoint (client_credentials grant). |
| `AGENT_CLIENT_ID` | Confidential OAuth2 client ID for the agent. |
| `AGENT_CLIENT_SECRET` | Client secret for the agent. |
| `HOMELAB_API_MCP_URL` | MCP JSON-RPC endpoint of homelab-api. |
| `HOMELAB_SETTINGS_API_MCP_URL` | MCP JSON-RPC endpoint of homelab-settings-api. |
| `LLM_PROVIDER` | `mlx` or `anthropic`. |
| `LLM_MODEL` | Model identifier passed to the provider. |
| `MLX_SERVER_URL` | OpenAI-compatible base URL (when `LLM_PROVIDER=mlx`). |
| `ANTHROPIC_API_KEY` | API key (when `LLM_PROVIDER=anthropic` or as fallback). |
| `MAX_AGENT_ITERATIONS` | Tool-call iteration cap. Default 8. |
| `LOG_LEVEL` | Log level. Default INFO. |

## Tests

```bash
uv run ruff check
uv run ruff format --check
uv run ty check
uv run pytest
```
