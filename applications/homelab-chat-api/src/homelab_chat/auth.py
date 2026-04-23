"""Authentik client_credentials token acquisition with in-process caching."""

from __future__ import annotations

import asyncio
import logging
import time
from collections.abc import Callable
from dataclasses import dataclass

import httpx

logger = logging.getLogger(__name__)

# Refresh this many seconds before the actual expiry to avoid races at the boundary.
_EXPIRY_SAFETY_MARGIN_SECONDS = 300


@dataclass(frozen=True, slots=True)
class CachedToken:
    """An access token with the absolute monotonic time at which it expires."""

    access_token: str
    expires_at_monotonic: float


class AgentTokenProvider:
    """Fetches and caches client_credentials tokens from Authentik.

    The provider guarantees at most one concurrent token fetch per process via an
    asyncio lock. Tokens are reused until ``_EXPIRY_SAFETY_MARGIN_SECONDS`` before
    the server-reported expiry, at which point the next call refetches.
    """

    def __init__(
        self,
        *,
        token_url: str,
        client_id: str,
        client_secret: str,
        http_client: httpx.AsyncClient,
        clock: Callable[[], float] | None = None,
    ) -> None:
        self._token_url = token_url
        self._client_id = client_id
        self._client_secret = client_secret
        self._http = http_client
        self._clock = clock or time.monotonic
        self._cached: CachedToken | None = None
        self._lock = asyncio.Lock()

    async def get_token(self) -> str:
        """Return a valid bearer token, fetching a fresh one if needed."""
        cached = self._cached
        if cached is not None and self._clock() < cached.expires_at_monotonic:
            return cached.access_token

        async with self._lock:
            # Re-check after acquiring the lock: another task may have refreshed.
            cached = self._cached
            if cached is not None and self._clock() < cached.expires_at_monotonic:
                return cached.access_token

            token = await self._fetch_new_token()
            self._cached = token
            return token.access_token

    async def _fetch_new_token(self) -> CachedToken:
        response = await self._http.post(
            self._token_url,
            data={
                "grant_type": "client_credentials",
                "client_id": self._client_id,
                "client_secret": self._client_secret,
            },
            headers={"Content-Type": "application/x-www-form-urlencoded"},
        )
        if response.status_code >= 400:
            raise RuntimeError(
                f"Authentik token request failed: {response.status_code} {response.text}"
            )
        payload = response.json()

        access_token = payload.get("access_token")
        expires_in = payload.get("expires_in")
        if not isinstance(access_token, str) or not isinstance(expires_in, int):
            raise RuntimeError(
                "Authentik token response missing access_token or expires_in "
                f"(payload keys: {sorted(payload)})"
            )

        effective_ttl = max(expires_in - _EXPIRY_SAFETY_MARGIN_SECONDS, 1)
        expires_at = self._clock() + effective_ttl
        logger.info(
            "fetched agent token",
            extra={"expires_in": expires_in, "effective_ttl_seconds": effective_ttl},
        )
        return CachedToken(access_token=access_token, expires_at_monotonic=expires_at)
