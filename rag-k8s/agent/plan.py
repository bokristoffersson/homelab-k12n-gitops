"""MLX-based command planner with strict JSON output."""

import json
import re
from typing import Dict, Any, Optional
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from mlx_lm import load, generate
from mlx_lm.sample_utils import make_sampler, make_logits_processors


# System prompt for strict JSON generation
SYSTEM_PROMPT = """You are a Kubernetes command generator. Output ONLY a valid JSON object.

Required keys: intent, namespace, target, selector, command, summary
Allowed verbs: get, describe, logs, scale, rollout restart, cordon, drain, uncordon
Always include '-n <namespace>' for namespaced resources.

Example output format:
{
  "intent": "restart",
  "namespace": "prod",
  "target": "deployment/api",
  "selector": null,
  "command": "kubectl rollout restart deployment/api -n prod",
  "summary": "Rolling restart of deployment/api in prod namespace"
}

Output ONLY valid JSON. No text before or after."""


class MLXPlanner:
    """MLX-based command planner."""

    def __init__(self, model_id: str = None, model=None, tokenizer=None):
        """Initialize planner with MLX model.
        
        Args:
            model_id: Model ID to load (if model/tokenizer not provided)
            model: Pre-loaded model (for server mode)
            tokenizer: Pre-loaded tokenizer (for server mode)
        """
        if model is not None and tokenizer is not None:
            # Use pre-loaded model (server mode)
            self.model = model
            self.tokenizer = tokenizer
            self.model_id = model_id or "pre-loaded"
        else:
            # Load model (standalone mode)
            if model_id is None:
                from agent.config import get_model_id
                model_id = get_model_id()
            print(f"Loading model: {model_id}")
            self.model, self.tokenizer = load(model_id)
            self.model_id = model_id

    def plan_command(
        self,
        intent: str,
        resource: str,
        namespace: str,
        name: str,
        selector: Optional[str] = None,
        context: str = ""
    ) -> Dict[str, Any]:
        """
        Generate a kubectl command plan.

        Args:
            intent: Operation intent (restart, diagnose, logs, etc.)
            resource: K8s resource type (deployment, pod, etc.)
            namespace: Target namespace
            name: Resource name
            selector: Optional label selector
            context: Retrieved context snippets from RAG

        Returns:
            Parsed JSON plan with keys: intent, namespace, target, selector, command, summary
        """
        # Build prompt
        user_goal = f"{intent} {resource} '{name}' in namespace '{namespace}'"
        if selector:
            user_goal += f" with selector '{selector}'"

        prompt_parts = [
            SYSTEM_PROMPT,
            "",
            "Context:",
            context if context else "(no context)",
            "",
            f"User goal: {user_goal}",
            "",
            "JSON:",
        ]

        prompt = "\n".join(prompt_parts)

        # Generate with strict settings for deterministic output
        sampler = make_sampler(temp=0.0, top_p=1.0)  # temp=0 for maximum determinism
        logits_processors = make_logits_processors(repetition_penalty=1.05)

        response = generate(
            self.model,
            self.tokenizer,
            prompt=prompt,
            max_tokens=256,  # Increased to allow full JSON completion
            sampler=sampler,
            logits_processors=logits_processors,
            verbose=False,
        )

        # Extract JSON from response
        plan = self._parse_json_response(response, prompt)

        return plan

    def _parse_json_response(self, response: str, original_prompt: str) -> Dict[str, Any]:
        """Parse JSON from model response, with retry on failure."""
        # Try to extract JSON object (handles nested braces)
        json_match = re.search(r'\{(?:[^{}]|(?:\{[^{}]*\}))*\}', response, re.DOTALL)

        if json_match:
            json_str = json_match.group(0)
            try:
                plan = json.loads(json_str)
                # Validate required keys
                required = ["intent", "namespace", "command", "summary"]
                if all(k in plan for k in required):
                    return plan
            except json.JSONDecodeError:
                pass

        # Retry with explicit correction prompt
        print("Invalid JSON detected. Retrying...")
        retry_prompt = original_prompt + "\n\nYour last output was invalid. Output ONLY this JSON structure:\n{\"intent\": \"...\", \"namespace\": \"...\", \"target\": \"...\", \"selector\": null, \"command\": \"...\", \"summary\": \"...\"}"

        retry_sampler = make_sampler(temp=0.0, top_p=1.0)
        retry_response = generate(
            self.model,
            self.tokenizer,
            prompt=retry_prompt,
            max_tokens=256,
            sampler=retry_sampler,
            verbose=False,
        )

        # Try again
        json_match = re.search(r'\{(?:[^{}]|(?:\{[^{}]*\}))*\}', retry_response, re.DOTALL)
        if json_match:
            json_str = json_match.group(0)
            try:
                plan = json.loads(json_str)
                required = ["intent", "namespace", "command", "summary"]
                if all(k in plan for k in required):
                    return plan
            except json.JSONDecodeError:
                pass

        # Fallback: construct minimal valid plan
        return {
            "intent": "unknown",
            "namespace": "default",
            "command": "# Failed to generate valid command",
            "summary": "JSON generation failed",
            "error": "Could not parse valid JSON from model output"
        }


# Demo
if __name__ == "__main__":
    """
    Demo usage:
        python -m agent.plan

    This will demonstrate the planner with a sample task.
    """
    from agent.retrieve import retrieve_k8s_help

    print("=== MLX Planner Demo ===\n")

    # Example task
    intent = "restart"
    resource = "deployment"
    namespace = "prod"
    name = "api"

    print(f"Task: {intent} {resource}/{name} in {namespace}\n")

    # Retrieve context
    print("Retrieving context...")
    context = retrieve_k8s_help(intent, resource, f"safely {intent} {resource}", k=2)
    print(f"Context length: {len(context)} chars\n")

    # Plan command
    print("Planning command...")
    planner = MLXPlanner()
    plan = planner.plan_command(intent, resource, namespace, name, context=context)

    print("\n=== Generated Plan ===")
    print(json.dumps(plan, indent=2))
