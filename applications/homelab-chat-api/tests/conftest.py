"""Shared pytest fixtures."""

from __future__ import annotations

from collections.abc import AsyncIterator

import httpx
import pytest_asyncio


@pytest_asyncio.fixture
async def http_client() -> AsyncIterator[httpx.AsyncClient]:
    """Fresh httpx AsyncClient for tests that mount respx."""
    async with httpx.AsyncClient() as client:
        yield client
