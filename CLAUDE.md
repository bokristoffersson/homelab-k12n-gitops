# Homelab K12n GitOps Project

## Project Overview

This is a Kubernetes homelab managed with GitOps using FluxCD. The infrastructure runs on a k3d cluster with various services for home automation, monitoring, and data collection.

### Key Technologies
- **GitOps**: FluxCD for continuous deployment
- **Kubernetes**: k3d cluster
- **Authentication**: Authentik (IdP) + oauth2-proxy (auth gateway)
- **Ingress**: Traefik with ForwardAuth middleware
- **Databases**:
  - TimescaleDB (telemetry data)
  - Authentik PostgreSQL (identity data)
- **Message Broker**: Redpanda (Kafka-compatible)
- **MQTT**: Mosquitto broker
- **Data Pipeline**: mqtt-kafka-bridge (Redpanda Connect/Benthos)

### Applications
- **homelab-api**: Rust/Axum read-only REST API serving TimescaleDB data (energy, heatpump, temperature)
- **heatpump-web**: React/TypeScript frontend SPA
- **energy-ws**: Rust/Axum WebSocket server streaming real-time energy data from Redpanda
- **redpanda-sink**: Kafka consumer writing telemetry to TimescaleDB
- **mqtt-kafka-bridge**: MQTT to Kafka/Redpanda bridge (Redpanda Connect)

### IoT Devices
- Shelly H&T Gen3 (temperature/humidity sensor)
  - MQTT topic: `shellyhtg3-e4b32322a0f4/events/rpc`
  - Wakes every 1 minute, sends on temp change ≥0.5°C or humidity ≥5%
  - Forced update every 2 hours

## Architecture Highlights

### Authentication Flow
1. Frontend (heatpump-web) uses OIDC authorization code flow with Authentik
2. SPA stores JWT tokens and includes them in API requests
3. Backend validates JWT signatures using Authentik's JWKS endpoint
4. oauth2-proxy available for ForwardAuth pattern (not currently used)

See `docs/AUTHENTICATION.md` for complete details.

### Data Pipeline
```
IoT Device → MQTT (Mosquitto) → Redpanda (via mqtt-kafka-bridge) → TimescaleDB (via redpanda-sink)
                                                                    ↓
                                                              homelab-api (REST)
                                                                    ↓
                                                              heatpump-web (SPA)
```

### Backup Strategy
- **TimescaleDB**: Daily backup at 2 AM to S3
- **Authentik PostgreSQL**: Daily backup at 3 AM to S3
- Backups use pg_dump + gzip compression
- Stored in separate S3 prefixes

## Code Preferences

### General
- Never use emojis unless explicitly requested
- Follow conventional commits format
- Keep solutions simple - avoid over-engineering
- Don't add features beyond what's requested
- Only add error handling at system boundaries

### Git Workflow
- **IMPORTANT**: Always use multi-line HEREDOC for commit messages:
  ```bash
  git commit -m "$(cat <<'EOF'
  type: description

  Detailed explanation...

  🤖 Generated with [Claude Code](https://claude.com/claude-code)

  Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
  EOF
  )"
  ```
- Main branch: `main`
- Use `kubectl` (not `k` alias)
- Push commits after user confirms

### Sealed Secrets
**CRITICAL**: I (the user) will create all sealed secrets manually. Claude should:
1. **Never** attempt to create sealed secrets directly
2. **Always** provide kubectl + kubeseal command snippets instead
3. Use this format for sealed secret creation snippets:

```bash
kubectl create secret generic <secret-name> \
  --namespace=<namespace> \
  --from-literal=KEY1=value1 \
  --from-literal=KEY2=value2 \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace <namespace> \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > <secret-name>-sealed.yaml
```

### Kubernetes
- Use full commands: `kubectl get pods -n namespace`
- Prefer `kubectl` over shell aliases
- Check pod status with `-l app=<name>` labels
- Always specify namespace explicitly

### Deployment Workflow
**IMPORTANT**: After certain changes, deployments must be manually restarted:

1. **After GitHub Actions builds**: When a new application image is built and pushed, k3d doesn't automatically detect it. You must:
   ```bash
   kubectl rollout restart deployment/<app-name> -n <namespace>
   ```

2. **After ConfigMap updates**: When ConfigMaps are modified via GitOps, pods don't automatically reload. You must:
   ```bash
   kubectl rollout restart deployment/<app-name> -n <namespace>
   ```

**Example workflow**:
```bash
# 1. Wait for GitHub Actions to complete
gh run watch

# 2. Import new image to k3d (if needed)
k3d image import <image-name>:<tag> -c <cluster-name>

# 3. Restart deployment to pick up changes
kubectl rollout restart deployment/homelab-api -n homelab-api
kubectl rollout restart deployment/heatpump-web -n heatpump-web

# 4. Verify rollout
kubectl rollout status deployment/homelab-api -n homelab-api
```

### Rust/Backend (homelab-api)
**ARCHITECTURAL RULE - READ ONLY**:
- homelab-api is a **read-only REST API** for serving data from TimescaleDB
- **NEVER** create database write operations (INSERT, UPDATE, DELETE)
- **NEVER** add Redpanda/Kafka consumer logic
- Purpose: Query and serve existing telemetry data only
- Data writes are handled exclusively by redpanda-sink service

**Code Guidelines**:
- Run `cargo fmt` before committing
- Use Axum for REST APIs
- JWT validation for authentication
- sqlx for database access (SELECT queries only)
- No over-engineering - direct implementations

### TypeScript/Frontend (heatpump-web)
- React with TypeScript (strict mode)
- TanStack Query for data fetching
- Recharts for data visualization
- No unused variables (lint will fail)

### Database
- TimescaleDB for time-series data
- Use continuous aggregates for summaries
- Always use transactions for multi-step operations

## Common Commands

### Flux
```bash
# Reconcile all apps
flux reconcile kustomization apps

# Reconcile specific app
flux reconcile kustomization <app-name>

# Check Flux status
flux get kustomizations
```

### Kubernetes
```bash
# Get pods in namespace
kubectl get pods -n <namespace>

# View logs
kubectl logs -n <namespace> -l app=<app-name> --tail=50

# Restart deployment
kubectl rollout restart deployment/<name> -n <namespace>

# Check deployment status
kubectl rollout status deployment/<name> -n <namespace>
```

### GitHub Actions
```bash
# List workflow runs
gh run list

# Watch workflow
gh run watch
```

## File Structure

```
gitops/
├── apps/
│   ├── base/           # Base kustomize configurations
│   └── homelab/        # Homelab-specific overlays
└── infrastructure/
    └── controllers/    # Infrastructure controllers (Flux, Traefik, etc.)

applications/
├── homelab-api/        # Rust backend
├── heatpump-web/       # React frontend
└── redpanda-sink/      # Kafka consumer

docs/
└── AUTHENTICATION.md   # Authentication architecture documentation
```

## Recent Changes

- Added temperature API endpoints and 24h chart (2025-12-30)
- Created authentication documentation (2025-12-30)
- Added Authentik PostgreSQL backup CronJob (2025-12-30)
- Fixed mqtt-kafka-bridge configuration for Shelly sensor (2025-12-30)

## Notes

- Don't commit `node_modules/` (already in .gitignore)
- Cloudflare Tunnel handles TLS termination
- Internal cluster traffic uses HTTP
- All API routes use CORS middleware for `https://heatpump.k12n.com` and `http://localhost:5173`
