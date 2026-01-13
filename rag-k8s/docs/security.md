# Security & Governance

## Overview

`rag-k8s` enforces multiple layers of security to prevent unauthorized or dangerous Kubernetes operations.

## Security Layers

### 1. RBAC Allow-lists

All commands are validated against `org/rbac-allowlist.yaml` before execution.

**Allowed Verbs**:
- `get`, `describe`, `logs` (read-only)
- `rollout`, `scale` (controlled mutations)
- `cordon`, `uncordon`, `drain` (node operations)

**Forbidden Verbs** (by default):
- `delete` (except via safe alternatives like `rollout restart`)
- `apply`, `create`, `edit` (direct manifest changes)
- `exec`, `port-forward` (interactive access)

**Allowed Resources**:
- Namespaced: `pod`, `deployment`, `statefulset`, `service`, `configmap`, `secret`
- Cluster-scoped: `node`

### 2. Namespace Enforcement

All namespaced resources **must** include `-n <namespace>` or `--namespace <namespace>`.

Commands without explicit namespaces are **rejected**.

**Exception**: Cluster-scoped resources (e.g., `node`) do not require namespaces.

### 3. Forbidden Combinations

Certain verb + resource combinations are explicitly blocked:

```yaml
forbidden_combinations:
  - verb: delete
    resource: pod
    reason: Use rollout restart instead
  
  - verb: delete
    resource: namespace
    reason: Too destructive
```

### 4. Confirmation Policy

High-risk operations require explicit user confirmation:

| Operation | Risk Level | Confirmation Required |
|-----------|------------|----------------------|
| `drain` | High | Yes |
| `scale --replicas=0` | Medium | Yes |
| `cordon` | Medium | Optional |
| `rollout restart` | Low | No |

When confirmation is required, the tool returns:

```json
{
  "status": "awaiting_confirmation",
  "message": "Command requires explicit confirmation before execution",
  "plan": { ... }
}
```

The orchestrator (Claude Code) must:
1. Present the plan to the user
2. Get explicit approval
3. Re-invoke with `constraints.confirm: true`

### 5. Dry-run Mode

All operations support dry-run:

```json
{
  "constraints": {
    "dryRun": true
  }
}
```

In dry-run mode:
- Commands are validated but **not executed**
- Returns simulated result with `dryRun: true`
- Use for testing before actual execution

## Running with Minimal RBAC

### Kubernetes Service Account

For production deployments, run the agent under a dedicated service account with minimal permissions.

**Example ServiceAccount**:

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: rag-k8s-agent
  namespace: tools
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: rag-k8s-agent
  namespace: prod
rules:
  - apiGroups: ["", "apps"]
    resources: ["pods", "deployments", "statefulsets"]
    verbs: ["get", "list", "describe"]
  
  - apiGroups: ["apps"]
    resources: ["deployments"]
    verbs: ["patch"]  # For rollout restart
  
  - apiGroups: [""]
    resources: ["pods/log"]
    verbs: ["get"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: rag-k8s-agent
  namespace: prod
subjects:
  - kind: ServiceAccount
    name: rag-k8s-agent
    namespace: tools
roleRef:
  kind: Role
  name: rag-k8s-agent
  apiGroup: rbac.authorization.k8s.io
```

**Usage**:

```bash
kubectl --as=system:serviceaccount:tools:rag-k8s-agent get pods -n prod
```

### Namespace-specific Allow-lists

Customize `org/namespaces.yaml` to restrict verbs per namespace:

```yaml
namespaces:
  - name: prod
    allowed_verbs: [get, describe, logs, rollout]
  
  - name: staging
    allowed_verbs: [get, describe, logs, rollout, scale]
  
  - name: dev
    allowed_verbs: [get, describe, logs, rollout, scale, delete]
```

The validator will enforce these restrictions **in addition to** the global RBAC allow-list.

## Audit Logging

All operations are logged to `logs/agent.log` in JSONL format.

**Log Fields**:
- `timestamp`: ISO 8601 UTC
- `operation_id`: Unique UUID for tracing
- `intent`, `namespace`, `target`: Operation metadata
- `command`: Executed kubectl command
- `exit_code`: Result code (0 = success)
- `duration`: Execution time in seconds
- `stdout_digest`, `stderr_digest`: Truncated output

**Example**:

```json
{
  "timestamp": "2026-01-12T10:30:45Z",
  "level": "INFO",
  "operation_id": "a1b2c3d4-5e6f-7a8b-9c0d-1e2f3a4b5c6d",
  "intent": "restart",
  "namespace": "prod",
  "target": "deployment/api",
  "command": "kubectl rollout restart deployment/api -n prod",
  "exit_code": 0,
  "duration": 1.23,
  "stdout_digest": "deployment.apps/api restarted",
  "stderr_digest": "",
  "truncated": false
}
```

### Log Retention

**Local**: Logs stored in `logs/agent.log` (30 days by default).

**Production**: Forward logs to centralized logging:
- **Fluent Bit**: See example config in `logging.yaml`
- **Filebeat**: See example config in `logging.yaml`

Logs should be retained for compliance (90+ days recommended).

## Configuration Management

### Environment Variables

Sensitive configuration via environment variables:

```bash
export KUBECONFIG=/path/to/kubeconfig
export MODEL_ID=mlx-community/Llama-3-8B-Instruct-4bit-mlx
export LOG_PATH=/var/log/rag-k8s/agent.log
```

### Config Files

Non-sensitive configuration in `org/`:
- `namespaces.yaml`: Namespace-specific allow-lists
- `rbac-allowlist.yaml`: Global verb/resource restrictions
- `label-conventions.yaml`: Label selectors
- `enums.yaml`: Approved enumerations

**Version control**: Commit config files to Git for change tracking.

## Threat Model

### Risks Mitigated

1. **Unauthorized deletion**: `delete` verb blocked or restricted
2. **Namespace leakage**: Explicit namespace enforcement
3. **Privilege escalation**: RBAC-based allow-lists
4. **Accidental drain**: Confirmation required for high-risk ops

### Risks NOT Mitigated

1. **Malicious prompts**: The LLM planner can be influenced by adversarial prompts
2. **KUBECONFIG exposure**: Protect credentials via environment isolation
3. **Output exfiltration**: Logs contain command output digests

**Mitigation**:
- Run in isolated environments (containers, sandboxes)
- Use read-only kubeconfigs where possible
- Audit all operations via centralized logging

## Best Practices

1. **Least Privilege**: Grant minimal RBAC permissions to the service account
2. **Namespace Isolation**: Separate allow-lists for prod/staging/dev
3. **Dry-run First**: Test commands with `dryRun: true` before execution
4. **Audit Everything**: Forward logs to SIEM for compliance
5. **Rotate Credentials**: Use short-lived tokens or OIDC for kubeconfig
6. **Review Allow-lists**: Periodically audit `rbac-allowlist.yaml`
7. **Monitor Anomalies**: Alert on unusual commands or failed validations

## Incident Response

If a malicious command is detected:

1. **Immediate**: Check `logs/agent.log` for operation ID
2. **Investigate**: Review command, namespace, and user context
3. **Rollback**: Use kubectl to undo changes if needed
4. **Revoke**: Update RBAC allow-lists to prevent recurrence
5. **Audit**: Review all operations from the same session

## Compliance

For regulated environments:

- **SOC 2**: Enable audit logging with 90-day retention
- **HIPAA/PCI**: Restrict to read-only operations in production
- **GDPR**: Ensure logs do not contain PII (sanitize kubectl output)

See `logging.yaml` for log forwarding to compliant storage.
