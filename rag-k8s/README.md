# RAG-K8S

Kubernetes command assistant powered by RAG + MLX local agent.

## Overview

`rag-k8s` is a RAG-powered Kubernetes operations tool that combines:

- **Command Card Corpus**: Curated YAML templates for common K8s operations
- **Semantic Search**: FAISS-based retrieval of relevant command patterns
- **MLX Local Agent**: 7B/13B parameter model (4-bit quantized) for plan generation
- **Validation & Execution**: RBAC enforcement, guardrails, and safe command execution
- **Tool Interface**: Single `k8s_exec` function for upstream orchestrators (Claude Code, etc.)

The system retrieves relevant command patterns, generates kubectl commands via a local LLM, validates against RBAC policies, and executes with safety guardrails.

## Key Features

### Safety & Governance
- **RBAC Allow-lists**: Namespace-scoped verb/resource restrictions
- **Guardrails**: Automatic blocking of dangerous operations (e.g., `delete pod` → suggest `rollout restart`)
- **Confirmation Policy**: Medium/high risk actions require explicit confirmation
- **Audit Logging**: JSONL logs with operation IDs, commands, and result digests
- **Dry-run Mode**: Test commands without execution

### Performance
- **Compact Context**: Retrieval returns 2-4 short snippets (≤1,200 tokens total)
- **Low-latency Inference**: MLX 4-bit models with constrained decoding (temp=0.2, max_tokens=160)
- **Efficient Indexing**: FAISS inner product search on normalized embeddings

## Getting Started

### Prerequisites
- Python 3.9+
- macOS with Apple Silicon (for MLX) or Linux (CPU-only mode)
- kubectl configured with cluster access

### Quick Start

```bash
# Clone or navigate to repo
cd rag-k8s/

# Run bootstrap script
./scripts/bootstrap.sh

# Activate virtual environment
source venv/bin/activate

# Configure environment
cp .env.example .env
# Edit .env with your KUBECONFIG path

# Run demo
./scripts/demo_restart.sh
```

The bootstrap script will:
1. Create a Python virtual environment
2. Install dependencies (faiss-cpu, sentence-transformers, mlx-lm, etc.)
3. Build the FAISS index from 15 command cards
4. Create logs directory
5. Set up .env template

## Usage

### 1. Direct CLI (Retrieval)

Retrieve relevant command snippets:

```bash
python -m agent.retrieve --intent restart --resource deployment --query "safe restart"
```

### 2. Python API

```python
from agent.tool import k8s_exec

payload = {
    "intent": "restart",
    "resource": "deployment",
    "namespace": "prod",
    "name": "api",
    "constraints": {
        "confirm": False,
        "dryRun": False
    }
}

result = k8s_exec(payload)
print(result["plan"]["command"])
```

### 3. Server Mode (Persistent Model)

Run the agent as a server with the model pre-loaded in memory for fast inference:

```bash
# Start server (loads model once at startup)
./scripts/start-server.sh

# Or manually:
python -m agent.server --host 127.0.0.1 --port 8000 --preload
```

The server exposes REST endpoints:
- `GET /health` - Health check
- `POST /k8s-exec` - Execute K8s operation (full workflow)
- `POST /plan` - Generate command plan only

Example request:
```bash
curl -X POST http://127.0.0.1:8000/k8s-exec \
  -H "Content-Type: application/json" \
  -d '{
    "intent": "restart",
    "resource": "deployment",
    "namespace": "prod",
    "name": "api",
    "constraints": {"dryRun": true}
  }'
```

**Benefits of server mode:**
- Model loaded once at startup (no per-request loading time)
- Fast inference for Claude Code integration
- Persistent connection for better performance

### 4. Claude Code Integration

From Claude Code, call the tool function:

```python
from rag_k8s.agent.tool import k8s_exec

result = k8s_exec({
    "intent": "diagnose",
    "resource": "pod",
    "namespace": "staging",
    "name": "worker-abc123",
    "constraints": {"dryRun": True}
})
```

### 4. Demo Script

```bash
./scripts/demo_restart.sh
```

## Architecture

```
User Query
    ↓
Retrieval (FAISS semantic search) → 2-4 command card snippets
    ↓
MLX Planner (7B/13B, 4-bit) → JSON plan with kubectl command
    ↓
Validator (RBAC + guardrails) → Safety checks
    ↓
Executor (subprocess with timeout) → kubectl execution
    ↓
Logger (JSONL audit log) → Observability
```

## Configuration

### Namespaces (`org/namespaces.yaml`)
Define allowed verbs per namespace:
- `prod`: Read-only + safe rollouts
- `staging`: Broader permissions
- `dev`: Full access

### RBAC Allow-list (`org/rbac-allowlist.yaml`)
Permitted verbs and resources. Violations are blocked before execution.

### Model Selection (`agent/config.py`)
Default: `mlx-community/Llama-3.2-3B-Instruct-4bit`

Alternatives:
- Faster (smaller): `mlx-community/Llama-3.2-1B-Instruct-4bit`
- Higher quality: `mlx-community/Llama-3-8B-Instruct-4bit-mlx`

## Command Cards

Located in `cards/`, each YAML file defines:
- `command_template`: Kubectl template with `{{placeholders}}`
- `intent`: Operation type (diagnose, restart, logs, etc.)
- `resource`: K8s resource (deployment, pod, node, etc.)
- `risk_level`: Safety classification (none, low, medium, high)
- `examples`: Worked examples with rendered commands
- `notes`: Implementation tips and warnings

Example: `cards/restart-deployment.yaml`

## Testing

```bash
make test
```

Runs:
- Index build/search tests
- Retrieval filtering tests
- Validation logic tests
- Executor timeout/truncation tests

## Logging

All operations logged to `logs/agent.log` in JSONL format:

```json
{
  "timestamp": "2026-01-12T10:30:45Z",
  "level": "INFO",
  "operation_id": "a1b2c3d4-...",
  "intent": "restart",
  "namespace": "prod",
  "command": "kubectl rollout restart deployment/api -n prod",
  "exit_code": 0,
  "duration": 1.23
}
```

For production, configure log forwarding to Fluent Bit or Filebeat (see `logging.yaml`).

## Troubleshooting

### Index not found
```bash
make build-index
```

### Invalid JSON from planner
The planner retries once automatically. If it continues failing, check context size and model load.

### Validation failures
Review `org/rbac-allowlist.yaml` and ensure commands include namespace (`-n <namespace>`).

### Command timeout
Default timeout is 15s. Adjust in `runtime/exec.py` or pass `timeout` parameter.

## Security

- Run the agent under a minimal RBAC service account (namespace-scoped)
- Review and customize `org/rbac-allowlist.yaml` for your environment
- High-risk operations (drain, scale to 0) require explicit confirmation
- Never expose the tool to untrusted inputs without authentication

See `docs/security.md` for details.

## Performance

- **Retrieval**: ~50ms for semantic search (15 cards)
- **Planning**: ~2-5s for MLX inference (3B model, M1/M2)
- **Total latency**: ~3-7s end-to-end

For faster responses, use smaller models (1B) or cache frequent commands.

See `docs/performance.md` for optimization tips.

## Documentation

- `docs/corpus-schema.md`: Command card schema reference
- `docs/prompts.md`: Claude Code integration guide
- `docs/security.md`: Security and governance details
- `docs/performance.md`: Performance optimization guide

## Contributing

To add new command cards:
1. Create YAML file in `cards/` following schema in `docs/corpus-schema.md`
2. Run `make build-index` to rebuild FAISS index
3. Run `make test` to verify

## Version

**Initial tag**: v0.1.0

## License

MIT
