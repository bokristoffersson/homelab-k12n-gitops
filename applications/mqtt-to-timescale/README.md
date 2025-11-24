# mqtt-to-timescale

Ingest JSON payloads from MQTT into TimescaleDB using configurable JSONPath mappings.

## Features
- MQTT **v5** client (TLS-ready), async ingestion
- YAML-configured pipelines (topics → Timescale tables/columns)
- **Bit flag parsing** – decode 8-bit status/alarm bytes into individual boolean fields
- Batched inserts via `sqlx`
- Structured logging with `tracing`
- Externalized config for k8s (via `APP_CONFIG`), DB URL override via `DATABASE_URL`

---

## Build the Docker image

```bash
# From project root
docker build -t mqtt-to-timescale:0.1.0 .
```

Push to GitHub Container Registry (GHCR):

```bash
docker tag mqtt-to-timescale:0.1.0 ghcr.io/<your-org>/mqtt-to-timescale:0.1.0
docker push ghcr.io/<your-org>/mqtt-to-timescale:0.1.0
```

> The image expects a config file mounted at `/config/config.yaml`. You can change the path using `APP_CONFIG`.

---

## Import the image into a k3s cluster (no external registry)

```bash
# Save and compress
docker save mqtt-to-timescale:0.1.0 | gzip > mqtt-to-timescale_0.1.0.tar.gz

# Copy to the k3s node and import
scp mqtt-to-timescale_0.1.0.tar.gz user@<k3s-node>:/tmp
ssh user@<k3s-node> "sudo k3s ctr images import /tmp/mqtt-to-timescale_0.1.0.tar.gz"
```

Then set your Deployment image to `mqtt-to-timescale:0.1.0`.

> If you do have GHCR/ACR/ECR, prefer pulling images directly in k3s with an ImagePullSecret.

---

## Kubernetes manifests (samples)

Sample manifests are in `k8s/`:
- `configmap.yaml` – your pipelines + MQTT settings (mounted to `/config/config.yaml`)
- `secret.yaml` – `DATABASE_URL` overwrite for secure DB credentials
- `deployment.yaml` – mounts ConfigMap, reads Secret, runs the app

Apply:

```bash
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/deployment.yaml
```

> The app reads the YAML path from `APP_CONFIG` (defaults to `/config/config.yaml`). If `DATABASE_URL` env is set, it overrides the `database.url` in the YAML.

---

## Configure pipelines (YAML)

Each pipeline binds an MQTT topic filter to a TimescaleDB table and defines how to map fields from the JSON payload.

```yaml
pipelines:
  - name: "heatpump"
    topic: "home/heatpump/telemetry"   # supports + and # wildcards
    qos: 1
    table: "telemetry"
    timestamp:
      path: "$.timestamp"               # rfc3339 | iso8601 | unix_ms | unix_s
      format: "rfc3339"
      use_now: true
    tags:                                # text columns
      device_id: "$.device_id"
      room: "$.room"
    fields:                              # typed metrics
      flow_temp_c:   { path: "$.flow_temp",   type: "float" }
      return_temp_c: { path: "$.return_temp", type: "float" }
      power_w:       { path: "$.power",       type: "int" }
    
    # Optional: decode byte values into individual boolean flags
    bit_flags:
      - source_path: "$.status_byte"
        flags:
          0: "compressor_on"
          1: "heating_mode"
          2: "hot_water_mode"
          3: "defrost_mode"
          4: "circulation_pump"
          5: "alarm_active"
          6: "smart_grid_signal"
          7: "summer_mode"
      
      # You can decode multiple bytes in the same pipeline
      - source_path: "$.alarm_byte"
        flags:
          0: "high_pressure_alarm"
          1: "low_pressure_alarm"
          2: "flow_sensor_error"
          3: "temp_sensor_error"
```

**Allowed field types**: `float`, `int`, `bool`, `text`, `nested`.

**Timestamps**:
- `rfc3339`: ISO8601 string with timezone (e.g., `2025-10-13T11:00:00Z`)
- `iso8601`: ISO8601 string without timezone, assumed UTC (e.g., `2025-10-25T18:48:26`)
- `unix_ms`: milliseconds since epoch (number)
- `unix_s`: seconds since epoch (number)
- If `path` is missing and `use_now: true`, the current time is used.

**Bit flags**:
- Decodes byte values (0-255) into individual boolean fields
- Each bit position (0-7) maps to a named field, where bit 0 is the least significant bit (LSB)
- You only need to specify the bits you care about (sparse mapping)
- Multiple byte sources can be decoded in the same pipeline
- All decoded flags are stored as boolean fields in the database

**Example**: If `status_byte: 21` (binary `0b00010101`), the bits 0, 2, and 4 are set:
- `compressor_on: true`
- `heating_mode: false`
- `hot_water_mode: true`
- `circulation_pump: true`

**Nested JSON objects**:
- Extract attributes from JSON objects into separate database columns
- Useful for structured data like energy consumption per phase
- Each attribute maps to a separate column with the specified name

**Example**: For energy meter data with `"activeActualConsumption":{"total":622,"L1":299,"L2":194,"L3":128}`:

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

This creates four separate columns: `consumption_total_w`, `consumption_l1_w`, `consumption_l2_w`, `consumption_l3_w`.

After updating the ConfigMap, roll the deployment:

```bash
kubectl apply -f k8s/configmap.yaml
kubectl rollout restart deployment mqtt-to-timescale -n telemetry
```

---

## Local development with docker-compose

```bash
docker compose up -d mosquitto timescaledb
psql postgres://postgres:postgres@localhost:5432/postgres -f migrations/001_init_telemetry.sql

# Run
cargo run
```

Publish a test message:

```bash
mosquitto_pub -h localhost -t home/heatpump/telemetry \
  -m '{"device_id":"hp-01","timestamp":"2025-10-13T11:00:00Z","flow_temp":38.2,"return_temp":31.8,"power":980,"room":"utility","status_byte":21}'
```

---

## GitHub Actions (build & push image)

Create `.github/workflows/docker-build.yml`:

```yaml
name: CI - Build & Push Image

on:
  push:
    branches: [ "main" ]
    tags: [ "v*" ]
  pull_request:

permissions:
  contents: read
  packages: write

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Docker metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}/mqtt-to-timescale
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=sha
            type=semver,pattern={{version}}
            type=raw,value=latest,enable={{is_default_branch}}

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: .
          push: ${{ github.event_name != 'pull_request' }}
          platforms: linux/amd64,linux/arm64
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

**Notes**
- Multi-platform (amd64/arm64) for mixed k3s nodes
- Automatic tags: `latest` on default branch, branch names, PRs, tags like `v0.1.0`, and a `sha` tag
- Layer caching via GitHub Actions cache backend

---

## Security & Ops tips
- Put `DATABASE_URL` in a Secret; keep broker creds (if any) out of ConfigMap
- Consider a narrow schema if you have many dynamic fields (ts, tags…, metric, value)
- Add Prometheus metrics (batch size, insert latency) if you need observability