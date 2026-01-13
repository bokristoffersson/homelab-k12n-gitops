"""Tests for command execution."""

import pytest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from runtime.exec import safe_exec


def test_exec_returns_result():
    """Test that exec returns a result dict."""
    result = safe_exec("echo 'test'")

    assert "code" in result
    assert "stdout" in result
    assert "stderr" in result
    assert "duration" in result


def test_successful_command():
    """Test successful command execution."""
    result = safe_exec("echo 'hello'")

    assert result["code"] == 0
    assert "hello" in result["stdout"]


def test_timeout_handling():
    """Test that long commands timeout."""
    result = safe_exec("sleep 30", timeout=1)

    assert result["code"] == -1
    assert "timeout" in result.get("stderr", "").lower() or result.get("timeout", False)


def test_output_truncation():
    """Test that large output is truncated."""
    # Generate >4000 chars of output
    long_cmd = "python3 -c \"print('x' * 5000)\""
    result = safe_exec(long_cmd)

    assert result.get("truncated", False) or len(result["stdout"]) <= 4100
