"""Tests for retrieval API."""

import pytest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from agent.retrieve import retrieve_k8s_help


def test_retrieve_returns_string():
    """Test that retrieve returns a string."""
    repo_root = Path(__file__).parent.parent
    index_dir = repo_root / "index"

    if not (index_dir / "faiss.index").exists():
        pytest.skip("Index not built")

    result = retrieve_k8s_help("restart", "deployment", "safely restart deployment", k=2)

    assert isinstance(result, str)
    assert len(result) > 0


def test_retrieve_filters_by_intent_resource():
    """Test that retrieval filters by intent and resource."""
    repo_root = Path(__file__).parent.parent
    index_dir = repo_root / "index"

    if not (index_dir / "faiss.index").exists():
        pytest.skip("Index not built")

    result = retrieve_k8s_help("logs", "pod", "view pod logs", k=2)

    assert "logs" in result.lower()
    assert "pod" in result.lower()
