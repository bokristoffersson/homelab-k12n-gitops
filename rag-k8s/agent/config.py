"""Configuration for MLX planner and performance settings."""

from pathlib import Path


# Model configuration
MODEL_ID = "mlx-community/Mistral-7B-Instruct-v0.3-4bit"

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
