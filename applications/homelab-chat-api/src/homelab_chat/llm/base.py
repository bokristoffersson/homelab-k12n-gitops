"""Provider-agnostic message and tool types plus the LLMProvider protocol."""

from __future__ import annotations

from collections.abc import AsyncIterator
from dataclasses import dataclass, field
from typing import Any, Literal, Protocol

Role = Literal["user", "assistant", "tool"]


@dataclass(slots=True)
class ToolCall:
    """A tool invocation requested by the model."""

    id: str
    name: str
    arguments: dict[str, Any]


@dataclass(slots=True)
class Message:
    """A single turn in a conversation.

    ``tool_calls`` is populated on assistant turns that requested tools; ``tool_call_id``
    and ``name`` identify the request on a tool-result turn.
    """

    role: Role
    content: str = ""
    tool_calls: list[ToolCall] = field(default_factory=list)
    tool_call_id: str | None = None
    name: str | None = None


@dataclass(slots=True)
class ToolSchema:
    """A tool description passed to the LLM."""

    name: str
    description: str
    input_schema: dict[str, Any]


@dataclass(slots=True)
class ProviderEvent:
    """Streaming event emitted by a provider.

    Exactly one of ``text_delta``, ``tool_call``, or ``finish`` is populated.
    """

    text_delta: str | None = None
    tool_call: ToolCall | None = None
    finish: bool = False


class ProviderError(RuntimeError):
    """Raised when the provider call fails for a reason the caller may want to handle."""


class ProviderUnavailable(ProviderError):
    """The provider endpoint is unreachable or returned a server error."""


class LLMProvider(Protocol):
    """An LLM backend capable of streaming tool-calling completions."""

    def stream(
        self,
        messages: list[Message],
        tools: list[ToolSchema],
    ) -> AsyncIterator[ProviderEvent]:
        """Stream a single completion turn.

        Implementations are async generators, so calling this method returns an
        :class:`AsyncIterator` directly. The iterator yields :class:`ProviderEvent`
        instances until the turn completes; the final event must have ``finish=True``.
        """
        ...
