"""K8s exec tool contract for upstream orchestrators."""

import json
from pathlib import Path
from typing import Dict, Any
import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from jsonschema import validate, ValidationError

from agent.retrieve import retrieve_k8s_help
from agent.plan import MLXPlanner
from runtime.validate import CommandValidator
from runtime.exec import safe_exec
from runtime.logging import AgentLogger


# Input schema
K8S_EXEC_SCHEMA = {
    "type": "object",
    "properties": {
        "intent": {
            "type": "string",
            "enum": ["diagnose", "restart", "scale", "logs", "status", "describe", "events", "top", "cordon", "uncordon", "drain", "flux-reconcile", "flux-suspend", "flux-resume", "flux-status", "job-restart", "config-view"]
        },
        "resource": {
            "type": "string",
            "enum": ["deployment", "pod", "statefulset", "node", "service", "namespace", "kustomization", "job", "configmap"]
        },
        "namespace": {"type": "string"},
        "name": {"type": "string"},
        "selector": {"type": "string"},
        "constraints": {
            "type": "object",
            "properties": {
                "confirm": {"type": "boolean"},
                "dryRun": {"type": "boolean"}
            }
        }
    },
    "required": ["intent", "resource", "namespace", "name"]
}


class K8sExecTool:
    """K8s execution tool for orchestrators."""

    def __init__(self, repo_root: Path = None):
        if repo_root is None:
            repo_root = Path(__file__).parent.parent

        self.repo_root = repo_root
        self.validator = CommandValidator(repo_root)
        self.logger = AgentLogger(repo_root / "logs")
        self.planner = None  # Lazy load

    def execute(self, payload: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute K8s operation from payload.

        Args:
            payload: Input conforming to K8S_EXEC_SCHEMA

        Returns:
            Result with operationId, plan, and execution result
        """
        # Validate input
        try:
            validate(instance=payload, schema=K8S_EXEC_SCHEMA)
        except ValidationError as e:
            return {
                "error": "Invalid input schema",
                "details": str(e)
            }

        # Extract parameters
        intent = payload["intent"]
        resource = payload["resource"]
        namespace = payload["namespace"]
        name = payload["name"]
        selector = payload.get("selector")
        constraints = payload.get("constraints", {})

        # Generate operation ID
        operation_id = self.logger.generate_operation_id()

        # Build query
        query = f"{intent} {resource} {name}"
        if selector:
            query += f" with selector {selector}"

        # Step 1: Retrieve context
        context = retrieve_k8s_help(intent, resource, query, k=4, repo_root=self.repo_root)

        # Step 2: Plan command
        if self.planner is None:
            from agent.config import get_model_id
            self.planner = MLXPlanner(model_id=get_model_id())

        plan = self.planner.plan_command(
            intent, resource, namespace, name, selector, context
        )

        # Step 3: Validate
        validation = self.validator.validate(plan["command"], namespace)

        if not validation.valid:
            # Log failed validation
            result = {
                "code": -1,
                "stdout": "",
                "stderr": f"Validation failed: {', '.join(validation.reasons)}",
                "duration": 0,
            }
            self.logger.log_action(operation_id, plan, plan["command"], result, level="ERROR")

            return {
                "operationId": operation_id,
                "plan": plan,
                "validation": {
                    "valid": False,
                    "reasons": validation.reasons
                },
                "result": None
            }

        # Step 4: Check confirmation requirement
        if constraints.get("confirm", False):
            # Return plan for user confirmation without executing
            return {
                "operationId": operation_id,
                "plan": plan,
                "validation": {
                    "valid": True,
                    "reasons": []
                },
                "status": "awaiting_confirmation",
                "message": "Command requires explicit confirmation before execution"
            }

        # Step 5: Execute (unless dry-run)
        if constraints.get("dryRun", False):
            # Simulate execution
            result = {
                "code": 0,
                "stdout": "[DRY RUN] Command would execute: " + plan["command"],
                "stderr": "",
                "duration": 0,
                "dryRun": True
            }
        else:
            result = safe_exec(plan["command"])

        # Step 6: Log
        level = "INFO" if result["code"] == 0 else "ERROR"
        self.logger.log_action(operation_id, plan, plan["command"], result, level)

        # Return result
        return {
            "operationId": operation_id,
            "plan": plan,
            "validation": {
                "valid": True,
                "reasons": []
            },
            "result": {
                "code": result["code"],
                "stdoutDigest": self.logger._digest(result.get("stdout", "")),
                "stderrDigest": self.logger._digest(result.get("stderr", "")),
                "duration": result.get("duration", 0),
                "truncated": result.get("truncated", False),
            }
        }


def k8s_exec(payload: Dict[str, Any]) -> Dict[str, Any]:
    """
    Main entrypoint for k8s_exec tool.

    Args:
        payload: Input conforming to K8S_EXEC_SCHEMA

    Returns:
        Execution result
    """
    tool = K8sExecTool()
    return tool.execute(payload)


# CLI demo
if __name__ == "__main__":
    import sys

    if len(sys.argv) > 1:
        # Load payload from file
        payload_file = sys.argv[1]
        with open(payload_file, "r") as f:
            payload = json.load(f)
    else:
        # Default demo payload
        payload = {
            "intent": "restart",
            "resource": "deployment",
            "namespace": "prod",
            "name": "api",
            "constraints": {
                "confirm": False,
                "dryRun": True
            }
        }

    print("=== K8s Exec Tool Demo ===\n")
    print(f"Input payload:\n{json.dumps(payload, indent=2)}\n")

    result = k8s_exec(payload)

    print(f"Result:\n{json.dumps(result, indent=2)}")
