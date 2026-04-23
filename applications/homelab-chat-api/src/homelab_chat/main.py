"""FastAPI application wiring: startup, endpoints, and dependency construction."""

from __future__ import annotations

import logging
import sys
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Annotated, Literal

import httpx
from fastapi import Depends, FastAPI, Header, HTTPException, Request
from fastapi.responses import PlainTextResponse
from pydantic import BaseModel
from pythonjsonlogger.json import JsonFormatter
from sse_starlette.sse import EventSourceResponse

from homelab_chat.agent import Agent, AgentConfig
from homelab_chat.auth import AgentTokenProvider
from homelab_chat.config import LLMProviderName, Settings, get_settings
from homelab_chat.llm.anthropic_provider import AnthropicProvider
from homelab_chat.llm.base import LLMProvider, Message
from homelab_chat.llm.mlx_provider import MLXProvider
from homelab_chat.mcp_client import MCPClient, ToolRouter

logger = logging.getLogger("homelab_chat")


class ChatMessage(BaseModel):
    """A single chat turn supplied by the client."""

    role: Literal["user", "assistant"]
    content: str


class ChatRequest(BaseModel):
    """The request body accepted by ``POST /api/v1/chat``."""

    messages: list[ChatMessage]


def _configure_logging(level: str) -> None:
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(
        JsonFormatter(
            "%(asctime)s %(levelname)s %(name)s %(message)s",
            rename_fields={"asctime": "timestamp", "levelname": "level"},
        )
    )
    root = logging.getLogger()
    root.handlers = [handler]
    root.setLevel(level.upper())


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    settings = get_settings()
    _configure_logging(settings.log_level)

    http_client = httpx.AsyncClient(timeout=settings.http_timeout_seconds)
    mcp_http_client = httpx.AsyncClient(timeout=settings.http_timeout_seconds)

    token_provider = AgentTokenProvider(
        token_url=settings.authentik_token_url,
        client_id=settings.agent_client_id,
        client_secret=settings.agent_client_secret,
        http_client=http_client,
    )

    tool_router = ToolRouter(
        [
            MCPClient(
                url=settings.homelab_api_mcp_url,
                http_client=mcp_http_client,
                token_provider=token_provider,
            ),
            MCPClient(
                url=settings.homelab_settings_api_mcp_url,
                http_client=mcp_http_client,
                token_provider=token_provider,
            ),
        ]
    )

    app.state.settings = settings
    app.state.http_client = http_client
    app.state.mcp_http_client = mcp_http_client
    app.state.token_provider = token_provider
    app.state.tool_router = tool_router

    logger.info("homelab-chat-api started", extra={"provider": settings.llm_provider.value})
    try:
        yield
    finally:
        await http_client.aclose()
        await mcp_http_client.aclose()


app = FastAPI(title="homelab-chat-api", lifespan=lifespan)


@app.get("/health", response_class=PlainTextResponse)
async def health() -> str:
    """Liveness probe. Returns 200 OK when the process is serving."""
    return "OK"


def _build_provider(
    settings: Settings,
    name: LLMProviderName,
    http_client: httpx.AsyncClient,
) -> LLMProvider:
    if name is LLMProviderName.MLX:
        if not settings.mlx_server_url:
            raise RuntimeError("LLM_PROVIDER=mlx but MLX_SERVER_URL is not set")
        return MLXProvider(
            base_url=settings.mlx_server_url,
            model=settings.llm_model,
            http_client=http_client,
        )
    if not settings.anthropic_api_key:
        raise RuntimeError("LLM_PROVIDER=anthropic but ANTHROPIC_API_KEY is not set")
    return AnthropicProvider(api_key=settings.anthropic_api_key, model=settings.llm_model)


def get_agent(request: Request) -> Agent:
    """FastAPI dependency that constructs a per-request :class:`Agent`."""
    settings: Settings = request.app.state.settings
    http_client: httpx.AsyncClient = request.app.state.http_client
    tool_router: ToolRouter = request.app.state.tool_router

    primary = _build_provider(settings, settings.llm_provider, http_client)
    fallback: LLMProvider | None = None
    if settings.llm_provider is LLMProviderName.MLX and settings.anthropic_api_key:
        fallback = _build_provider(settings, LLMProviderName.ANTHROPIC, http_client)

    return Agent(
        primary_provider=primary,
        fallback_provider=fallback,
        tool_router=tool_router,
        config=AgentConfig(max_iterations=settings.max_agent_iterations),
    )


@app.post("/api/v1/chat")
async def chat(
    payload: ChatRequest,
    agent: Annotated[Agent, Depends(get_agent)],
    x_forwarded_user: Annotated[str | None, Header(alias="X-Forwarded-User")] = None,
) -> EventSourceResponse:
    """Run the agent loop and stream SSE events back to the caller."""
    if not payload.messages:
        raise HTTPException(status_code=400, detail="messages must not be empty")

    user = x_forwarded_user or "anonymous"
    logger.info(
        "chat request received",
        extra={"user": user, "message_count": len(payload.messages)},
    )

    messages = [Message(role=m.role, content=m.content) for m in payload.messages]

    async def event_generator() -> AsyncIterator[dict[str, str]]:
        async for sse_event in agent.run(messages):
            yield sse_event.render()

    return EventSourceResponse(event_generator())
