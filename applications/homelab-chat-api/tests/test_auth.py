"""Token cache behavior: first fetch, reuse within TTL, refetch past TTL."""

from __future__ import annotations

import httpx
import pytest
import respx

from homelab_chat.auth import AgentTokenProvider

TOKEN_URL = "https://authentik.example/application/o/token/"


class FakeClock:
    def __init__(self, start: float = 0.0) -> None:
        self._now = start

    def __call__(self) -> float:
        return self._now

    def advance(self, seconds: float) -> None:
        self._now += seconds


@pytest.mark.asyncio
async def test_first_call_fetches_and_second_reuses_token() -> None:
    clock = FakeClock()
    async with httpx.AsyncClient() as http_client, respx.mock() as mock:
        route = mock.post(TOKEN_URL).mock(
            return_value=httpx.Response(
                200,
                json={"access_token": "token-one", "expires_in": 3600, "token_type": "bearer"},
            )
        )
        provider = AgentTokenProvider(
            token_url=TOKEN_URL,
            client_id="agent",
            client_secret="secret",
            http_client=http_client,
            clock=clock,
        )

        first = await provider.get_token()
        second = await provider.get_token()

        assert first == "token-one"
        assert second == "token-one"
        assert route.call_count == 1


@pytest.mark.asyncio
async def test_refetches_after_effective_ttl_elapses() -> None:
    clock = FakeClock()
    async with httpx.AsyncClient() as http_client, respx.mock() as mock:
        route = mock.post(TOKEN_URL).mock(
            side_effect=[
                httpx.Response(
                    200,
                    json={
                        "access_token": "token-one",
                        "expires_in": 600,
                        "token_type": "bearer",
                    },
                ),
                httpx.Response(
                    200,
                    json={
                        "access_token": "token-two",
                        "expires_in": 600,
                        "token_type": "bearer",
                    },
                ),
            ]
        )
        provider = AgentTokenProvider(
            token_url=TOKEN_URL,
            client_id="agent",
            client_secret="secret",
            http_client=http_client,
            clock=clock,
        )

        first = await provider.get_token()
        # expires_in=600, safety_margin=300 -> effective_ttl=300; advance past it.
        clock.advance(301)
        second = await provider.get_token()

        assert first == "token-one"
        assert second == "token-two"
        assert route.call_count == 2


@pytest.mark.asyncio
async def test_raises_on_authentik_error() -> None:
    async with httpx.AsyncClient() as http_client, respx.mock() as mock:
        mock.post(TOKEN_URL).mock(
            return_value=httpx.Response(401, json={"error": "invalid_client"})
        )
        provider = AgentTokenProvider(
            token_url=TOKEN_URL,
            client_id="agent",
            client_secret="secret",
            http_client=http_client,
        )

        with pytest.raises(RuntimeError, match="Authentik token request failed"):
            await provider.get_token()
