# Claude Code Integration with RAG-K8S

## Overview

This document describes how to configure Claude Code to automatically use the RAG-K8S agent for Kubernetes and GitHub operations, ensuring safe command execution with RBAC validation and semantic search.

## Forcing Claude to Always Use RAG-K8S

### Method 1: Project Instructions (CLAUDE.md)

Add the following section to your `/CLAUDE.md` file to instruct Claude Code to use RAG-K8S for all relevant operations:

```markdown
## RAG-K8S Agent - MANDATORY USAGE

**CRITICAL**: You MUST use the RAG-K8S agent for ALL Kubernetes and GitHub CLI operations. Direct kubectl or gh commands are ONLY allowed for:
1. Simple read-only queries (get namespaces, list resources)
2. When the agent explicitly fails or is unavailable

### When to Use RAG-K8S

Use the agent via Python for these operations:

**Kubernetes Operations:**
- Restarting deployments → intent: "restart"
- Viewing logs → intent: "logs"
- Diagnosing pod issues → intent: "diagnose"
- Scaling resources → intent: "scale"
- Checking rollout status → intent: "status"
- FluxCD reconciliation → intent: "flux-reconcile"

**GitHub Operations:**
- Watching workflow runs → intent: "gh-run-watch"
- Listing workflow runs → intent: "gh-run-list"
- Viewing run details → intent: "gh-run-view"
- Listing pull requests → intent: "gh-pr-list"
- Viewing PR details → intent: "gh-pr-view"

### Required Workflow

1. **ALWAYS** start with `dryRun: True` to preview the command
2. Show the user what will be executed
3. **NEVER** execute with `dryRun: False` without user confirmation
4. Check `result['validation']['valid']` before executing

### Example Usage

```python
import sys
sys.path.insert(0, '/path/to/rag-k8s')
from agent.tool import k8s_exec

# DRY RUN FIRST
result = k8s_exec({
    "intent": "restart",
    "resource": "deployment",
    "namespace": "heatpump-settings",
    "name": "heatpump-settings-api",
    "constraints": {"dryRun": True}
})

print(f"Would execute: {result['plan']['command']}")

# After user confirms:
# result = k8s_exec({...same..., "constraints": {"dryRun": False}})
```

### Error Recovery

If the agent fails with "namespace not found", use kubectl directly to discover the correct namespace, then retry with the agent using the correct name.
```

### Method 2: Cursor Rules (.cursorrules)

If using Cursor IDE, add RAG-K8S rules to your project's `.cursorrules` file (already configured in this project).

### Method 3: Session Instructions

At the start of each Claude Code session, remind Claude:

```
For all Kubernetes and GitHub operations, use the RAG-K8S agent with dry-run first.
```

## Available Operations

### Kubernetes Operations

| Intent | Resource | Description | Risk Level |
|--------|----------|-------------|------------|
| `restart` | `deployment` | Rolling restart | low |
| `logs` | `pod` | View pod logs | low |
| `diagnose` | `pod` | Diagnose CrashLoopBackOff/ImagePullBackOff | low |
| `scale` | `deployment` | Scale replicas | medium |
| `status` | `deployment` | Check rollout status | low |
| `cordon` | `node` | Mark node unschedulable | high |
| `uncordon` | `node` | Mark node schedulable | low |
| `drain` | `node` | Drain node for maintenance | high |
| `flux-reconcile` | `kustomization` | Trigger FluxCD reconciliation | low |

### GitHub Operations

| Intent | Resource | Description | Risk Level |
|--------|----------|-------------|------------|
| `gh-run-watch` | `workflow` | Watch workflow run in real-time | low |
| `gh-run-list` | `workflow` | List workflow runs | low |
| `gh-run-view` | `workflow` | View run details | low |
| `gh-workflow-view` | `workflow` | View workflow configuration | low |
| `gh-pr-list` | `pull-request` | List pull requests | low |
| `gh-pr-view` | `pull-request` | View PR details | low |

## Safety Features

### RBAC Validation

All commands are validated against the allow-list in `rag-k8s/org/rbac-allowlist.yaml`:

```yaml
allowed_operations:
  kubectl:
    - command: rollout restart deployment
      risk_level: low
      namespaces:
        - "*"  # All namespaces allowed
      requires_approval: false
```

### Namespace Enforcement

All Kubernetes commands must specify a namespace. Cluster-wide operations are blocked unless explicitly allowed.

### Dangerous Operation Blocking

The agent automatically blocks dangerous operations:
- `kubectl delete pod` → Suggests `kubectl rollout restart` instead
- Direct pod deletion without proper context
- Operations not in the RBAC allow-list

### Audit Logging

All agent executions are logged to `rag-k8s/logs/agent.log` with:
- Timestamp
- Intent and resource
- Generated command
- Validation result
- Execution result

## Architecture

```
Claude Code
    ↓
[RAG-K8S Agent]
    ↓
[Semantic Search] → Find relevant cards
    ↓
[RBAC Validation] → Check allow-list
    ↓
[Command Planning] → Generate kubectl/gh command
    ↓
[Dry Run] → Show user what will execute
    ↓
[Execution] (if approved)
    ↓
[Audit Log]
```

## Adding New Cards

To add support for new operations:

1. Create a new card in `rag-k8s/cards/`:

```yaml
id: my-operation
title: My Operation Title
intent: my-operation
resource: my-resource
risk_level: low|medium|high

preconditions:
  - Precondition 1
  - Precondition 2

command_template: "kubectl do-something {{parameter}}"

examples:
  - goal: Example use case
    render:
      command: kubectl do-something value
      checks:
        - kubectl verify-something

notes:
  - Important notes about this operation

references:
  - https://docs.url
```

2. Rebuild the search index:

```bash
cd rag-k8s
python index/build_index.py
```

3. Test the new card:

```python
from agent.tool import k8s_exec

result = k8s_exec({
    "intent": "my-operation",
    "resource": "my-resource",
    "constraints": {"dryRun": True}
})
```

## RBAC Configuration

Update `rag-k8s/org/rbac-allowlist.yaml` to allow new operations:

```yaml
allowed_operations:
  kubectl:
    - command: your new command
      risk_level: low
      namespaces:
        - specific-namespace  # Or "*" for all
      requires_approval: true  # For high-risk operations
```

## Troubleshooting

### Agent Not Being Used

1. Check CLAUDE.md has RAG-K8S instructions
2. Verify .cursorrules is configured (if using Cursor)
3. Explicitly remind Claude at session start

### Namespace Not Found Errors

1. Use kubectl directly to discover correct namespace:
   ```bash
   kubectl get namespaces | grep pattern
   ```
2. Retry with the agent using the correct namespace

### Command Not in Allow-List

1. Check `rag-k8s/org/rbac-allowlist.yaml`
2. Add the operation if it's safe
3. Rebuild index if you added a new card

## Best Practices

1. **Always dry-run first** - Never execute without previewing
2. **Namespace specificity** - Always provide namespace, never assume
3. **Validate output** - Check `result['validation']['valid']`
4. **Review generated commands** - Don't blindly execute
5. **Audit logs** - Review `rag-k8s/logs/agent.log` regularly
6. **Least privilege** - Only allow necessary operations in RBAC

## References

- RAG-K8S Tool Documentation: `rag-k8s/README.md`
- Card Templates: `rag-k8s/cards/*.yaml`
- RBAC Allow-List: `rag-k8s/org/rbac-allowlist.yaml`
- Audit Logs: `rag-k8s/logs/agent.log`
