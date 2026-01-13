# RAG-K8S Quick Start Guide

## ✅ Installation Status

The RAG-K8S system has been successfully installed and tested:

- ✅ 15 command cards loaded
- ✅ FAISS index built (384-dimensional embeddings)
- ✅ Semantic search working
- ✅ Retrieval API functional
- ✅ Command validation operational
- ✅ All dependencies installed

## 🚀 Quick Examples

### 1. Test Semantic Search

```bash
cd rag-k8s
source venv/bin/activate
python index/search.py "restart deployment"
```

### 2. Test Retrieval with Filters

```bash
python -m agent.retrieve \
  --intent restart \
  --resource deployment \
  --query "safely restart a deployment"
```

### 3. Test Validation

```bash
python -m runtime.validate
```

### 4. Use the k8s_exec Tool (Python API)

```python
from agent.tool import k8s_exec

# Dry-run example
result = k8s_exec({
    "intent": "restart",
    "resource": "deployment",
    "namespace": "prod",
    "name": "api",
    "constraints": {
        "confirm": False,
        "dryRun": True
    }
})

print(result)
```

## 📊 System Capabilities

### Available Intents
- `diagnose` - Troubleshoot issues
- `restart` - Rolling restarts
- `logs` - View logs
- `scale` - Scale deployments
- `status` - Check rollout status
- `describe` - Get detailed info
- `events` - View namespace events
- `top` - Resource usage
- `cordon` / `uncordon` / `drain` - Node operations

### Safety Features

1. **RBAC Allow-lists**: Only approved verbs/resources
2. **Namespace Enforcement**: All commands must specify namespace
3. **Dangerous Operation Blocking**: `delete pod` is blocked → suggests `rollout restart`
4. **Dry-run Mode**: Test before executing
5. **Audit Logging**: All operations logged to `logs/agent.log`

## 🔧 Configuration

### Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
KUBECONFIG=/path/to/kubeconfig
MODEL_ID=mlx-community/Llama-3.2-3B-Instruct-4bit
LOG_PATH=logs/agent.log
```

### Customize Allow-lists

Edit `org/rbac-allowlist.yaml` to adjust permissions:

```yaml
allowed_verbs:
  - get
  - describe
  - logs
  - rollout
  - scale

allowed_resources:
  - pod
  - deployment
  - statefulset
  - node
```

## 📚 Next Steps

1. **Read Documentation**: Check `docs/` for detailed guides
2. **Add Command Cards**: Create new cards in `cards/` following the schema
3. **Rebuild Index**: Run `make build-index` after adding cards
4. **Run Tests**: Execute `make test` to verify functionality

## 🤖 MLX Model Usage

The MLX planner generates kubectl commands using a local 3B parameter LLM with 4-bit quantization.

**Performance**:
- First load: ~5-10 seconds (model download and initialization)
- Subsequent calls: <1 second (model cached in memory)
- Memory usage: ~2GB RAM

**Requirements**:
- macOS with Apple Silicon (M1/M2/M3)
- For testing without MLX, use dry-run mode or mock the planner

**Generation Settings**:
- Temperature: 0.0 (deterministic output)
- Max tokens: 256 (complete JSON responses)
- Retry logic: Automatic retry with corrected prompt on JSON parse failure

## 🎯 Integration with Claude Code

From Claude Code, you can now call:

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

See `docs/prompts.md` for complete integration guide.

## 📝 Example Workflow

```bash
# 1. Find relevant command patterns
python -m agent.retrieve \
  --intent diagnose \
  --resource pod \
  --query "crashloop backoff"

# 2. Use the tool (dry-run first)
python -c "
from agent.tool import k8s_exec
import json

result = k8s_exec({
    'intent': 'diagnose',
    'resource': 'pod',
    'namespace': 'prod',
    'name': 'api-xyz',
    'constraints': {'dryRun': True}
})

print(json.dumps(result, indent=2))
"

# 3. Check logs
tail -f logs/agent.log
```

## ⚠️ Important Notes

- **Never run in production without testing**: Always use dry-run first
- **Review allow-lists**: Customize `org/` configs for your environment
- **Audit logs**: Monitor `logs/agent.log` for all operations
- **KUBECONFIG security**: Protect your cluster credentials

## 🆘 Troubleshooting

### Index not found
```bash
make build-index
```

### Import errors
```bash
make deps
```

### Validation failures
Check `org/rbac-allowlist.yaml` and ensure commands include namespace.

---

**Version**: v0.1.0
**Created**: 2026-01-13
