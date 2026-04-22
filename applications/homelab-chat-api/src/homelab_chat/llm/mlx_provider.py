"""MLX provider that talks to an OpenAI-compatible /v1/chat/completions endpoint."""

from __future__ import annotations

import json
import logging
from collections.abc import AsyncIterator
from typing import Any

import httpx

from homelab_chat.llm.base import (
    LLMProvider,
    Message,
    ProviderEvent,
    ProviderError,
    ProviderUnavailable,
    ToolCall,
    ToolSchema,
)

logger = logging.getLogger(__name__)


class MLXProvider(LLMProvider):
    """Non-streaming OpenAI-compatible client for a local mlx_server.

    mlx_server's streaming support is inconsistent across models, so this provider
    takes the simple route and issues a single ``stream=false`` request. It still
    honours the streaming protocol externally: on completion it emits the full text
    as a single ``text_delta`` event, followed by any tool calls, followed by the
    terminal ``finish`` event.
    """

    def __init__(
        self,
        *,
        base_url: str,
        model: str,
        http_client: httpx.AsyncClient,
        max_tokens: int = 4096,
    ) -> None:
        self._base_url = base_url.rstrip("/")
        self._model = model
        self._http = http_client
        self._max_tokens = max_tokens

    async def stream(
        self,
        messages: list[Message],
        tools: list[ToolSchema],
    ) -> AsyncIterator[ProviderEvent]:
        payload: dict[str, Any] = {
            "model": self._model,
            "messages": _to_openai_messages(messages),
            "max_tokens": self._max_tokens,
            "stream": False,
        }
        if tools:
            payload["tools"] = [
                {
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.input_schema,
                    },
                }
                for tool in tools
            ]

        url = f"{self._base_url}/chat/completions"
        try:
            response = await self._http.post(url, json=payload)
        except (httpx.ConnectError, httpx.TimeoutException) as exc:
            raise ProviderUnavailable(f"mlx endpoint unreachable: {exc}") from exc
        except httpx.HTTPError as exc:
            raise ProviderError(f"mlx request failed: {exc}") from exc

        if response.status_code >= 500:
            raise ProviderUnavailable(
                f"mlx server returned {response.status_code}: {response.text}"
            )
        if response.status_code >= 400:
            raise ProviderError(
                f"mlx server returned {response.status_code}: {response.text}"
            )

        body = response.json()
        choices = body.get("choices") or []
        if not choices:
            yield ProviderEvent(finish=True)
            return

        message = choices[0].get("message") or {}

        text = message.get("content") or ""
        if text:
            yield ProviderEvent(text_delta=text)

        for call in message.get("tool_calls") or []:
            function = call.get("function") or {}
            raw_args = function.get("arguments") or "{}"
            try:
                parsed_args = (
                    raw_args if isinstance(raw_args, dict) else json.loads(raw_args)
                )
            except json.JSONDecodeError:
                logger.warning("mlx tool call had non-JSON arguments: %r", raw_args)
                parsed_args = {}
            yield ProviderEvent(
                tool_call=ToolCall(
                    id=str(call.get("id", "")),
                    name=str(function.get("name", "")),
                    arguments=parsed_args if isinstance(parsed_args, dict) else {},
                )
            )

        yield ProviderEvent(finish=True)


def _to_openai_messages(messages: list[Message]) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for msg in messages:
        if msg.role == "user":
            out.append({"role": "user", "content": msg.content})
        elif msg.role == "assistant":
            entry: dict[str, Any] = {"role": "assistant", "content": msg.content or ""}
            if msg.tool_calls:
                entry["tool_calls"] = [
                    {
                        "id": call.id,
                        "type": "function",
                        "function": {
                            "name": call.name,
                            "arguments": json.dumps(call.arguments),
                        },
                    }
                    for call in msg.tool_calls
                ]
            out.append(entry)
        elif msg.role == "tool":
            out.append(
                {
                    "role": "tool",
                    "tool_call_id": msg.tool_call_id or "",
                    "content": msg.content,
                }
            )
    return out
