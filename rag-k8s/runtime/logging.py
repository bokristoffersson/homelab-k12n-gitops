"""Logging and observability for agent operations."""

import json
import uuid
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional


class AgentLogger:
    """Logger for agent operations."""

    def __init__(self, log_dir: Path = None):
        if log_dir is None:
            log_dir = Path(__file__).parent.parent / "logs"

        self.log_dir = log_dir
        self.log_dir.mkdir(parents=True, exist_ok=True)
        self.log_file = self.log_dir / "agent.log"

    def log_action(
        self,
        operation_id: str,
        plan_json: Dict[str, Any],
        cmd: str,
        result: Dict[str, Any],
        level: str = "INFO"
    ) -> None:
        """
        Log an agent action.

        Args:
            operation_id: Unique operation identifier
            plan_json: The generated plan
            cmd: Executed command
            result: Execution result
            level: Log level (INFO, WARNING, ERROR)
        """
        log_entry = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "level": level,
            "operation_id": operation_id,
            "intent": plan_json.get("intent"),
            "namespace": plan_json.get("namespace"),
            "target": plan_json.get("target"),
            "command": cmd,
            "exit_code": result.get("code"),
            "duration": result.get("duration"),
            "stdout_digest": self._digest(result.get("stdout", "")),
            "stderr_digest": self._digest(result.get("stderr", "")),
            "truncated": result.get("truncated", False),
            "timeout": result.get("timeout", False),
        }

        # Append to JSONL
        with open(self.log_file, "a") as f:
            f.write(json.dumps(log_entry) + "\n")

    def _digest(self, text: str, max_len: int = 200) -> str:
        """Create a digest of text for logging."""
        if not text:
            return ""

        # Take first and last portions
        if len(text) <= max_len:
            return text

        half = max_len // 2
        return text[:half] + " [...] " + text[-half:]

    def generate_operation_id(self) -> str:
        """Generate unique operation ID."""
        return str(uuid.uuid4())


# Convenience function
def log_action(
    operation_id: str,
    plan_json: Dict[str, Any],
    cmd: str,
    result: Dict[str, Any],
    level: str = "INFO"
) -> None:
    """Log an agent action using default logger."""
    logger = AgentLogger()
    logger.log_action(operation_id, plan_json, cmd, result, level)


# CLI demo
if __name__ == "__main__":
    logger = AgentLogger()

    # Example log entry
    operation_id = logger.generate_operation_id()
    plan = {
        "intent": "restart",
        "namespace": "prod",
        "target": "deployment/api",
        "command": "kubectl rollout restart deployment/api -n prod",
        "summary": "Restart api deployment in prod",
    }

    result = {
        "code": 0,
        "stdout": "deployment.apps/api restarted",
        "stderr": "",
        "duration": 1.23,
        "truncated": False,
    }

    logger.log_action(operation_id, plan, plan["command"], result)

    print(f"Logged operation: {operation_id}")
    print(f"Log file: {logger.log_file}")
