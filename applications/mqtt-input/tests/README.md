# Integration Tests

This directory contains integration tests for the `mqtt-input` application.

## Test Structure

### `integration_test.rs`
Unit and integration tests that don't require external services:
- Configuration loading and validation
- Message transformation
- Pipeline matching
- Bit flag parsing
- Nested field extraction
- Timestamp parsing
- Interval throttling (requires Redpanda, but gracefully skips if unavailable)

These tests run in CI and don't require Docker.

### `end_to_end_test.rs`
End-to-end tests using testcontainers that spin up real MQTT and Redpanda containers:
- Full MQTT → Redpanda message flow
- Interval throttling in real scenario

These tests are marked with `#[ignore]` and require Docker to be running.

## Running Tests

### Run all tests (excluding ignored)
```bash
cargo test
```

### Run only integration tests
```bash
cargo test --test integration_test
```

### Run end-to-end tests (requires Docker and Redpanda)
```bash
# Option 1: Start Redpanda using docker-compose (from project root)
cd ../..
docker compose -f applications/mqtt-input/docker-compose.yml up -d redpanda

# Option 2: Start Redpanda manually
docker run -d \
  --name redpanda-test \
  -p 9092:9092 \
  docker.redpanda.com/redpandadata/redpanda:v24.2.19 \
  redpanda start \
  --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:9092 \
  --advertise-kafka-addr internal://localhost:9092,external://localhost:9092 \
  --smp 1 \
  --memory 1G \
  --mode dev-container

# Wait for Redpanda to be ready
sleep 5
docker logs redpanda-test | grep "Started Redpanda" || echo "Waiting for Redpanda..."

# Run the tests
cd applications/mqtt-input
REDPANDA_BROKERS=localhost:9092 cargo test --test end_to_end_test -- --ignored

# Cleanup (if using manual container)
docker stop redpanda-test && docker rm redpanda-test
```

### Run specific test
```bash
cargo test --test integration_test test_config_loading
```

## CI Integration

The GitHub Actions CI workflow runs:
- Unit tests (`cargo test --lib`)
- Integration tests (`cargo test --test '*'`)
- Formatting check (`cargo fmt --all -- --check`)
- Clippy linting (`cargo clippy --all-targets --all-features -- -D warnings`)

End-to-end tests are **not** run in CI by default as they require Docker and testcontainers. They can be run manually or in a separate workflow that has Docker available.

## Test Dependencies

- `tokio-test`: Async testing utilities
- `mockall`: Mocking framework (for future use)
- `testcontainers`: Docker container management for E2E tests
- `rdkafka`: Kafka/Redpanda client for E2E tests

## Writing New Tests

### Integration Test Example
```rust
#[tokio::test]
async fn test_my_feature() {
    // Test code here
    assert!(true);
}
```

### End-to-End Test Example
```rust
#[tokio::test]
#[ignore]  // Mark as ignored to skip in regular CI
async fn test_e2e_flow() {
    let docker = clients::Cli::default();
    // Start containers, test, verify
}
```

## Troubleshooting

### Tests fail with "Redpanda not available"
Some tests gracefully skip if Redpanda is not available. This is expected in CI environments without testcontainers.

### End-to-end tests fail

**Redpanda connection refused:**
- Ensure Docker is running: `docker ps`
- Check if Redpanda container is running: `docker ps | grep redpanda`
- Verify Redpanda is listening on port 9092: `nc -z localhost 9092`
- Check Redpanda logs: `docker logs redpanda-test` (or your container name)
- Wait longer for Redpanda to start (it can take 10-30 seconds)

**Start Redpanda manually:**
```bash
docker run -d \
  --name redpanda-test \
  -p 9092:9092 \
  docker.redpanda.com/redpandadata/redpanda:v24.2.19 \
  redpanda start \
  --kafka-addr 0.0.0.0:9092 \
  --advertise-kafka-addr localhost:9092 \
  --smp 1 \
  --memory 1G \
  --mode dev-container

# Wait and check logs
sleep 10
docker logs redpanda-test
```

### Port conflicts
If you see port binding errors, another service may be using port 9092:
- Check what's using the port: `lsof -i :9092` or `netstat -an | grep 9092`
- Stop conflicting services or use a different port
- If using docker-compose, modify the port mapping in `docker-compose.yml`

