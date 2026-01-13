"""Semantic search over command cards."""

import json
from pathlib import Path
from typing import List, Dict, Any, Optional

import faiss
import numpy as np
from sentence_transformers import SentenceTransformer


class Hit:
    """Search result hit."""

    def __init__(self, card_meta: Dict[str, Any], score: float):
        self.id = card_meta["id"]
        self.title = card_meta["title"]
        self.intent = card_meta["intent"]
        self.resource = card_meta["resource"]
        self.risk_level = card_meta["risk_level"]
        self.command_template = card_meta["command_template"]
        self.source_file = card_meta["source_file"]
        self.score = score

    def __repr__(self):
        return f"Hit(id={self.id}, score={self.score:.3f})"


class CardSearch:
    """Semantic search engine for command cards."""

    def __init__(self, index_dir: Path):
        self.index_dir = index_dir
        self.index = faiss.read_index(str(index_dir / "faiss.index"))

        with open(index_dir / "meta.json", "r") as f:
            self.metadata = json.load(f)

        self.model = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")

    def semantic_search(self, query: str, k: int = 6) -> List[Hit]:
        """Perform semantic search."""
        # Embed query
        query_embedding = self.model.encode([query], normalize_embeddings=True)
        query_embedding = query_embedding.astype(np.float32)

        # Search
        scores, indices = self.index.search(query_embedding, k)

        # Build hits
        hits = []
        for score, idx in zip(scores[0], indices[0]):
            if idx == -1:  # FAISS returns -1 for invalid indices
                continue
            card_meta = self.metadata["cards"][idx]
            hits.append(Hit(card_meta, float(score)))

        return hits

    def keyword_filter(self, hits: List[Hit], intent: Optional[str] = None,
                      resource: Optional[str] = None) -> List[Hit]:
        """Filter hits by intent and/or resource."""
        filtered = hits

        if intent:
            filtered = [h for h in filtered if h.intent == intent]

        if resource:
            filtered = [h for h in filtered if h.resource == resource]

        return filtered

    def search(self, query: str, k: int = 6, intent: Optional[str] = None,
              resource: Optional[str] = None) -> List[Hit]:
        """Search with optional keyword filtering."""
        # Semantic search with higher k to allow for filtering
        hits = self.semantic_search(query, k=k * 2)

        # Apply filters if specified
        if intent or resource:
            filtered = self.keyword_filter(hits, intent, resource)
            # If filtering removed too many, fall back to top semantic hits
            if len(filtered) < 2:
                filtered = hits[:k]
            else:
                filtered = filtered[:k]
            return filtered

        return hits[:k]

    def render_hit_snippet(self, hit: Hit) -> str:
        """Render a compact snippet for a hit."""
        return f"""Card: {hit.id}
Title: {hit.title}
Intent: {hit.intent} | Resource: {hit.resource} | Risk: {hit.risk_level}
Template: {hit.command_template}
Score: {hit.score:.3f}
"""


if __name__ == "__main__":
    # Demo CLI
    import sys

    repo_root = Path(__file__).parent.parent
    index_dir = repo_root / "index"

    if not (index_dir / "faiss.index").exists():
        print("Index not found. Run: make build-index")
        sys.exit(1)

    searcher = CardSearch(index_dir)

    if len(sys.argv) > 1:
        query = " ".join(sys.argv[1:])
    else:
        query = "restart deployment safely"

    print(f"Query: {query}\n")
    hits = searcher.search(query, k=4)

    for hit in hits:
        print(searcher.render_hit_snippet(hit))
        print("-" * 60)
