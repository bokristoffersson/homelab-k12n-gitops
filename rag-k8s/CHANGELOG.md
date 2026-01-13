# Changelog

All notable changes to the RAG-K8S project will be documented in this file.

## [0.1.2] - 2026-01-13

### Added
- **Namespace Discovery**: New command card for listing and filtering namespaces
  - `get-namespaces.yaml` card for discovering available namespaces
  - Helpful for resolving "namespace not found" errors
  - Supports grep-based filtering (e.g., find all namespaces containing "heatpump")
  - Cluster-scoped resource support (no namespace flag required)

### Changed
- **Validator**: Extended cluster-scoped resource support
  - Added namespace, persistentvolume, storageclass, clusterrole, clusterrolebinding
  - Commands for cluster-scoped resources no longer require `-n` flag
- **RBAC**: Added `namespace` to allowed resources list
- **Documentation**: Added "Error Recovery - Namespace Not Found" workflow to CLAUDE.md

## [0.1.1] - 2026-01-13

### Fixed
- **MLX Planner JSON Generation**: Fixed JSON generation reliability issues
  - Added concrete JSON example to system prompt for better model guidance
  - Changed temperature from 0.2 to 0.0 for deterministic output
  - Increased max_tokens from 160 to 256 to allow complete JSON completion
  - Improved JSON extraction regex to handle nested braces
  - Enhanced retry logic with explicit JSON structure template

### Changed
- **Decoding Configuration**: Updated default parameters in `agent/config.py`
  - `temperature`: 0.2 → 0.0 (maximum determinism)
  - `top_p`: 0.9 → 1.0 (greedy decoding)
  - `max_tokens`: 160 → 256 (complete JSON)
  - `repetition_penalty`: 1.1 → 1.05 (lighter penalty)

- **System Prompt**: Enhanced with structured example
  - Added complete JSON example showing all required fields
  - Clarified output format requirements
  - Emphasized "no text before or after" JSON constraint

### Improved
- **Error Handling**: Better JSON parsing with fallback mechanisms
- **Documentation**: Updated QUICKSTART.md with MLX performance details

## [0.1.0] - 2026-01-13

### Added
- Initial release of RAG-K8S system
- FAISS-based semantic search with 15 command cards
- MLX-powered local LLM planner (Llama 3.2 3B 4-bit)
- RBAC validation and safety guardrails
- Audit logging system
- Command execution with timeout and dry-run support
- `k8s_exec` tool contract for orchestrator integration
- Comprehensive documentation and examples

### Supported Operations
- `diagnose` - Troubleshoot pod issues
- `restart` - Rolling deployment restarts
- `logs` - View pod logs
- `scale` - Scale deployments
- `status` - Check rollout status
- `describe` - Get resource details
- `events` - View namespace events
- `top` - Resource usage metrics
- Node operations: `cordon`, `uncordon`, `drain`

### Safety Features
- RBAC allow-lists for verbs and resources
- Namespace enforcement
- Dangerous operation blocking (e.g., `delete pod` → suggests `rollout restart`)
- Dry-run mode for safe testing
- JSONL audit logging

---

**Version Format**: [MAJOR.MINOR.PATCH]
- MAJOR: Breaking changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, improvements
