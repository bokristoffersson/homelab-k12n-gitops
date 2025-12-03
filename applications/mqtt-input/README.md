# mqtt-input

Ingest JSON payloads from MQTT and publish to Redpanda topics using configurable JSONPath mappings.

## Features
- MQTT **v5** client (TLS-ready), async ingestion
- YAML-configured pipelines (MQTT topics → Redpanda topics)
- **Bit flag parsing** – decode 8-bit status/alarm bytes into individual boolean fields
- Immediate publishing to Redpanda via `rdkafka`
- Structured logging with `tracing`
- Externalized config for k8s (via `APP_CONFIG`), Redpanda brokers override via `REDPANDA_BROKERS`

---

## Build the Docker image

```bash
# From project root
docker build -t mqtt-input:0.1.0 .
```

Push to GitHub Container Registry (GHCR):

```bash
docker tag mqtt-input:0.1.0 ghcr.io/<your-org>/mqtt-input:0.1.0
docker push ghcr.io/<your-org>/mqtt-input:0.1.0
```

> The image expects a config file mounted at `/config/config.yaml`. You can change the path using `APP_CONFIG`.

---

## Import the image into a k3s cluster (no external registry)

```bash
# Save and compress
docker save mqtt-input:0.1.0 | gzip > mqtt-input_0.1.0.tar.gz

# Copy to the k3s node and import
scp mqtt-input_0.1.0.tar.gz user@<k3s-node>:/tmp
ssh user@<k3s-node> "sudo k3s ctr images import /tmp/mqtt-input_0.1.0.tar.gz"
```

Then set your Deployment image to `mqtt-input:0.1.0`.

> If you do have GHCR/ACR/ECR, prefer pulling images directly in k3s with an ImagePullSecret.

---

## Kubernetes manifests (samples)

Sample manifests are in `k8s/`:
- `configmap.yaml` – your pipelines + MQTT settings (mounted to `/config/config.yaml`)
- `secret.yaml` – `REDPANDA_BROKERS` overwrite for Redpanda connection (optional)
- `deployment.yaml` – mounts ConfigMap, reads Secret, runs the app

Apply:

```bash
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/deployment.yaml
```

> The app reads the YAML path from `APP_CONFIG` (defaults to `/config/config.yaml`). If `REDPANDA_BROKERS` env is set, it overrides the `redpanda.brokers` in the YAML.

---

## Configure pipelines (YAML)

Each pipeline binds an MQTT topic filter to a Redpanda topic and defines how to map fields from the JSON payload.

```yaml
redpanda:
  brokers: "redpanda.redpanda.svc.cluster.local:9092"

pipelines:
  - name: "heatpump"
    topic: "home/heatpump/telemetry"   # supports + and # wildcards
    qos: 1
    redpanda_topic: "heatpump-telemetry"
    timestamp:
      path: "$.timestamp"               # rfc3339 | iso8601 | unix_ms | unix_s
      format: "rfc3339"
      use_now: true
    tags:                                # string metadata
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
    
    # Optional: throttle publishing frequency
    store_interval: "MINUTE"  # Only publish one message per minute
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
- All decoded flags are published as boolean fields in the JSON message

**Example**: If `status_byte: 21` (binary `0b00010101`), the bits 0, 2, and 4 are set:
- `compressor_on: true`
- `heating_mode: false`
- `hot_water_mode: true`
- `circulation_pump: true`

**Nested JSON objects**:
- Extract attributes from JSON objects into separate fields
- Useful for structured data like energy consumption per phase
- Each attribute maps to a separate field with the specified name

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

This creates four separate fields: `consumption_total_w`, `consumption_l1_w`, `consumption_l2_w`, `consumption_l3_w`.

**Message Format**: Messages published to Redpanda have the following JSON structure:

```json
{
  "ts": "2025-10-13T11:00:00Z",
  "tags": {
    "device_id": "hp-01",
    "room": "utility"
  },
  "fields": {
    "flow_temp_c": 38.2,
    "return_temp_c": 31.8,
    "power_w": 980,
    "compressor_on": true
  }
}
```

After updating the ConfigMap, roll the deployment:

```bash
kubectl apply -f k8s/configmap.yaml
kubectl rollout restart deployment mqtt-input -n <namespace>
```

---

## Local development

### Option 1: Using docker-compose

```bash
docker compose up -d mosquitto redpanda

# Run
cargo run
```

### Option 2: Start Redpanda manually with Docker

If you prefer to run Redpanda standalone (without docker-compose):

**Note:** If you encounter Docker connection errors, ensure Docker Desktop is fully started:
```bash
# Check Docker is running
docker ps

# If you get connection errors, try:
# 1. Restart Docker Desktop from the menu
# 2. Wait 10-20 seconds for it to fully start
# 3. Verify: docker info
```

```bash
# Start Redpanda container
docker run -d \
  --name redpanda \
  -p 9092:9092 \
  docker.redpanda.com/redpandadata/redpanda:v24.2.19 \
  redpanda start \
  --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:9092 \
  --advertise-kafka-addr internal://localhost:9092,external://localhost:9092 \
  --pandaproxy-addr internal://0.0.0.0:8082,external://0.0.0.0:8082 \
  --advertise-pandaproxy-addr internal://localhost:8082,external://localhost:8082 \
  --schema-registry-addr internal://0.0.0.0:8081,external://0.0.0.0:8081 \
  --rpc-addr localhost:33145 \
  --advertise-rpc-addr localhost:33145 \
  --smp 1 \
  --memory 1G \
  --mode dev-container \
  --default-log-level=info

# Wait for Redpanda to be ready (check logs)
docker logs -f redpanda

# Stop Redpanda when done
docker stop redpanda
docker rm redpanda
```

**Verify Redpanda is running:**
```bash
# Check if port 9092 is listening
nc -z localhost 9092 && echo "Redpanda is ready" || echo "Redpanda is not ready"

# Or check container logs
docker logs redpanda | grep "Started Redpanda"
```

**Create topics (if needed):**
```bash
docker exec redpanda rpk topic create heatpump-telemetry --brokers localhost:9092
```

### Testing

Publish a test message to MQTT:

```bash
mosquitto_pub -h localhost -t home/heatpump/telemetry \
  -m '{"device_id":"hp-01","timestamp":"2025-10-13T11:00:00Z","flow_temp":38.2,"return_temp":31.8,"power":980,"room":"utility","status_byte":21}'
```

Consume from Redpanda to verify:

```bash
# Using docker-compose
docker compose exec redpanda rpk topic consume heatpump-telemetry --brokers localhost:9092

# Or if running standalone container
docker exec redpanda rpk topic consume heatpump-telemetry --brokers localhost:9092
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
          images: ghcr.io/${{ github.repository }}/mqtt-input
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
- Put `REDPANDA_BROKERS` in a Secret if needed; keep broker creds (if any) out of ConfigMap
- Ensure Redpanda topics are created before the application starts (or enable auto-creation)
- Add Prometheus metrics (publish latency, error rates) if you need observability
- Consider message key strategy for partitioning (currently uses pipeline name as key)
