"""Anthropic provider using the official SDK with streaming tool-use support."""

from __future__ import annotations

import logging
from collections.abc import AsyncIterator
from typing import Any

from anthropic import APIConnectionError as AnthropicConnectionError
from anthropic import APIStatusError as AnthropicStatusError
from anthropic import AsyncAnthropic

from homelab_chat.llm.base import (
    LLMProvider,
    Message,
    ProviderError,
    ProviderEvent,
    ProviderUnavailableError,
    ToolCall,
    ToolSchema,
)

logger = logging.getLogger(__name__)


class AnthropicProvider(LLMProvider):
    """Streaming Anthropic provider."""

    def __init__(self, *, api_key: str, model: str, max_tokens: int = 4096) -> None:
        self._client = AsyncAnthropic(api_key=api_key)
        self._model = model
        self._max_tokens = max_tokens

    async def stream(
        self,
        messages: list[Message],
        tools: list[ToolSchema],
    ) -> AsyncIterator[ProviderEvent]:
        system_prompt, anthropic_messages = _to_anthropic_messages(messages)
        anthropic_tools = [
            {
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.input_schema,
            }
            for tool in tools
        ]

        try:
            stream_cm = self._client.messages.stream(
                model=self._model,
                max_tokens=self._max_tokens,
                system=system_prompt,
                messages=anthropic_messages,
                tools=anthropic_tools,
            )
            async with stream_cm as stream:
                async for event in _translate_stream(stream):
                    yield event
        except AnthropicConnectionError as exc:
            raise ProviderUnavailableError(f"Anthropic connection error: {exc}") from exc
        except AnthropicStatusError as exc:
            if exc.status_code >= 500:
                raise ProviderUnavailableError(
                    f"Anthropic returned {exc.status_code}: {exc.message}"
                ) from exc
            raise ProviderError(f"Anthropic returned {exc.status_code}: {exc.message}") from exc


async def _translate_stream(stream: Any) -> AsyncIterator[ProviderEvent]:
    async for event in stream:
        event_type = getattr(event, "type", None)
        if event_type == "content_block_delta":
            delta = getattr(event, "delta", None)
            delta_type = getattr(delta, "type", None)
            if delta_type == "text_delta":
                text = getattr(delta, "text", "")
                if text:
                    yield ProviderEvent(text_delta=text)

    final_message = await stream.get_final_message()
    for block in final_message.content:
        if getattr(block, "type", None) == "tool_use":
            tool_input = getattr(block, "input", {}) or {}
            yield ProviderEvent(
                tool_call=ToolCall(
                    id=str(getattr(block, "id", "")),
                    name=str(getattr(block, "name", "")),
                    arguments=dict(tool_input) if isinstance(tool_input, dict) else {},
                )
            )

    yield ProviderEvent(finish=True)


def _to_anthropic_messages(
    messages: list[Message],
) -> tuple[str, list[dict[str, Any]]]:
    system_prompt = (
        "You are the homelab assistant. Answer factual questions by calling the "
        "available tools; prefer a single tool call over speculation."
    )
    out: list[dict[str, Any]] = []
    for msg in messages:
        if msg.role == "user":
            out.append({"role": "user", "content": msg.content})
        elif msg.role == "assistant":
            blocks: list[dict[str, Any]] = []
            if msg.content:
                blocks.append({"type": "text", "text": msg.content})
            for call in msg.tool_calls:
                blocks.append(
                    {
                        "type": "tool_use",
                        "id": call.id,
                        "name": call.name,
                        "input": call.arguments,
                    }
                )
            if blocks:
                out.append({"role": "assistant", "content": blocks})
        elif msg.role == "tool":
            out.append(
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": msg.tool_call_id or "",
                            "content": msg.content,
                        }
                    ],
                }
            )
    return system_prompt, out
