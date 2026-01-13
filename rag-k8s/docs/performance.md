# Performance Guide

## Overview

`rag-k8s` is optimized for low-latency operation suitable for interactive use within Claude Code or similar orchestrators.

## Performance Profile

### Typical Latencies (M1/M2 Mac, 3B model)

| Operation | Latency |
|-----------|---------|
| Index search (FAISS) | ~50ms |
| Context retrieval | ~100ms |
| MLX model loading (first call) | ~5-10s |
| MLX inference (cached) | ~2-5s |
| Command validation | ~10ms |
| Command execution (kubectl) | ~500ms - 2s |
| **Total (end-to-end)** | **~3-7s** |

### Bottlenecks

1. **MLX Model Loading**: First call loads model into memory (~5-10s)
   - Mitigation: Keep agent process running (persistent service)

2. **MLX Inference**: Decoding 160 tokens takes ~2-5s
   - Mitigation: Use smaller models (1B) for speed

3. **Kubectl Execution**: Network latency to API server
   - Mitigation: Colocate agent with cluster or use low-latency network

## Optimization Strategies

### 1. Model Selection

Choose model based on quality/speed tradeoff:

| Model | Size | Latency | Quality |
|-------|------|---------|---------|
| Llama-3.2-1B-Instruct-4bit | ~600MB | ~1-2s | Good |
| Llama-3.2-3B-Instruct-4bit | ~1.5GB | ~2-5s | Better |
| Llama-3-8B-Instruct-4bit | ~4GB | ~5-10s | Best |

**Recommendation**: Start with 3B, downgrade to 1B if speed is critical.

**Configuration**: Edit `agent/config.py`:

```python
MODEL_ID = "mlx-community/Llama-3.2-1B-Instruct-4bit"
```

### 2. Decoding Parameters

Lower temperature and max_tokens reduce latency:

```python
DECODING_CONFIG = {
    "temp": 0.2,  # Deterministic (0.1-0.3)
    "max_tokens": 160,  # Short outputs only
}
```

**Trade-off**: Very low temp (<0.1) may reduce creativity. 0.2 is optimal.

### 3. Context Window Management

**Current Strategy**: Retrieve 4 cards, ~1,200 tokens total.

**Why not bigger context?**
- Larger context → longer inference time
- RAG retrieval is cheaper than LLM processing
- 4 cards provide sufficient context for most operations

**When to increase**:
- Complex multi-step workflows
- Unfamiliar Kubernetes resources

**Configuration**: Edit `agent/config.py`:

```python
RETRIEVAL_CONFIG = {
    "k": 6,  # Retrieve more cards
    "max_context_tokens": 2000,  # Allow larger context
}
```

### 4. KV-Cache Considerations

MLX uses KV-caching to speed up sequential generation.

**Implication**: Repeated similar queries are faster (cache hits).

**Anti-pattern**: Changing context frequently invalidates cache.

**Best Practice**: Group similar operations together.

### 5. Persistent Agent Process

Instead of spawning a new process per query, run as a service:

```python
# server.py (example)
from agent.tool import K8sExecTool

tool = K8sExecTool()  # Load model once

while True:
    payload = receive_request()  # e.g., from HTTP, gRPC
    result = tool.execute(payload)
    send_response(result)
```

**Benefit**: Model stays in memory, avoiding 5-10s loading overhead.

### 6. Retrieval Optimization

**Current**: FAISS `IndexFlatIP` (exact search, fast for small datasets).

**Scalability**: For 100+ cards, consider `IndexIVFFlat` or `IndexHNSW` for approximate search.

**Trade-off**: Approximate search trades recall for speed.

**Current dataset**: 15 cards → exact search is optimal.

## Scaling to Larger Corpuses

### 100 - 1,000 Cards

- Switch to `IndexIVFFlat` with 10-50 clusters
- Retrieval latency remains <100ms

### 1,000 - 10,000 Cards

- Use `IndexHNSW` for sublinear search
- Retrieval latency ~100-200ms

### Embedding Model

Current: `sentence-transformers/all-MiniLM-L6-v2` (384 dimensions, fast).

**Alternatives**:
- Faster: `paraphrase-MiniLM-L3-v2` (384d, lower quality)
- Better: `all-mpnet-base-v2` (768d, higher quality, slower)

**Recommendation**: Stick with `all-MiniLM-L6-v2` for most use cases.

## Benchmarking

Run benchmarks to measure performance:

```bash
# Benchmark retrieval
time python -m agent.retrieve --intent restart --resource deployment --query "restart"

# Benchmark full pipeline (dry-run)
time python -m agent.tool examples/claude-tool-call.json
```

Track:
- Retrieval time
- Planning time
- Validation time
- Total end-to-end time

## Memory Usage

| Component | Memory |
|-----------|--------|
| FAISS index (15 cards) | ~5MB |
| Sentence transformer | ~100MB |
| MLX 3B model (4-bit) | ~1.5GB |
| Python runtime | ~200MB |
| **Total** | **~2GB** |

**Recommendation**: Minimum 4GB RAM for comfortable operation.

## When to Step Up from 7B to 13B

Consider 13B models if:
- 3B/7B generates incorrect commands frequently
- Complex, multi-flag commands are needed
- Higher accuracy is critical (production environments)

**Trade-off**: 13B models are 2-3x slower and require more memory (~5-6GB).

## Practical Tips

1. **Cache Model**: Run agent as persistent service to avoid reload overhead
2. **Batch Operations**: Group similar commands to benefit from KV-cache
3. **Dry-run First**: Validate commands without kubectl execution latency
4. **Profile Bottlenecks**: Use Python profilers to identify slow paths
5. **Monitor Metrics**: Track latency percentiles (p50, p95, p99)

## Future Optimizations

- **Speculative Decoding**: Draft models + verification for faster generation
- **Quantization**: 2-bit or 3-bit models for even lower latency
- **Distillation**: Train smaller domain-specific models for kubectl generation
- **Caching**: Cache common command patterns to skip inference
