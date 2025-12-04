# Testing Guide for redpanda-sink

This guide explains how to test the redpanda-sink application locally before committing and pushing.

## Quick Start

### 1. Unit and Integration Tests (No External Dependencies)

These tests don't require any running services:

```bash
cd applications/redpanda-sink

# Run all unit and integration tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_config_loading
```

### 2. Build the Application

```bash
# Build in release mode
cargo build --release

# The binary will be at: target/release/redpanda-sink
```

## Local Testing with Docker

### Step 1: Start Required Services

You'll need Redpanda and PostgreSQL running. Use Docker Compose or run them separately:

#### Option A: Docker Compose (Recommended)

Create a `docker-compose.test.yml` in the application directory:

```yaml
version: '3.8'
services:
  redpanda:
    image: docker.redpanda.com/redpandadata/redpanda:v24.2.19
    command:
      - redpanda
      - start
      - --kafka-addr
      - internal://0.0.0.0:9092,external://0.0.0.0:9092
      - --advertise-kafka-addr
      - internal://localhost:9092,external://localhost:9092
      - --smp
      - "1"
      - --memory
      - "1G"
      - --mode
      - dev-container
    ports:
      - "9092:9092"
  
  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: test
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5
```

Start services:
```bash
docker-compose -f docker-compose.test.yml up -d
```

Wait for services to be ready:
```bash
# Check Redpanda
docker-compose -f docker-compose.test.yml logs redpanda | grep "Started Kafka API server"

# Check PostgreSQL
docker-compose -f docker-compose.test.yml exec postgres pg_isready -U postgres
```

#### Option B: Run Services Separately

**Start Redpanda:**
```bash
docker run -d --name redpanda-test \
  -p 9092:9092 \
  docker.redpanda.com/redpandadata/redpanda:v24.2.19 \
  redpanda start \
  --kafka-addr 0.0.0.0:9092 \
  --advertise-kafka-addr localhost:9092 \
  --smp 1 --memory 1G --mode dev-container
```

**Start PostgreSQL:**
```bash
docker run -d --name postgres-test \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=test \
  postgres:15
```

### Step 2: Set Up Database Schema

Run the migration scripts. Migration files are now in GitOps at `../../gitops/apps/base/redpanda-sink/migrations/`:

```bash
# Connect to PostgreSQL (using GitOps migrations)
docker exec -i postgres-test psql -U postgres -d test < ../../gitops/apps/base/redpanda-sink/migrations/001_init_timeseries_tables.sql
docker exec -i postgres-test psql -U postgres -d test < ../../gitops/apps/base/redpanda-sink/migrations/002_init_static_tables.sql
```

Or if using docker-compose:
```bash
docker-compose -f docker-compose.test.yml exec -T postgres psql -U postgres -d test < ../../gitops/apps/base/redpanda-sink/migrations/001_init_timeseries_tables.sql
docker-compose -f docker-compose.test.yml exec -T postgres psql -U postgres -d test < ../../gitops/apps/base/redpanda-sink/migrations/002_init_static_tables.sql
```

**Note:** For local testing, you can also create tables inline in your test scripts (as done in the test files) or copy the migration files to a local directory if needed.

### Step 3: Create Test Configuration

Create a test config file `config/config.test.yaml`:

```yaml
redpanda:
  brokers: "localhost:9092"
  group_id: "redpanda-sink-test"
  auto_offset_reset: "earliest"

database:
  url: "postgres://postgres:postgres@localhost:5432/test"
  write:
    batch_size: 10  # Smaller batch for testing
    linger_ms: 100

pipelines:
  - name: "test-telemetry"
    topic: "test-telemetry"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags:
      device_id: "$.device_id"
    fields:
      temperature_c:
        path: "$.temperature"
        type: "float"
      power_w:
        path: "$.power"
        type: "int"
  
  - name: "test-devices"
    topic: "test-devices"
    table: "heatpump_devices"
    data_type: "static"
    upsert_key: ["device_id"]
    timestamp:
      use_now: true
    tags:
      device_id: "$.device_id"
    fields:
      name:
        path: "$.name"
        type: "text"
      status:
        path: "$.status"
        type: "text"
```

### Step 4: Run End-to-End Tests

```bash
# Set environment variables
export REDPANDA_BROKERS=localhost:9092
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/test

# Run end-to-end tests
cargo test --test end_to_end_test -- --ignored --nocapture
```

### Step 5: Manual Testing

#### Test 1: Publish Messages to Redpanda

You can use `rpk` (Redpanda CLI) or any Kafka client. First, install rpk or use a simple producer:

**Using rpk (if installed):**
```bash
# Create topic
rpk topic create test-telemetry --brokers localhost:9092

# Produce a message
echo '{"device_id":"test-001","temperature":21.5,"power":950}' | \
  rpk topic produce test-telemetry --brokers localhost:9092
```

**Using a simple Python script:**
```python
from kafka import KafkaProducer
import json

producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    value_serializer=lambda v: json.dumps(v).encode('utf-8')
)

# Send timeseries data
producer.send('test-telemetry', {
    'device_id': 'test-001',
    'temperature': 21.5,
    'power': 950
})

# Send static data
producer.send('test-devices', {
    'device_id': 'hp-001',
    'name': 'Heatpump 1',
    'status': 'active'
})

producer.flush()
```

#### Test 2: Run the Application

```bash
# Set environment variables
export REDPANDA_BROKERS=localhost:9092
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/test
export APP_CONFIG=config/config.test.yaml
export RUST_LOG=debug

# Run the application
cargo run --release
```

The application will:
1. Connect to Redpanda
2. Subscribe to topics
3. Consume messages
4. Store them in the database

#### Test 3: Verify Data in Database

```bash
# Check timeseries data
docker exec -it postgres-test psql -U postgres -d test -c \
  "SELECT * FROM telemetry ORDER BY ts DESC LIMIT 5;"

# Check static data
docker exec -it postgres-test psql -U postgres -d test -c \
  "SELECT * FROM heatpump_devices;"
```

### Step 6: Test Upsert Functionality

For static data, test that updates work:

```bash
# Send initial message
echo '{"device_id":"hp-001","name":"Heatpump 1","status":"active"}' | \
  rpk topic produce test-devices --brokers localhost:9092

# Wait a moment, then send update
sleep 2
echo '{"device_id":"hp-001","name":"Heatpump 1 Updated","status":"inactive"}' | \
  rpk topic produce test-devices --brokers localhost:9092

# Verify only one row exists with updated values
docker exec -it postgres-test psql -U postgres -d test -c \
  "SELECT * FROM heatpump_devices WHERE device_id = 'hp-001';"
```

## Cleanup

After testing:

```bash
# Stop and remove containers
docker-compose -f docker-compose.test.yml down

# Or if using separate containers
docker stop redpanda-test postgres-test
docker rm redpanda-test postgres-test
```

## Testing Checklist

Before committing, verify:

- [ ] All unit tests pass: `cargo test`
- [ ] Integration tests pass: `cargo test --test integration_test`
- [ ] Code compiles without warnings: `cargo build --release`
- [ ] Application connects to Redpanda successfully
- [ ] Application connects to PostgreSQL successfully
- [ ] Timeseries data is inserted correctly
- [ ] Static data upsert works correctly
- [ ] Consumer group management works (test with multiple instances)
- [ ] Error handling works (test with invalid messages)

## Troubleshooting

### Redpanda Connection Issues

```bash
# Check if Redpanda is running
docker ps | grep redpanda

# Check Redpanda logs
docker logs redpanda-test

# Test connection
rpk cluster info --brokers localhost:9092
```

### Database Connection Issues

```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Test connection
docker exec -it postgres-test psql -U postgres -d test -c "SELECT 1;"

# Check connection string format
# Should be: postgres://user:password@host:port/database
```

### Application Issues

```bash
# Run with debug logging
export RUST_LOG=debug
cargo run --release

# Check for compilation errors
cargo check

# Check for clippy warnings
cargo clippy -- -D warnings
```

