"""Agent loop with a fake LLM and a fake tool router."""

from __future__ import annotations

from collections.abc import AsyncIterator
from typing import Any

import pytest

from homelab_chat.agent import Agent, AgentConfig
from homelab_chat.llm.base import (
    LLMProvider,
    Message,
    ProviderEvent,
    ProviderUnavailable,
    ToolCall,
    ToolSchema,
)
from homelab_chat.mcp_client import MCPError


class FakeProvider(LLMProvider):
    """Replays a scripted sequence of event batches across successive stream() calls."""

    def __init__(self, batches: list[list[ProviderEvent]]) -> None:
        self._batches = list(batches)
        self.calls: list[list[Message]] = []

    async def stream(
        self,
        messages: list[Message],
        tools: list[ToolSchema],
    ) -> AsyncIterator[ProviderEvent]:
        self.calls.append(list(messages))
        if not self._batches:
            raise AssertionError("FakeProvider.stream called with no batches left")
        batch = self._batches.pop(0)
        for event in batch:
            yield event


class FakeRouter:
    """Implements the subset of ToolRouter the Agent needs."""

    def __init__(
        self,
        tool_names: list[str],
        responses: dict[str, Any],
        *,
        errors: dict[str, MCPError] | None = None,
    ) -> None:
        self._tools = [
            type(
                "T",
                (),
                {
                    "name": n,
                    "description": "",
                    "input_schema": {"type": "object", "properties": {}},
                },
            )()
            for n in tool_names
        ]
        self._responses = responses
        self._errors = errors or {}
        self.calls: list[tuple[str, dict[str, Any]]] = []

    async def ensure_loaded(self) -> list[Any]:
        return self._tools

    async def call(self, name: str, arguments: dict[str, Any]) -> Any:
        self.calls.append((name, arguments))
        if name in self._errors:
            raise self._errors[name]
        return self._responses.get(name)


@pytest.mark.asyncio
async def test_agent_runs_tool_then_returns_final_text() -> None:
    provider = FakeProvider(
        [
            [
                ProviderEvent(
                    tool_call=ToolCall(id="t1", name="energy_latest", arguments={})
                ),
                ProviderEvent(finish=True),
            ],
            [
                ProviderEvent(text_delta="Your power draw is 1.2 kW."),
                ProviderEvent(finish=True),
            ],
        ]
    )
    router = FakeRouter(
        tool_names=["energy_latest"],
        responses={"energy_latest": {"power_kw": 1.2}},
    )
    agent = Agent(
        primary_provider=provider,
        fallback_provider=None,
        tool_router=router,  # ty: ignore[invalid-argument-type]
        config=AgentConfig(max_iterations=4),
    )

    events = [e async for e in agent.run([Message(role="user", content="power?")])]

    event_types = [e.event for e in events]
    assert event_types == ["tool_call", "tool_result", "token", "done"]
    assert events[1].data == {
        "id": "t1",
        "name": "energy_latest",
        "result": {"power_kw": 1.2},
    }
    assert events[2].data == {"text": "Your power draw is 1.2 kW."}
    assert router.calls == [("energy_latest", {})]


@pytest.mark.asyncio
async def test_agent_stops_at_max_iterations_when_tool_loop_never_resolves() -> None:
    provider = FakeProvider(
        [
            [
                ProviderEvent(
                    tool_call=ToolCall(id=f"t{i}", name="energy_latest", arguments={})
                ),
                ProviderEvent(finish=True),
            ]
            for i in range(5)
        ]
    )
    router = FakeRouter(
        tool_names=["energy_latest"],
        responses={"energy_latest": {"power_kw": 1.0}},
    )
    agent = Agent(
        primary_provider=provider,
        fallback_provider=None,
        tool_router=router,  # ty: ignore[invalid-argument-type]
        config=AgentConfig(max_iterations=2),
    )

    events = [e async for e in agent.run([Message(role="user", content="x")])]
    done = events[-1]
    assert done.event == "done"
    assert done.data["reason"] == "max_iterations_reached"


@pytest.mark.asyncio
async def test_agent_emits_tool_result_error_when_mcp_call_fails() -> None:
    provider = FakeProvider(
        [
            [
                ProviderEvent(
                    tool_call=ToolCall(id="t1", name="broken", arguments={"a": 1})
                ),
                ProviderEvent(finish=True),
            ],
            [
                ProviderEvent(text_delta="Sorry, the tool failed."),
                ProviderEvent(finish=True),
            ],
        ]
    )
    router = FakeRouter(
        tool_names=["broken"],
        responses={},
        errors={"broken": MCPError(code=-32000, message="boom")},
    )
    agent = Agent(
        primary_provider=provider,
        fallback_provider=None,
        tool_router=router,  # ty: ignore[invalid-argument-type]
        config=AgentConfig(max_iterations=4),
    )

    events = [e async for e in agent.run([Message(role="user", content="x")])]
    tool_result = next(e for e in events if e.event == "tool_result")
    assert "error" in tool_result.data
    assert tool_result.data["error"]["code"] == -32000


@pytest.mark.asyncio
async def test_agent_falls_back_when_primary_unavailable() -> None:
    class UnavailableProvider(LLMProvider):
        async def stream(
            self,
            messages: list[Message],
            tools: list[ToolSchema],
        ) -> AsyncIterator[ProviderEvent]:
            raise ProviderUnavailable("mlx down")
            yield  # pragma: no cover - unreachable, makes this an async generator

    fallback = FakeProvider(
        [
            [
                ProviderEvent(text_delta="Hello from fallback."),
                ProviderEvent(finish=True),
            ]
        ]
    )
    router = FakeRouter(tool_names=[], responses={})
    agent = Agent(
        primary_provider=UnavailableProvider(),
        fallback_provider=fallback,
        tool_router=router,  # ty: ignore[invalid-argument-type]
        config=AgentConfig(max_iterations=4),
    )

    events = [e async for e in agent.run([Message(role="user", content="hi")])]
    token_event = next(e for e in events if e.event == "token")
    assert token_event.data == {"text": "Hello from fallback."}
    assert events[-1].event == "done"
