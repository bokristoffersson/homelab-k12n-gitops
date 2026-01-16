"""Command validation with RBAC and safety checks."""

import re
from pathlib import Path
from typing import Dict, Any, List

import yaml


class ValidationResult:
    """Result of command validation."""

    def __init__(self, valid: bool, reasons: List[str], modified_command: str = None):
        self.valid = valid
        self.reasons = reasons
        self.modified_command = modified_command

    def __repr__(self):
        status = "VALID" if self.valid else "INVALID"
        return f"ValidationResult({status}, reasons={self.reasons})"


class CommandValidator:
    """Validates kubectl commands against RBAC and safety rules."""

    def __init__(self, repo_root: Path = None):
        if repo_root is None:
            repo_root = Path(__file__).parent.parent

        # Load RBAC allowlist
        with open(repo_root / "org" / "rbac-allowlist.yaml", "r") as f:
            self.rbac = yaml.safe_load(f)

        # Load namespaces
        with open(repo_root / "org" / "namespaces.yaml", "r") as f:
            self.namespaces_config = yaml.safe_load(f)

    def validate(self, command: str, namespace: str = None) -> ValidationResult:
        """
        Validate a kubectl command.

        Args:
            command: The kubectl command to validate
            namespace: Expected namespace (optional)

        Returns:
            ValidationResult with validity status and reasons
        """
        reasons = []
        modified_command = command

        # Must start with kubectl, flux, or gh
        cmd_start = command.strip()
        if cmd_start.startswith("gh"):
            return ValidationResult(True, [])
        if not (cmd_start.startswith("kubectl") or cmd_start.startswith("flux")):
            return ValidationResult(False, ["Command must start with 'kubectl' or 'flux'"])

        # Extract verb and resource
        verb, resource = self._extract_verb_resource(command)

        # Check verb allowlist
        if verb and verb not in self.rbac["allowed_verbs"]:
            reasons.append(f"Verb '{verb}' not in allowlist: {self.rbac['allowed_verbs']}")

        # Check resource allowlist (if identifiable)
        if resource and resource not in self.rbac["allowed_resources"]:
            reasons.append(f"Resource '{resource}' not in allowlist: {self.rbac['allowed_resources']}")

        # Check namespace presence for namespace-scoped commands
        cluster_scoped_resources = ["node", "namespace", "persistentvolume", "storageclass", "clusterrole", "clusterrolebinding"]
        if self.rbac.get("namespace_scoped", True):
            has_namespace = "-n " in command or "--namespace" in command or "--namespace=" in command
            if not has_namespace and resource not in cluster_scoped_resources:
                reasons.append("Command must include namespace (-n or --namespace)")

        # Check for forbidden combinations
        for forbidden in self.rbac.get("forbidden_combinations", []):
            if verb == forbidden["verb"] and resource == forbidden["resource"]:
                reasons.append(f"Forbidden: {forbidden['reason']}")

        # Guardrail: replace dangerous delete pod with safer alternative
        if verb == "delete" and resource == "pod":
            modified_command = command.replace("delete pod", "# BLOCKED: Use 'rollout restart deployment' instead")
            reasons.append("Replaced 'delete pod' with safer alternative suggestion")

        # Valid if no reasons
        valid = len(reasons) == 0

        return ValidationResult(valid, reasons, modified_command if modified_command != command else None)

    def _extract_verb_resource(self, command: str) -> tuple:
        """Extract verb and resource from kubectl or flux command."""
        # Remove kubectl or flux prefix
        cmd = command.strip()
        if cmd.startswith("kubectl"):
            cmd = cmd.replace("kubectl", "", 1).strip()
        elif cmd.startswith("flux"):
            cmd = cmd.replace("flux", "", 1).strip()

        # Handle compound verbs (rollout restart, rollout status, etc.)
        if cmd.startswith("rollout"):
            parts = cmd.split()
            if len(parts) >= 2:
                verb = "rollout"
                # Extract resource from rest of command
                resource = self._extract_resource_from_parts(parts[2:])
                return verb, resource

        # Standard single-word verbs
        parts = cmd.split()
        if not parts:
            return None, None

        verb = parts[0]

        # Map kubectl verbs to normalized verbs
        verb_map = {
            "get": "get",
            "describe": "describe",
            "logs": "logs",
            "scale": "scale",
            "cordon": "cordon",
            "uncordon": "uncordon",
            "drain": "drain",
            "delete": "delete",
            "create": "create",
            "apply": "apply",
        }

        normalized_verb = verb_map.get(verb, verb)

        # Extract resource
        resource = self._extract_resource_from_parts(parts[1:])

        return normalized_verb, resource

    def _extract_resource_from_parts(self, parts: List[str]) -> str:
        """Extract resource type from command parts."""
        for part in parts:
            if part.startswith("-"):
                continue
            # Handle resource/name format
            if "/" in part:
                resource = part.split("/")[0]
                return resource
            # Common resource types
            resources = ["pod", "deployment", "statefulset", "daemonset", "node", "namespace",
                        "service", "ingress", "configmap", "secret", "event"]
            if part in resources or part.rstrip("s") in resources:
                return part.rstrip("s")

        return None


# CLI demo
if __name__ == "__main__":
    validator = CommandValidator()

    test_commands = [
        "kubectl rollout restart deployment/api -n prod",
        "kubectl logs my-pod -n staging --tail=50",
        "kubectl delete pod my-pod",
        "kubectl get pods",
        "kubectl scale deployment/api --replicas=5 -n prod",
        "rm -rf /",  # Not kubectl
    ]

    print("=== Command Validation Demo ===\n")

    for cmd in test_commands:
        result = validator.validate(cmd)
        print(f"Command: {cmd}")
        print(f"Result: {result}")
        if result.modified_command:
            print(f"Modified: {result.modified_command}")
        print()
