# Architecture

## Design Principles

### Read-Only Architecture

**CRITICAL RULE**: homelab-api is strictly a **read-only REST API**.

- **NEVER** create database write operations (INSERT, UPDATE, DELETE)
- **NEVER** add Redpanda/Kafka consumer logic
- **Purpose**: Query and serve existing telemetry data only
- **Data writes**: Handled exclusively by redpanda-sink service

This architectural constraint ensures:
- Clear separation of concerns
- Simpler codebase and security model
- No risk of data corruption via API
- Optimal read performance without write contention

## Tech Stack

```
┌─────────────────────┐
│   heatpump-web SPA  │
│   (React/TypeScript)│
└──────────┬──────────┘
           │ HTTPS + JWT
           ▼
┌─────────────────────┐
│   homelab-api       │
│   (Rust/Axum)       │
│   - JWT validation  │
│   - Read queries    │
│   - CORS middleware │
└──────────┬──────────┘
           │ sqlx
           ▼
┌─────────────────────┐
│   TimescaleDB       │
│   - Hypertables     │
│   - Continuous aggs │
│   - Compression     │
└─────────────────────┘
```

## Authentication Flow

1. **Frontend** requests access token from Authentik (OIDC Authorization Code Flow)
2. **Frontend** stores JWT and includes in API requests
3. **homelab-api** validates JWT signature using Authentik's JWKS endpoint
4. If valid, query executes; if invalid, returns 401

### JWT Validation

```rust
// Validates token signature and expiry
let token_data = decode::<Claims>(
    token,
    &DecodingKey::from_jwks(&jwks),
    &Validation::new(Algorithm::RS256)
)?;
```

## Database Access

### Connection Pool

Uses sqlx's connection pooling for efficient database access:

```rust
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

### Query Patterns

All queries are SELECT-only and use prepared statements via sqlx macros:

```rust
sqlx::query_as!(
    EnergyReading,
    r#"
    SELECT timestamp, power_w, voltage, current
    FROM energy_data
    ORDER BY timestamp DESC
    LIMIT 1
    "#
)
.fetch_one(&pool)
.await?
```

### TimescaleDB Optimization

Leverages TimescaleDB features:
- **Hypertables**: Automatic partitioning by time
- **Continuous Aggregates**: Pre-computed 5-minute aggregates for 24h queries
- **Compression**: Automatic compression for data older than 7 days

## API Layer Structure

```
src/
├── main.rs           # App initialization, routing
├── handlers/         # Route handlers
│   ├── energy.rs
│   ├── heatpump.rs
│   └── temperature.rs
├── models/           # Data models
│   └── telemetry.rs
├── middleware/       # Auth, CORS
│   └── auth.rs
└── database/         # DB queries
    └── queries.rs
```

## Deployment

### Container Build

Multi-stage Dockerfile with dependency caching:

```dockerfile
# Stage 1: Build dependencies (cached)
FROM rust:1.83 AS builder
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Stage 2: Build application
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/homelab-api /usr/local/bin/
CMD ["homelab-api"]
```

### Kubernetes Deployment

- **Namespace**: `homelab-api`
- **Replicas**: 2 (high availability)
- **Resources**: 100m CPU, 128Mi memory (efficient Rust footprint)
- **Probes**: Liveness and readiness on `/health`
- **Ingress**: Traefik with TLS via Cloudflare Tunnel

## Performance Characteristics

- **Latency**: <50ms typical response time
- **Throughput**: Handles 100+ req/s per pod
- **Memory**: ~30MB RSS per pod (Rust efficiency)
- **CPU**: Minimal usage due to async I/O

## Security

1. **JWT Authentication**: All endpoints protected
2. **Read-only queries**: No data modification possible
3. **SQL injection**: Protected via prepared statements (sqlx)
4. **CORS**: Restricted to known frontends
5. **TLS**: All traffic encrypted via Cloudflare Tunnel

## Monitoring

- **Logs**: Structured JSON logging to stdout
- **Metrics**: Prometheus metrics endpoint (planned)
- **Tracing**: Request ID correlation (planned)
- **Health checks**: `/health` endpoint for k8s probes
