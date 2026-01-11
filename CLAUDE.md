# Homelab K12n GitOps Project

## Project Overview

This is a Kubernetes homelab managed with GitOps using FluxCD. The infrastructure runs on a k3s cluster deployed across two Raspberry Pi nodes with various services for home automation, monitoring, and data collection.

## Infrastructure

- **Cluster Type**: k3s
- **Nodes**:
  - p0.local (Raspberry Pi)
  - p1.local (Raspberry Pi)
- **Architecture**: arm64
- **Container Registry**: GitHub Container Registry (ghcr.io)

### Key Technologies
- **GitOps**: FluxCD for continuous deployment
- **Kubernetes**: k3s cluster (arm64)
- **Authentication**: Authentik (IdP) + oauth2-proxy (auth gateway)
- **Ingress**: Traefik with ForwardAuth middleware
- **Databases**:
  - TimescaleDB (telemetry data)
  - Authentik PostgreSQL (identity data)
- **Message Broker**: Redpanda (Kafka-compatible)
  - Topics managed via `rpk` commands in Kubernetes Job (not operator/CRDs)
  - Single-node cluster in `redpanda-v2` namespace
- **MQTT**: Mosquitto broker
- **Data Pipeline**: mqtt-kafka-bridge (Redpanda Connect/Benthos)

### Applications
- **homelab-api**: Rust/Axum read-only REST API serving TimescaleDB data (energy, heatpump, temperature)
- **heatpump-web**: React/TypeScript frontend SPA
- **heatpump-settings-api**: Rust/Axum API managing heatpump settings (Kafka consumer: `heatpump-settings-api` group)
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

1. **After GitHub Actions builds**: When a new application image is built and pushed to GHCR, Kubernetes doesn't automatically detect it. You must:
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

# 2. Restart deployment to pick up new image
kubectl rollout restart deployment/homelab-api -n homelab-api
kubectl rollout restart deployment/heatpump-web -n heatpump-web

# 3. Verify rollout
kubectl rollout status deployment/homelab-api -n homelab-api
```

**Note**: k3s pulls images directly from GHCR, so no manual image import is needed (unlike k3d).

### Docker
**REQUIRED**: All Dockerfiles must implement layered builds with dependency caching.

**Multi-stage builds**:
- Separate dependency installation from source code compilation
- Cache dependency layers to speed up rebuilds
- Only rebuild when dependencies change (Cargo.toml, package.json, etc.)

**Example - Rust applications**:
```dockerfile
# Build stage
FROM rust:1.83 AS builder

WORKDIR /app

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Build dependencies with dummy source files
# IMPORTANT: For lib+bin crates, create both lib.rs and main.rs
RUN mkdir src && \
    echo "pub fn lib_dummy() {}" > src/lib.rs && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual application (dependencies already cached)
RUN touch src/main.rs && cargo build --release
```

**Notes**:
- Always create both `lib.rs` and `main.rs` dummies for Rust projects (handles both lib+bin and bin-only crates)
- Binary-only crates will ignore lib.rs, but lib+bin crates will fail without it
- The `touch` command forces rebuild of the actual code while reusing cached dependencies

**Example - Node.js/TypeScript applications**:
```dockerfile
# Build stage
FROM node:20 AS builder

WORKDIR /app

# Copy package files first to cache npm install
COPY package.json package-lock.json ./

# Install dependencies (cached layer)
RUN npm ci

# Copy source code
COPY . .

# Build application
RUN npm run build
```

**Benefits**:
- First build: ~30 minutes (builds everything)
- Subsequent code-only changes: ~5 minutes (reuses dependency cache)
- Reduces CI/CD time and costs

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

- Deployed heatpump-settings-api service with separate Kafka consumer group (2026-01-11)
- Replaced Redpanda operator with rpk-based topic management (Job in redpanda-v2 namespace) (2026-01-11)
- Created comprehensive TechDocs for redpanda-v2 with Backstage integration (2026-01-11)
- Removed redpanda-operator and redpanda-system namespace (2026-01-11)
- Added temperature API endpoints and 24h chart (2025-12-30)
- Created authentication documentation (2025-12-30)
- Added Authentik PostgreSQL backup CronJob (2025-12-30)
- Fixed mqtt-kafka-bridge configuration for Shelly sensor (2025-12-30)

## Notes

- Don't commit `node_modules/` (already in .gitignore)
- Cloudflare Tunnel handles TLS termination
- Internal cluster traffic uses HTTP
- All API routes use CORS middleware for `https://heatpump.k12n.com` and `http://localhost:5173`
