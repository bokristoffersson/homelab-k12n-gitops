#!/usr/bin/env python3
"""Build FAISS index from command cards."""

import json
import os
from pathlib import Path
from typing import List, Dict, Any

import faiss
import numpy as np
import yaml
from sentence_transformers import SentenceTransformer


def load_cards(cards_dir: Path) -> List[Dict[str, Any]]:
    """Load all YAML command cards from directory."""
    cards = []
    for card_file in cards_dir.glob("*.yaml"):
        with open(card_file, "r") as f:
            card = yaml.safe_load(f)
            card["_source_file"] = str(card_file.relative_to(cards_dir.parent))
            cards.append(card)
    return cards


def card_to_searchable_text(card: Dict[str, Any]) -> str:
    """Convert card to searchable text representation."""
    parts = [
        f"ID: {card['id']}",
        f"Title: {card['title']}",
        f"Intent: {card['intent']}",
        f"Resource: {card['resource']}",
        f"Risk: {card['risk_level']}",
        f"Command: {card['command_template']}",
    ]

    if card.get("notes"):
        parts.append("Notes: " + " ".join(card["notes"]))

    if card.get("examples"):
        example_texts = []
        for ex in card["examples"]:
            example_texts.append(f"{ex['goal']}: {ex['render']['command']}")
        parts.append("Examples: " + " | ".join(example_texts))

    return "\n".join(parts)


def build_index(cards_dir: Path, output_dir: Path) -> None:
    """Build FAISS index and metadata from cards."""
    print(f"Loading cards from {cards_dir}")
    cards = load_cards(cards_dir)
    print(f"Loaded {len(cards)} cards")

    # Convert cards to searchable text
    texts = [card_to_searchable_text(card) for card in cards]

    # Load embedding model
    print("Loading embedding model...")
    model = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")

    # Generate embeddings
    print("Generating embeddings...")
    embeddings = model.encode(texts, normalize_embeddings=True, show_progress_bar=True)
    embeddings = embeddings.astype(np.float32)

    # Build FAISS index
    print("Building FAISS index...")
    dimension = embeddings.shape[1]
    index = faiss.IndexFlatIP(dimension)  # Inner product (cosine similarity with normalized vectors)
    index.add(embeddings)

    # Save index
    output_dir.mkdir(parents=True, exist_ok=True)
    index_path = output_dir / "faiss.index"
    faiss.write_index(index, str(index_path))
    print(f"Saved index to {index_path}")

    # Save metadata
    metadata = {
        "cards": [
            {
                "id": card["id"],
                "title": card["title"],
                "intent": card["intent"],
                "resource": card["resource"],
                "risk_level": card["risk_level"],
                "source_file": card["_source_file"],
                "command_template": card["command_template"],
            }
            for card in cards
        ],
        "count": len(cards),
        "dimension": dimension,
    }

    meta_path = output_dir / "meta.json"
    with open(meta_path, "w") as f:
        json.dump(metadata, f, indent=2)
    print(f"Saved metadata to {meta_path}")

    print(f"\nIndex built successfully:")
    print(f"  Cards: {len(cards)}")
    print(f"  Dimension: {dimension}")
    print(f"  Index size: {index.ntotal} vectors")


if __name__ == "__main__":
    repo_root = Path(__file__).parent.parent
    cards_dir = repo_root / "cards"
    output_dir = repo_root / "index"

    build_index(cards_dir, output_dir)
