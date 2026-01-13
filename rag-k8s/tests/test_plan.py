"""Tests for MLX planner."""

import pytest
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from agent.plan import MLXPlanner


def test_planner_returns_dict():
    """Test that planner returns a dictionary."""
    pytest.skip("MLX model loading is slow - skip in CI")

    planner = MLXPlanner()
    plan = planner.plan_command(
        intent="restart",
        resource="deployment",
        namespace="prod",
        name="api",
        context="Template: kubectl rollout restart deployment/{{name}} -n {{namespace}}"
    )

    assert isinstance(plan, dict)
    assert "command" in plan


def test_plan_has_required_keys():
    """Test that plan has required keys."""
    pytest.skip("MLX model loading is slow - skip in CI")

    planner = MLXPlanner()
    plan = planner.plan_command(
        intent="logs",
        resource="pod",
        namespace="staging",
        name="worker-abc123",
        context=""
    )

    required_keys = ["intent", "namespace", "command", "summary"]
    for key in required_keys:
        assert key in plan
