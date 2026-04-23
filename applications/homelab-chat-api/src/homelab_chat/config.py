"""Configuration loaded from environment variables."""

from __future__ import annotations

from enum import StrEnum
from functools import lru_cache

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class LLMProviderName(StrEnum):
    """Runtime-selectable LLM provider."""

    MLX = "mlx"
    ANTHROPIC = "anthropic"


class Settings(BaseSettings):
    """Service settings sourced from environment variables.

    Instantiated once per process through :func:`get_settings`.
    """

    model_config = SettingsConfigDict(
        env_file=None,
        case_sensitive=True,
        extra="ignore",
    )

    authentik_token_url: str = Field(alias="AUTHENTIK_TOKEN_URL")
    agent_client_id: str = Field(alias="AGENT_CLIENT_ID")
    agent_client_secret: str = Field(alias="AGENT_CLIENT_SECRET")

    homelab_api_mcp_url: str = Field(alias="HOMELAB_API_MCP_URL")
    homelab_settings_api_mcp_url: str = Field(alias="HOMELAB_SETTINGS_API_MCP_URL")

    llm_provider: LLMProviderName = Field(alias="LLM_PROVIDER")
    llm_model: str = Field(alias="LLM_MODEL")

    mlx_server_url: str | None = Field(default=None, alias="MLX_SERVER_URL")
    anthropic_api_key: str | None = Field(default=None, alias="ANTHROPIC_API_KEY")

    max_agent_iterations: int = Field(default=8, alias="MAX_AGENT_ITERATIONS")
    log_level: str = Field(default="INFO", alias="LOG_LEVEL")

    http_timeout_seconds: float = Field(default=30.0, alias="HTTP_TIMEOUT_SECONDS")


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    """Return the process-wide settings instance."""
    return Settings()  # ty: ignore[missing-argument]
