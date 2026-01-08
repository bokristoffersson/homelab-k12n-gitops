# Development Guide

## Prerequisites

- Rust 1.83+ (`rustup install stable`)
- PostgreSQL/TimescaleDB instance
- kubectl access to cluster (for testing)

## Local Development

### Environment Setup

Create `.env` file:

```bash
DATABASE_URL=postgresql://user:password@localhost:5432/homelab
JWKS_URL=https://auth.k12n.com/application/o/homelab-api/jwks/
RUST_LOG=info
PORT=8080
```

### Database Connection

Forward TimescaleDB port from cluster:

```bash
kubectl port-forward -n timescaledb svc/timescaledb-primary 5432:5432
```

Then update `.env`:

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/homelab
```

### Run Development Server

```bash
# Install dependencies and run
cargo run

# With auto-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

Server starts on `http://localhost:8080`.

### Testing Endpoints

```bash
# Get JWT token from Authentik first, then:
export TOKEN="your-jwt-token"

# Test energy endpoint
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/energy/latest

# Test with pretty JSON output
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/energy/24h | jq
```

## Code Quality

### Format Code

```bash
cargo fmt
```

Always run before committing (enforced in CI).

### Linting

```bash
cargo clippy --all-targets --all-features
```

CI fails on warnings, so fix all clippy suggestions.

### Run Tests

```bash
cargo test --verbose
```

## Building

### Local Build

```bash
cargo build --release
```

Binary output: `target/release/homelab-api`

### Docker Build

The Dockerfile uses layered builds for dependency caching:

```bash
# Build ARM64 image (for Raspberry Pi cluster)
docker build --platform linux/arm64 -t homelab-api:test .
```

First build: ~30 minutes (compiles all dependencies)
Subsequent builds: ~5 minutes (only recompiles changed code)

### Deployment

GitHub Actions workflow (`.github/workflows/homelab-api.yml`) automatically:

1. Runs tests on self-hosted ARM64 runner
2. Builds Docker image with BuildKit
3. Pushes to ghcr.io
4. FluxCD detects new image and deploys

## Adding New Endpoints

### 1. Define Model

Add struct in `src/models/telemetry.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NewData {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}
```

### 2. Create Query

Add query in handler file `src/handlers/newdata.rs`:

```rust
pub async fn get_latest(
    State(pool): State<PgPool>,
) -> Result<Json<NewData>, StatusCode> {
    let data = sqlx::query_as!(
        NewData,
        "SELECT timestamp, value FROM table ORDER BY timestamp DESC LIMIT 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(data))
}
```

### 3. Add Route

In `src/main.rs`:

```rust
let app = Router::new()
    .route("/api/v1/newdata/latest", get(handlers::newdata::get_latest))
    .with_state(pool);
```

### 4. Test

```bash
cargo run
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/newdata/latest
```

## Debugging

### Enable Detailed Logging

```bash
RUST_LOG=debug cargo run
```

### Database Query Logging

```bash
RUST_LOG=sqlx=debug cargo run
```

Shows all SQL queries with execution time.

### Connection Pool Stats

Check sqlx pool metrics in logs:

```
INFO sqlx::pool: connection established [connections=3/5]
```

## Performance Profiling

### Benchmarking

Use cargo-criterion for benchmarks:

```bash
cargo install cargo-criterion
cargo criterion
```

### Query Performance

Check TimescaleDB query plans:

```sql
EXPLAIN ANALYZE
SELECT * FROM energy_data
ORDER BY timestamp DESC
LIMIT 1;
```

## Troubleshooting

### "Connection refused" to Database

Ensure port-forward is active:

```bash
kubectl port-forward -n timescaledb svc/timescaledb-primary 5432:5432
```

### JWT Validation Fails

1. Check JWKS_URL is accessible
2. Verify token hasn't expired
3. Ensure token was issued by Authentik

### CORS Errors in Browser

Add your dev URL to CORS middleware in `src/main.rs`:

```rust
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    // ...
```

## Contributing

1. Create feature branch
2. Make changes
3. Run `cargo fmt` and `cargo clippy`
4. Run tests: `cargo test`
5. Push and create PR
6. CI must pass before merge

## Related Documentation

- [Architecture](architecture.md) - System design and patterns
- [API Reference](api.md) - Complete endpoint documentation
- [TimescaleDB Docs](../../gitops/apps/base/timescaledb) - Database schema
