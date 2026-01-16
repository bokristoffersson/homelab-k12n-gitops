"""Configuration for MLX planner and performance settings."""

from pathlib import Path
import os


def _load_env_file(path: Path) -> None:
    """Load simple KEY=VALUE pairs from .env into process env (if unset)."""
    if not path.exists():
        return

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip('"').strip("'")
        os.environ.setdefault(key, value)


def _resolve_kubeconfig(raw_value: str, repo_root: Path) -> str:
    """Resolve KUBECONFIG entries to absolute paths."""
    paths = [p for p in raw_value.split(os.pathsep) if p]
    resolved = []
    for entry in paths:
        candidate = Path(entry).expanduser()
        if not candidate.is_absolute():
            candidate = (repo_root / candidate).resolve()
        resolved.append(str(candidate))
    return os.pathsep.join(resolved)


def _resolve_model_id(raw_value: str, repo_root: Path) -> str:
    """Resolve local path-like model IDs relative to repo root."""
    if not raw_value:
        return raw_value

    looks_like_path = raw_value.startswith((".", "/", "~"))
    candidate = Path(raw_value).expanduser()
    if not candidate.is_absolute():
        candidate = (repo_root / candidate).resolve()

    if looks_like_path or candidate.exists():
        return str(candidate)

    return raw_value


def _get_env_model_id(repo_root: Path) -> str | None:
    """Read model id from environment if present."""
    raw_value = os.environ.get("MODEL_ID")
    if not raw_value:
        return None

    resolved = _resolve_model_id(raw_value, repo_root)
    if raw_value.startswith((".", "/", "~")) and not Path(resolved).exists():
        print(
            f"MODEL_ID path not found: {resolved}. "
            f"Falling back to default model."
        )
        return None

    return resolved


def _default_model_id() -> str:
    return "mlx-community/Mistral-7B-Instruct-v0.3-4bit"


# Load .env from repo root and rag-k8s directory if present
_repo_root = Path(__file__).parent.parent
_rag_root = Path(__file__).parent
_load_env_file(_repo_root / ".env")
_load_env_file(_rag_root / ".env")

# Normalize KUBECONFIG if provided in .env
if "KUBECONFIG" in os.environ:
    os.environ["KUBECONFIG"] = _resolve_kubeconfig(
        os.environ["KUBECONFIG"], _repo_root
    )

# Model configuration
MODEL_ID = _get_env_model_id(_repo_root) or _default_model_id()

# Alternative models (uncomment to use):
# MODEL_ID = "mlx-community/Llama-3.2-3B-Instruct-4bit"  # Faster, lower quality
# MODEL_ID = "mlx-community/Llama-3.2-1B-Instruct-4bit"  # Faster, lower quality
# MODEL_ID = "mlx-community/Meta-Llama-3-8B-Instruct-4bit"  # Slower, higher quality

# Decoding parameters
DECODING_CONFIG = {
    "temperature": 0.0,  # Zero temperature for maximum determinism
    "top_p": 1.0,  # No nucleus sampling (greedy decoding)
    "max_tokens": 256,  # Sufficient for complete JSON output
    "repetition_penalty": 1.05,  # Light repetition penalty
}

# Context window
CONTEXT_WINDOW = 8192  # Default 8k tokens
# CONTEXT_WINDOW = 16384  # Optional: Use 16k for longer contexts

# Retrieval configuration
RETRIEVAL_CONFIG = {
    "k": 4,  # Number of cards to retrieve
    "max_context_tokens": 1200,  # Maximum total context size
    "min_score_threshold": 0.3,  # Minimum similarity score
}

# Execution configuration
EXECUTION_CONFIG = {
    "default_timeout": 15,  # Default command timeout in seconds
    "max_stdout_chars": 4000,  # Max stdout before truncation
    "max_stderr_chars": 2000,  # Max stderr before truncation
}

# Logging configuration
LOGGING_CONFIG = {
    "log_dir": "logs",
    "log_file": "agent.log",
    "max_size_mb": 100,
    "retention_days": 30,
}


def get_repo_root() -> Path:
    """Get repository root path."""
    return Path(__file__).parent.parent


def get_model_id() -> str:
    """Get configured model ID."""
    return MODEL_ID


def get_decoding_config() -> dict:
    """Get decoding configuration."""
    return DECODING_CONFIG.copy()


def get_retrieval_config() -> dict:
    """Get retrieval configuration."""
    return RETRIEVAL_CONFIG.copy()


def get_execution_config() -> dict:
    """Get execution configuration."""
    return EXECUTION_CONFIG.copy()
