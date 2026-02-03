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
- **homelab-settings-api**: Rust/Axum API managing homelab settings including heatpump (Kafka consumer: `homelab-settings-api` group)
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

**CRITICAL - Development Workflow**:
1. **Always create feature branches** - Never commit directly to `main`
2. **Create Pull Requests** - Push feature branch and create PR on GitHub
3. **AI Architectural Review** - PRs are automatically reviewed by an AI agent that checks for:
   - Compliance with architectural rules in CLAUDE.md
   - Code quality and best practices
   - Database migration issues
   - Security concerns
4. **User merges PRs** - I (the user) will merge the PR after reviewing the AI feedback

**Feature Branch Workflow**:
```bash
# Create feature branch
git checkout -b feature/descriptive-name

# Make changes and commit
git add -A
git commit -m "$(cat <<'EOF'
type: description

Detailed explanation...

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"

# Push and create PR
git push -u origin feature/descriptive-name
gh pr create --title "Title" --body "Description"

# User will merge after AI review
```

**Commit Message Format**:
- **IMPORTANT**: Always use multi-line HEREDOC for commit messages
- Follow conventional commits: `type: description`
- Include detailed explanation of changes
- Always include Claude Code footer

**Other Rules**:
- Main branch: `main`
- Use `kubectl` (not `k` alias)
- Never force push to `main`

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

### Database Migrations

**CRITICAL**: Database migrations are managed through GitOps using Kubernetes Jobs. **NEVER** run migrations manually using `kubectl exec`.

**Migration Locations**:
- **TimescaleDB**: `gitops/apps/base/timescaledb/migrations/`
- **Homelab Settings (PostgreSQL)**: `gitops/apps/base/homelab-settings/migrations/`
- **Authentik (PostgreSQL)**: Managed by Authentik application

**Migration Workflow**:
1. **Create migration file** in the appropriate directory:
   ```sql
   -- Migration: XXX_descriptive_name
   -- Description: What this migration does

   \c database_name  -- For TimescaleDB only

   -- Your migration SQL here
   ALTER TABLE table_name ADD COLUMN new_column type;

   -- Record migration
   INSERT INTO schema_migrations (version, name)
   VALUES (XXX, 'descriptive_name')
   ON CONFLICT (version) DO NOTHING;
   ```

2. **Update kustomization.yaml** to include the new migration file:
   - **TimescaleDB**: Edit `gitops/apps/base/timescaledb/kustomization.yaml`
   - **Homelab Settings**: Edit `gitops/apps/base/homelab-settings/kustomization.yaml`

   Add the new migration file to the `configMapGenerator` section:
   ```yaml
   configMapGenerator:
     - name: timescaledb-migrations  # or postgres-migrations
       files:
         - migrations/001_initial_schema.sql
         - migrations/002_fix_energy_hourly.sql
         - migrations/XXX_your_new_migration.sql  # ADD THIS LINE
         - migrations/run_migrations.sh
   ```

   **CRITICAL**: If you forget this step, FluxCD will NOT sync the migration file to the cluster and the migration will NOT run.

3. **Commit and push** to feature branch - migration files are part of GitOps
4. **Create PR** - AI agent will review migrations
5. **After merge** - FluxCD will automatically sync the updated ConfigMaps
6. **Manually restart migration jobs** to apply the new migrations:
   ```bash
   kubectl delete job -n timescaledb timescaledb-migration
   kubectl delete job -n homelab-settings postgres-migration
   ```
7. **Verify migration** completed successfully:
   ```bash
   kubectl logs -n timescaledb job/timescaledb-migration
   kubectl logs -n homelab-settings job/postgres-migration
   ```

**Migration File Naming**:
- Use sequential numbers: `001_`, `002_`, `003_`, etc.
- Descriptive name: `add_integral_to_heatpump.sql`
- Full example: `004_add_integral_to_heatpump.sql`

**Schema Migrations Table**:
- **TimescaleDB**: `schema_migrations (version, name)`
- **Homelab Settings**: `schema_migrations (version, name, applied_at)`

**NEVER DO THIS**:
```bash
# ❌ WRONG - Manual migration execution
kubectl exec -n timescaledb pod -- psql -U postgres -d telemetry -c "ALTER TABLE..."
```

**DO THIS INSTEAD**:
```bash
# ✅ CORRECT - GitOps migration workflow
1. Create migration file in gitops/apps/base/timescaledb/migrations/
2. Commit to feature branch
3. Create PR and wait for AI review
4. User merges PR
5. FluxCD automatically runs migration Job
```

### Kubernetes Operations

Use `kubectl` directly for all Kubernetes operations.

### Deployment Workflow
**IMPORTANT**: After certain changes, deployments must be manually restarted.

1. **After GitHub Actions builds**: When a new application image is built and pushed to GHCR, Kubernetes doesn't automatically detect it.

2. **After ConfigMap updates**: When ConfigMaps are modified via GitOps, pods don't automatically reload.

**Example workflow**:
```bash
# 1. Wait for GitHub Actions to complete
gh run watch

# 2. Restart deployment
kubectl rollout restart deployment/homelab-api -n homelab-api

# 3. Check rollout status
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

### Testing Philosophy

**Pragmatic Homelab Approach**: Test critical paths that could cause data loss, security issues, or system downtime. Manual testing is acceptable for UI and non-critical flows.

#### What to Test

**CRITICAL (must have tests)**:
- **Data writes**: redpanda-sink's Kafka → TimescaleDB pipeline
- **Authentication**: JWT validation in all APIs
- **Message processing**: Kafka consumer logic (deserialization, business rules)
- **Database migrations**: Schema changes must be validated

**IMPORTANT (should have tests)**:
- **API contracts**: REST endpoints returning correct data shapes
- **Settings mutations**: heatpump-settings-api state changes
- **Time-series queries**: TimescaleDB aggregations and date ranges

**OPTIONAL (manual testing OK)**:
- Frontend components and styling
- Dashboard visualizations
- Non-critical UI flows

#### Testing Guidelines

**Rust services**:
- Unit tests for business logic (parsing, validation, transformations)
- Integration tests for external systems using testcontainers (PostgreSQL, Redpanda)
- Run tests in CI - builds must pass tests before image push

**TypeScript/Frontend**:
- Tests for critical user flows only (login, data fetching)
- Manual testing acceptable for UI components

**CI/CD**:
- GitHub Actions runs tests before building images
- Failing tests block deployment
- Manual testing during staging (port-forward to cluster)

**Test data**:
- Use fixtures for known sensor payloads (Shelly H&T messages)
- Testcontainers for isolated DB/Kafka environments
- Never test against production TimescaleDB

#### When Tests Run

- **Pre-push**: Optional (developer choice)
- **CI**: Required for all Rust services
- **Pre-deployment**: Manual smoke test after image restart

### AI Architectural Review

**Purpose**: Automated guardrails for developers (including AI assistants) to ensure code quality and architectural compliance.

All PRs are automatically reviewed by an AI agent (Claude Sonnet 4.5) that acts as a **senior architect**. The agent:
- **Reviews against CLAUDE.md rules** - Checks compliance with all architectural principles in this document
- **Posts detailed feedback** - Summary comment with violations, suggestions, and rationale
- **Blocks merge on violations** - Critical issues prevent PR merge until fixed
- **Re-reviews automatically** - Updates review when PR is updated with new commits
- **Provides educational feedback** - Explains why certain patterns are preferred

**What gets reviewed**:
- **Code changes**: Database operations (read-only API enforcement), auth patterns, over-engineering detection
- **Dockerfiles**: Layered builds, dependency caching, security best practices
- **Tests**: Coverage for critical paths (data writes, auth, message processing)
- **GitOps configs**: Sealed secrets usage, resource limits, FluxCD best practices
- **Database migrations**: Proper migration structure, schema_migrations table updates
- **Git workflow**: Feature branches, commit message format, PR descriptions

**Review workflow**:
1. Push commits to feature branch
2. Create PR on GitHub
3. AI agent automatically reviews within minutes
4. Address feedback by pushing new commits
5. AI re-reviews automatically
6. User merges after approval

This ensures high code quality even when working with AI coding assistants. The AI agent acts as a **guardrail** to catch common mistakes and enforce best practices.

See `docs/ARCH_REVIEW.md` for complete details.

## Common Commands

### FluxCD

**Reconcile kustomization** (after GitOps push):
```bash
flux reconcile kustomization <app-name>
```

**Check Flux status**:
```bash
flux get kustomizations
```

**Suspend during maintenance**:
```bash
flux suspend kustomization <app-name>
```

**Resume after maintenance**:
```bash
flux resume kustomization <app-name>
```

### Kubernetes

**View logs**:
```bash
kubectl logs -n <namespace> deployment/<app-name>
```

**Restart deployment**:
```bash
kubectl rollout restart deployment/<app-name> -n <namespace>
```

**Check rollout status**:
```bash
kubectl rollout status deployment/<app-name> -n <namespace>
```

**Describe deployment**:
```bash
kubectl describe deployment/<app-name> -n <namespace>
```

**Get pods**:
```bash
kubectl get pods -n <namespace>
```

**Exec into pod**:
```bash
kubectl exec -it -n <namespace> <pod-name> -- /bin/sh
```

**Port forward**:
```bash
kubectl port-forward -n <namespace> deployment/<app-name> 8080:8080
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

- Renamed heatpump-settings to homelab-settings (namespace, database, services, images) (2026-02-03)
- Deployed homelab-settings-api service with separate Kafka consumer group (2026-01-11)
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
- **CORS Configuration**: Traefik CORS middleware (`gitops/apps/base/traefik-middlewares/cors.yaml`) allows:
  - `https://homelab.k12n.com`
  - `http://localhost:5173`
  - `http://localhost:8080`
- **CORS Preflight**: A dedicated Traefik route handles OPTIONS requests for `/api/*` paths at priority 210, ensuring preflight requests succeed without requiring Authorization headers
