"""Tool-calling agent loop that streams SSE events for the HTTP handler."""

from __future__ import annotations

import json
import logging
from collections.abc import AsyncIterator
from dataclasses import dataclass
from typing import Any

from homelab_chat.llm.base import (
    LLMProvider,
    Message,
    ProviderError,
    ProviderUnavailableError,
    ToolCall,
    ToolSchema,
)
from homelab_chat.mcp_client import MCPError, ToolRouter

logger = logging.getLogger(__name__)


@dataclass(frozen=True, slots=True)
class SSEEvent:
    """An event to write to the SSE response stream."""

    event: str
    data: dict[str, Any]

    def render(self) -> dict[str, str]:
        """Return the payload in the shape sse-starlette expects."""
        return {"event": self.event, "data": json.dumps(self.data)}


@dataclass(frozen=True, slots=True)
class AgentConfig:
    """Runtime settings for :class:`Agent`."""

    max_iterations: int


class Agent:
    """Runs the LLM tool-calling loop against a :class:`ToolRouter`."""

    def __init__(
        self,
        *,
        primary_provider: LLMProvider,
        fallback_provider: LLMProvider | None,
        tool_router: ToolRouter,
        config: AgentConfig,
    ) -> None:
        self._primary = primary_provider
        self._fallback = fallback_provider
        self._router = tool_router
        self._config = config

    async def run(self, messages: list[Message]) -> AsyncIterator[SSEEvent]:
        """Run the agent loop, yielding SSE events until a final answer or limit."""
        tool_defs = await self._router.ensure_loaded()
        tools: list[ToolSchema] = [
            ToolSchema(name=t.name, description=t.description, input_schema=t.input_schema)
            for t in tool_defs
        ]

        conversation = list(messages)
        provider = self._primary
        used_fallback = False

        for iteration in range(self._config.max_iterations):
            assistant_text_parts: list[str] = []
            tool_calls: list[ToolCall] = []

            try:
                async for event in provider.stream(conversation, tools):
                    if event.text_delta is not None:
                        assistant_text_parts.append(event.text_delta)
                        yield SSEEvent(event="token", data={"text": event.text_delta})
                    if event.tool_call is not None:
                        tool_calls.append(event.tool_call)
                    if event.finish:
                        break
            except ProviderUnavailableError as exc:
                if used_fallback or self._fallback is None:
                    raise
                logger.warning(
                    "primary LLM provider unavailable, falling back for this request: %s",
                    exc,
                )
                provider = self._fallback
                used_fallback = True
                continue

            assistant_text = "".join(assistant_text_parts)

            if not tool_calls:
                yield SSEEvent(event="done", data={"iterations": iteration + 1})
                return

            conversation.append(
                Message(role="assistant", content=assistant_text, tool_calls=tool_calls)
            )

            for call in tool_calls:
                yield SSEEvent(
                    event="tool_call",
                    data={"id": call.id, "name": call.name, "arguments": call.arguments},
                )
                tool_message = await self._execute_tool(call)
                yield _tool_result_event(call, tool_message)
                conversation.append(tool_message)

        yield SSEEvent(
            event="done",
            data={
                "iterations": self._config.max_iterations,
                "reason": "max_iterations_reached",
            },
        )

    async def _execute_tool(self, call: ToolCall) -> Message:
        try:
            result = await self._router.call(call.name, call.arguments)
        except MCPError as exc:
            logger.warning("tool %s failed: %s", call.name, exc)
            error_payload = json.dumps({"error": {"code": exc.code, "message": exc.message}})
            return Message(
                role="tool",
                content=error_payload,
                tool_call_id=call.id,
                name=call.name,
            )
        except ProviderError as exc:
            logger.warning("tool %s failed with provider error: %s", call.name, exc)
            return Message(
                role="tool",
                content=json.dumps({"error": {"message": str(exc)}}),
                tool_call_id=call.id,
                name=call.name,
            )

        return Message(
            role="tool",
            content=_serialize_tool_result(result),
            tool_call_id=call.id,
            name=call.name,
        )


def _serialize_tool_result(result: Any) -> str:
    if isinstance(result, str):
        return result
    try:
        return json.dumps(result)
    except (TypeError, ValueError):
        return str(result)


def _tool_result_event(call: ToolCall, message: Message) -> SSEEvent:
    try:
        parsed = json.loads(message.content) if message.content else None
    except json.JSONDecodeError:
        parsed = message.content

    if isinstance(parsed, dict) and "error" in parsed:
        return SSEEvent(
            event="tool_result",
            data={"id": call.id, "name": call.name, "error": parsed["error"]},
        )
    return SSEEvent(
        event="tool_result",
        data={"id": call.id, "name": call.name, "result": parsed},
    )
