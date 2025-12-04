# redpanda-sink

A data sink that consumes messages from Redpanda topics and stores them in PostgreSQL/TimescaleDB, supporting both timeseries and static data storage patterns.

## Features

- **Redpanda/Kafka consumer** using `rdkafka` with async message processing
- **Dual data type support**:
  - **Timeseries data**: Stored in TimescaleDB hypertables for efficient time-based queries
  - **Static data**: Stored in regular PostgreSQL tables with upsert logic (INSERT ... ON CONFLICT UPDATE)
- YAML-configured pipelines (topics → database tables/columns)
- **Bit flag parsing** – decode 8-bit status/alarm bytes into individual boolean fields
- **Nested JSON extraction** – extract nested objects into multiple columns
- Batched inserts/upserts via `sqlx`
- Structured logging with `tracing`
- Externalized config for k8s (via `APP_CONFIG`), DB URL override via `DATABASE_URL`
- Consumer group management with configurable offset reset strategy

---

## Architecture

This application is a **data sink** in the data pipeline:

```
Source → Pipeline/Stream → Sink
         (Redpanda)      (This app → Database)
```

- **Source**: Where data originates (e.g., MQTT broker, sensors, APIs)
- **Pipeline/Stream**: The flow of data through Redpanda
- **Sink**: This application - consumes from Redpanda and stores in database

---

## Build the Docker image

```bash
# From project root
docker build -t redpanda-sink:0.1.0 .
```

Push to GitHub Container Registry (GHCR):

```bash
docker tag redpanda-sink:0.1.0 ghcr.io/<your-org>/redpanda-sink:0.1.0
docker push ghcr.io/<your-org>/redpanda-sink:0.1.0
```

> The image expects a config file mounted at `/config/config.yaml`. You can change the path using `APP_CONFIG`.

---

## Configure pipelines (YAML)

Each pipeline binds a Redpanda topic to a database table and defines how to map fields from the JSON payload.

### Timeseries Pipeline Example

```yaml
pipelines:
  - name: "telemetry"
    topic: "telemetry"                    # Redpanda topic name
    table: "telemetry"                     # Database table
    data_type: "timeseries"                # Will use hypertable
    timestamp:
      path: "$.timestamp"                  # rfc3339 | iso8601 | unix_ms | unix_s
      format: "rfc3339"
      use_now: true                        # Use current time if path not found
    tags:                                  # Text columns (indexed)
      device_id: "$.device_id"
      room: "$.room"
    fields:                                # Typed metrics
      flow_temp_c:   { path: "$.flow_temp",   type: "float" }
      return_temp_c: { path: "$.return_temp", type: "float" }
      power_w:       { path: "$.power",       type: "int" }
    store_interval: "MINUTE"               # Optional: throttle writes
```

### Static Data Pipeline Example

```yaml
pipelines:
  - name: "devices"
    topic: "devices"
    table: "devices"
    data_type: "static"                    # Will use upsert
    upsert_key: ["device_id"]             # Required: columns for conflict resolution
    timestamp:
      use_now: true
    tags:
      device_id: "$.device_id"
      location: "$.location"
    fields:
      name: { path: "$.name", type: "text" }
      status: { path: "$.status", type: "text" }
      firmware_version: { path: "$.firmware_version", type: "text" }
```

### Advanced Features

#### Bit Flag Parsing

Decode byte values into individual boolean fields:

```yaml
bit_flags:
  - source_path: "$.status_byte"
    flags:
      0: "compressor_on"
      1: "heating_mode"
      2: "hot_water_mode"
      4: "circulation_pump"
```

#### Nested JSON Extraction

Extract nested objects into multiple columns:

```yaml
fields:
  activeActualConsumption:
    path: "$.activeActualConsumption"
    type: "nested"
    attributes:
      total: "consumption_total_w"
      L1: "consumption_l1_w"
      L2: "consumption_l2_w"
      L3: "consumption_l3_w"
```

---

## Database Setup

### Timeseries Tables (Hypertables)

For `data_type: "timeseries"`, create hypertables:

```sql
CREATE TABLE IF NOT EXISTS telemetry
(
    ts          TIMESTAMPTZ       NOT NULL,
    device_id   TEXT,
    flow_temp_c DOUBLE PRECISION,
    power_w     BIGINT
);

-- Convert to hypertable (TimescaleDB)
SELECT create_hypertable('telemetry', 'ts', if_not_exists => TRUE);
```

### Static Tables

For `data_type: "static"`, create tables with unique constraints:

```sql
CREATE TABLE IF NOT EXISTS devices
(
    ts              TIMESTAMPTZ       NOT NULL,
    device_id       TEXT              NOT NULL,
    name            TEXT,
    status          TEXT,
    PRIMARY KEY (device_id)
);
```

**Note:** Migration scripts are now in GitOps at `gitops/apps/base/redpanda-sink/migrations/`. They are automatically executed via Kubernetes Job during deployment.

---

## Configuration

### Redpanda Configuration

```yaml
redpanda:
  brokers: "localhost:9092"              # Comma-separated broker list
  group_id: "redpanda-sink"             # Consumer group ID
  auto_offset_reset: "earliest"         # "earliest", "latest", or "none"
```

### Database Configuration

```yaml
database:
  url: "postgres://user:pass@host:5432/dbname"
  write:
    batch_size: 500                      # Batch size for inserts/upserts
    linger_ms: 500                       # Max wait time before flushing batch
```

### Environment Variable Overrides

- `DATABASE_URL`: Overrides `database.url`
- `REDPANDA_BROKERS`: Overrides `redpanda.brokers`
- `APP_CONFIG`: Overrides config file path (default: `/config/config.yaml`)

---

## Running Locally

```bash
# Set environment variables
export REDPANDA_BROKERS=localhost:9092
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
export APP_CONFIG=config/config.example.yaml

# Run the application
cargo run --release
```

---

## Testing

### Unit and Integration Tests

```bash
cargo test
```

### End-to-End Tests

Requires running Redpanda and PostgreSQL:

```bash
# Start Redpanda
docker run -d -p 9092:9092 \
  docker.redpanda.com/redpandadata/redpanda:v24.2.19 \
  redpanda start \
  --kafka-addr 0.0.0.0:9092 \
  --advertise-kafka-addr localhost:9092 \
  --smp 1 --memory 1G --mode dev-container

# Start PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15

# Run end-to-end tests
REDPANDA_BROKERS=localhost:9092 \
DATABASE_URL=postgres://postgres:postgres@localhost:5432/test \
cargo test --test end_to_end_test -- --ignored
```

See `tests/README.md` for more details.

---

## Data Types

### Timeseries Data

- Stored in TimescaleDB hypertables
- Optimized for time-based queries
- Uses simple `INSERT` operations
- Best for: sensor data, metrics, telemetry

### Static Data

- Stored in regular PostgreSQL tables
- Uses `INSERT ... ON CONFLICT UPDATE` (upsert)
- Requires `upsert_key` configuration
- Best for: device metadata, configuration, state

---

## Consumer Groups

A consumer group is a set of consumers that work together to consume messages from topics:

- Messages are distributed among consumers in the same group (load balancing)
- Each consumer group maintains its own offset (position) in each topic
- If you run multiple instances, use the same `group_id` for load balancing
- Use different `group_id` values if you want separate processing

---

## Migration Scripts

**Note:** Migration scripts have been moved to GitOps for version control and automated execution.

Migration scripts are now located in `gitops/apps/base/redpanda-sink/migrations/`:

- `001_init_timeseries_tables.sql` - Timeseries hypertables matching the pipeline configuration
- `002_init_static_tables.sql` - Static tables with unique constraints matching the pipeline configuration

Migrations are automatically executed via a Kubernetes Job (`migration-job.yaml`) as part of the GitOps deployment process. The job runs before the application deployment to ensure the database schema is up-to-date.

For local development, you can run migrations from the GitOps location:
```bash
# From the application directory
psql $DATABASE_URL -f ../../gitops/apps/base/redpanda-sink/migrations/001_init_timeseries_tables.sql
psql $DATABASE_URL -f ../../gitops/apps/base/redpanda-sink/migrations/002_init_static_tables.sql
```

See `gitops/apps/base/redpanda-sink/` for the GitOps configuration and migration job.

---

## License

MIT

