# Redpanda Sink Tests

This directory contains tests for the redpanda-sink application.

## Test Structure

- **Unit Tests**: Located in source files with `#[cfg(test)]` modules
  - `src/config.rs` - Configuration loading and validation
  - `src/mapping.rs` - Data extraction and transformation
  - `src/db.rs` - Database operations (insert/upsert)
  - `src/ingest.rs` - Batch processing and interval throttling
  - `src/redpanda.rs` - Consumer configuration

- **Integration Tests**: `tests/integration_test.rs`
  - Config loading with environment variable overrides
  - Message transformation and field extraction
  - Pipeline topic matching
  - Timestamp extraction
  - Data type validation

- **End-to-End Tests**: `tests/end_to_end_test.rs`
  - Full pipeline: Redpanda → Consumer → Database
  - Tests both timeseries and static data flows
  - Requires running Redpanda and PostgreSQL instances

## Running Tests

### Unit and Integration Tests

```bash
# Run all unit and integration tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_config_loading
```

### End-to-End Tests

End-to-end tests are marked with `#[ignore]` and require external services:

1. **Start Redpanda**:
   ```bash
   docker run -d -p 9092:9092 \
     docker.redpanda.com/redpandadata/redpanda:v24.2.19 \
     redpanda start \
     --kafka-addr 0.0.0.0:9092 \
     --advertise-kafka-addr localhost:9092 \
     --smp 1 \
     --memory 1G \
     --mode dev-container
   ```

2. **Start PostgreSQL**:
   ```bash
   docker run -d -p 5432:5432 \
     -e POSTGRES_PASSWORD=postgres \
     postgres:15
   
   # Create test database
   docker exec -it <container-id> psql -U postgres -c 'CREATE DATABASE test;'
   ```

3. **Run end-to-end tests**:
   ```bash
   REDPANDA_BROKERS=localhost:9092 \
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/test \
   cargo test --test end_to_end_test -- --ignored --nocapture
   ```

## Test Coverage

- ✅ Configuration loading and validation
- ✅ Environment variable overrides
- ✅ Message transformation (JSON extraction)
- ✅ Topic matching with wildcards
- ✅ Timestamp parsing (RFC3339, Unix, ISO8601)
- ✅ Bit flag extraction
- ✅ Nested JSON field extraction
- ✅ Data type handling (timeseries vs static)
- ✅ Database insert operations
- ✅ Database upsert operations
- ✅ Batch processing
- ✅ Interval throttling

## CI/CD Considerations

End-to-end tests are excluded from regular CI runs by default. To run them in CI:

1. Ensure Docker is available
2. Use testcontainers or similar for service orchestration
3. Run with: `cargo test --test end_to_end_test -- --ignored`

