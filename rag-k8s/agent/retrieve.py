"""Retrieval API for K8s command help."""

import yaml
from pathlib import Path
from typing import Optional

import typer
from rich.console import Console
from rich.panel import Panel

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from index.search import CardSearch


console = Console()


def load_card_full(source_file: str, repo_root: Path) -> dict:
    """Load full card YAML from source file."""
    card_path = repo_root / source_file
    with open(card_path, "r") as f:
        return yaml.safe_load(f)


def render_compact_snippet(hit, full_card: dict) -> str:
    """Render compact context snippet for a card."""
    lines = [
        f"Card: {hit.id}",
        f"Template: {hit.command_template}",
    ]

    # Add 1-2 examples
    if full_card.get("examples"):
        lines.append("Examples:")
        for ex in full_card["examples"][:2]:
            lines.append(f"  - {ex['goal']}: {ex['render']['command']}")

    # Add 2-3 key notes
    if full_card.get("notes"):
        lines.append("Notes:")
        for note in full_card["notes"][:3]:
            lines.append(f"  - {note}")

    return "\n".join(lines)


def retrieve_k8s_help(
    intent: str,
    resource: str,
    query: str,
    k: int = 4,
    repo_root: Optional[Path] = None
) -> str:
    """
    Retrieve relevant K8s command help.

    Args:
        intent: Operation intent (diagnose, restart, logs, etc.)
        resource: K8s resource type (deployment, pod, node, etc.)
        query: Natural language query
        k: Number of results to return
        repo_root: Repository root path (auto-detected if None)

    Returns:
        Compact context snippets as a single string
    """
    if repo_root is None:
        repo_root = Path(__file__).parent.parent

    index_dir = repo_root / "index"

    # Initialize searcher
    searcher = CardSearch(index_dir)

    # Search with filters
    hits = searcher.search(query, k=k * 2, intent=intent, resource=resource)

    # If no filtered hits, fall back to pure semantic
    if not hits:
        hits = searcher.semantic_search(query, k=k)

    # Limit to top k
    hits = hits[:k]

    # Load full cards and render snippets
    snippets = []
    for hit in hits:
        full_card = load_card_full(hit.source_file, repo_root)
        snippet = render_compact_snippet(hit, full_card)
        snippets.append(snippet)

    return "\n\n---\n\n".join(snippets)


# CLI
app = typer.Typer()


@app.command()
def main(
    intent: str = typer.Option(..., "--intent", "-i", help="Operation intent"),
    resource: str = typer.Option(..., "--resource", "-r", help="K8s resource type"),
    query: str = typer.Option(..., "--query", "-q", help="Natural language query"),
    k: int = typer.Option(4, "--k", help="Number of results"),
):
    """Retrieve K8s command help snippets."""
    context = retrieve_k8s_help(intent, resource, query, k)

    console.print(Panel(
        context,
        title=f"[bold cyan]Context for: {intent} {resource}[/bold cyan]",
        border_style="cyan"
    ))


if __name__ == "__main__":
    app()
