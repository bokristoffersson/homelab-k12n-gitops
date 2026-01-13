"""Tests for command validation."""

import pytest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from runtime.validate import CommandValidator


def test_valid_command_passes():
    """Test that valid commands pass validation."""
    validator = CommandValidator()

    result = validator.validate("kubectl rollout restart deployment/api -n prod")

    assert result.valid
    assert len(result.reasons) == 0


def test_missing_namespace_fails():
    """Test that commands without namespace fail (for namespaced resources)."""
    validator = CommandValidator()

    result = validator.validate("kubectl get pods")

    assert not result.valid
    assert any("namespace" in reason.lower() for reason in result.reasons)


def test_forbidden_verb_fails():
    """Test that forbidden verbs fail validation."""
    validator = CommandValidator()

    # 'apply' is not in the allowlist
    result = validator.validate("kubectl apply -f manifest.yaml -n prod")

    assert not result.valid


def test_delete_pod_blocked():
    """Test that delete pod is blocked."""
    validator = CommandValidator()

    result = validator.validate("kubectl delete pod my-pod -n prod")

    assert not result.valid
    assert any("delete" in reason.lower() for reason in result.reasons)


def test_non_kubectl_command_fails():
    """Test that non-kubectl commands fail."""
    validator = CommandValidator()

    result = validator.validate("rm -rf /tmp")

    assert not result.valid
    assert any("kubectl" in reason.lower() for reason in result.reasons)
