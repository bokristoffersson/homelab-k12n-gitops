# Prompts Guide

## Claude Code Integration

### Master Controller Prompt

This repository was generated using a master controller prompt that orchestrates multi-stage generation. See the root-level controller prompt for details.

### Using k8s_exec from Claude Code

When Claude Code needs to perform Kubernetes operations, it can call the `k8s_exec` tool:

```python
from rag_k8s.agent.tool import k8s_exec

# Example: Restart a deployment
result = k8s_exec({
    "intent": "restart",
    "resource": "deployment",
    "namespace": "prod",
    "name": "api",
    "constraints": {
        "confirm": False,
        "dryRun": False
    }
})

# Check result
if result.get("result"):
    print(f"Command executed: {result['plan']['command']}")
    print(f"Exit code: {result['result']['code']}")
else:
    print(f"Validation failed: {result['validation']['reasons']}")
```

### Input Schema

```json
{
  "intent": "<diagnose|restart|scale|logs|status|describe|events|top|cordon|uncordon|drain>",
  "resource": "<deployment|pod|statefulset|node|service|namespace>",
  "namespace": "<namespace-name>",
  "name": "<resource-name>",
  "selector": "<optional-label-selector>",
  "constraints": {
    "confirm": false,
    "dryRun": false
  }
}
```

### Output Schema

```json
{
  "operationId": "<uuid>",
  "plan": {
    "intent": "<intent>",
    "namespace": "<namespace>",
    "target": "<resource-type/name>",
    "command": "<kubectl-command>",
    "summary": "<human-readable-summary>"
  },
  "validation": {
    "valid": true,
    "reasons": []
  },
  "result": {
    "code": 0,
    "stdoutDigest": "<truncated-stdout>",
    "stderrDigest": "<truncated-stderr>",
    "duration": 1.23,
    "truncated": false
  }
}
```

## Troubleshooting Prompts

### Invalid JSON Re-emit

If the MLX planner generates invalid JSON, it automatically retries with this correction prompt:

```
Your last output was invalid JSON. Output valid JSON only.
```

The planner uses strict temperature (0.2) and constrained max_tokens (160) to minimize hallucination.

### Unsafe Command Rejection

If a command violates RBAC or safety policies, the tool returns:

```json
{
  "validation": {
    "valid": false,
    "reasons": [
      "Forbidden: Use 'rollout restart deployment' instead"
    ]
  },
  "result": null
}
```

Claude should interpret this and either:
1. Adjust the payload with a safer alternative
2. Ask the user for clarification

### Confirmation Required

High-risk operations (e.g., drain) return:

```json
{
  "status": "awaiting_confirmation",
  "message": "Command requires explicit confirmation before execution",
  "plan": { ... }
}
```

Claude should present the plan to the user and wait for explicit approval before re-calling with `confirm: true`.

## Example Workflows

### Diagnose CrashLoopBackOff

```python
# 1. Describe the pod
result = k8s_exec({
    "intent": "diagnose",
    "resource": "pod",
    "namespace": "prod",
    "name": "api-7d8f6c9b-xk2p9",
    "constraints": {"dryRun": False}
})

# 2. If needed, get previous logs
logs_result = k8s_exec({
    "intent": "logs",
    "resource": "pod",
    "namespace": "prod",
    "name": "api-7d8f6c9b-xk2p9",
    "constraints": {"dryRun": False}
})
```

### Safe Deployment Restart

```python
# Dry-run first to validate
dry_result = k8s_exec({
    "intent": "restart",
    "resource": "deployment",
    "namespace": "prod",
    "name": "api",
    "constraints": {"dryRun": True}
})

# If valid, execute
if dry_result["validation"]["valid"]:
    result = k8s_exec({
        "intent": "restart",
        "resource": "deployment",
        "namespace": "prod",
        "name": "api",
        "constraints": {"dryRun": False}
    })
```

### Node Maintenance (Cordon + Drain)

```python
# 1. Cordon the node
cordon_result = k8s_exec({
    "intent": "cordon",
    "resource": "node",
    "namespace": "",  # Nodes are cluster-scoped
    "name": "p0.local",
    "constraints": {"confirm": True}  # Requires confirmation
})

# 2. If user confirms, drain
drain_result = k8s_exec({
    "intent": "drain",
    "resource": "node",
    "namespace": "",
    "name": "p0.local",
    "constraints": {"confirm": True}  # High-risk, requires confirmation
})
```

## Best Practices

1. **Always dry-run first** for destructive operations
2. **Check validation results** before presenting output to user
3. **Log operation IDs** for troubleshooting
4. **Handle confirmation flow** for high-risk operations
5. **Interpret result codes**: 0 = success, non-zero = error
