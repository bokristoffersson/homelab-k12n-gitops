"""Tests for index building and search."""

import pytest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from index.build_index import load_cards, card_to_searchable_text
from index.search import CardSearch


def test_load_cards():
    """Test loading cards from directory."""
    repo_root = Path(__file__).parent.parent
    cards = load_cards(repo_root / "cards")

    assert len(cards) >= 15
    assert all("id" in card for card in cards)
    assert all("intent" in card for card in cards)


def test_card_to_searchable_text():
    """Test card conversion to searchable text."""
    card = {
        "id": "test-card",
        "title": "Test Card",
        "intent": "diagnose",
        "resource": "pod",
        "risk_level": "low",
        "command_template": "kubectl get pod {{name}} -n {{namespace}}",
        "notes": ["Note 1", "Note 2"],
        "examples": [
            {"goal": "Test goal", "render": {"command": "kubectl get pod test -n default"}}
        ]
    }

    text = card_to_searchable_text(card)

    assert "test-card" in text
    assert "diagnose" in text
    assert "kubectl get pod" in text


def test_search_returns_hits():
    """Test that search returns expected hit structure."""
    repo_root = Path(__file__).parent.parent
    index_dir = repo_root / "index"

    # Skip if index not built
    if not (index_dir / "faiss.index").exists():
        pytest.skip("Index not built")

    searcher = CardSearch(index_dir)
    hits = searcher.search("restart deployment", k=3)

    assert len(hits) > 0
    assert all(hasattr(hit, "id") for hit in hits)
    assert all(hasattr(hit, "score") for hit in hits)
    assert all(hasattr(hit, "command_template") for hit in hits)


def test_search_with_filters():
    """Test search with intent/resource filters."""
    repo_root = Path(__file__).parent.parent
    index_dir = repo_root / "index"

    if not (index_dir / "faiss.index").exists():
        pytest.skip("Index not built")

    searcher = CardSearch(index_dir)
    hits = searcher.search("deployment", k=3, intent="restart", resource="deployment")

    assert len(hits) > 0
    # Should prioritize matching intent/resource
    if hits:
        assert hits[0].intent == "restart" or hits[0].resource == "deployment"
