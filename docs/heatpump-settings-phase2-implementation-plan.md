# Heatpump Settings Phase 2 - Incremental Implementation Plan

**Status:** In Progress
**Approach:** Option 2 - Incremental with commits at each milestone
**Last Updated:** 2026-01-12

---

## Overview

This plan breaks down Phase 2 (transactional outbox + edit functionality) into 6 testable milestones. Each milestone can be committed, tested, and verified independently.

---

## Milestone 1: Database Migration ✅ READY TO COMMIT

### What's Done

- ✅ Created `applications/heatpump-settings-api/migrations/002_create_outbox_table.sql`
- ✅ Created `applications/heatpump-settings-api/src/repositories/outbox.rs`
- ✅ Updated `applications/heatpump-settings-api/src/repositories/mod.rs`
- ✅ Updated API models with outbox fields
- ✅ Updated PATCH endpoint with transactional outbox
- ✅ Added GET outbox status endpoint handler

### What's Needed to Complete

1. **Update routes.rs** to add outbox status route:

```rust
// In src/api/routes.rs
.route(
    "/api/v1/heatpump/settings/outbox/{id}",
    get(settings::get_outbox_status),
)
```

2. **Update main.rs** to initialize OutboxRepository in AppState:

```rust
// In src/main.rs, update AppState initialization:
let outbox_repository = Arc::new(OutboxRepository::new(pool.clone()));

let state = AppState {
    repository: Arc::new(settings_repository),
    outbox_repository,
    pool: pool.clone(),
};
```

3. **Run the migration** (when ready to deploy):

```bash
# Connect to PostgreSQL in heatpump-settings namespace
kubectl exec -it -n heatpump-settings postgres-0 -- psql -U postgres -d heatpump_settings

# Run migration
\i /path/to/002_create_outbox_table.sql

# Or use psql from local machine:
kubectl port-forward -n heatpump-settings svc/postgres 5432:5432
psql -h localhost -U postgres -d heatpump_settings -f applications/heatpump-settings-api/migrations/002_create_outbox_table.sql
```

### Testing Milestone 1

1. **Build the API** to catch any compilation errors:
   ```bash
   cd applications/heatpump-settings-api
   cargo build
   ```

2. **Run migration** against test database

3. **Test PATCH endpoint** returns 202 Accepted with outbox_id:
   ```bash
   curl -X PATCH https://heatpump.k12n.com/api/v1/heatpump/settings/device123 \
     -H "Content-Type: application/json" \
     -d '{"indoor_target_temp": 21.5}'

   # Expected response (202 Accepted):
   {
     "device_id": "device123",
     "indoor_target_temp": 21.5,
     ...
     "outbox_id": 1,
     "outbox_status": "pending"
   }
   ```

4. **Verify database state**:
   ```sql
   -- Should have updated settings
   SELECT * FROM settings WHERE device_id = 'device123';

   -- Should have outbox entry
   SELECT * FROM outbox WHERE id = 1;
   -- status should be 'pending'
   ```

5. **Test GET outbox status**:
   ```bash
   curl https://heatpump.k12n.com/api/v1/heatpump/settings/outbox/1

   # Expected response:
   {
     "id": 1,
     "status": "pending",
     "created_at": "2026-01-12T...",
     "published_at": null,
     "confirmed_at": null,
     "error_message": null,
     "retry_count": 0
   }
   ```

### Commit Milestone 1

```bash
git add applications/heatpump-settings-api/
git commit -m "feat(api): implement transactional outbox pattern for settings updates

Phase 2 Milestone 1: Database and API changes

- Add outbox table migration (002_create_outbox_table.sql)
- Create OutboxRepository with CRUD operations
- Update PATCH endpoint to use transaction (settings + outbox atomic write)
- Add GET /outbox/{id} endpoint for status polling
- Return 202 Accepted with outbox_id when settings updated

Benefits:
- Atomic write ensures settings and command are saved together
- No lost updates if MQTT publish fails
- Audit trail of all setting changes
- Foundation for outbox processor service

Next: Milestone 2 - Outbox processor service

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Milestone 2: Outbox Processor Service (Structure Only)

### Goal

Create the Rust service that will process outbox entries. Start with basic structure, no MQTT/Kafka yet.

### Files to Create

1. **applications/heatpump-settings-outbox-processor/Cargo.toml**
2. **applications/heatpump-settings-outbox-processor/src/main.rs** (basic structure)
3. **applications/heatpump-settings-outbox-processor/src/config.rs**
4. **applications/heatpump-settings-outbox-processor/src/db.rs** (connect to database)
5. **applications/heatpump-settings-outbox-processor/Dockerfile**

### Implementation Steps

1. **Create Cargo.toml**:
   ```toml
   [package]
   name = "heatpump-settings-outbox-processor"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   tokio = { version = "1", features = ["full"] }
   sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "json"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   chrono = { version = "0.4", features = ["serde"] }
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   rumqttc = "0.24"  # MQTT client
   rdkafka = { version = "0.36", features = ["tokio"] }
   dotenvy = "0.15"
   ```

2. **Create basic main.rs** (polling loop):
   ```rust
   use std::time::Duration;
   use tokio::time::sleep;
   use tracing::info;

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       tracing_subscriber::fmt::init();

       info!("Starting heatpump-settings-outbox-processor");

       // TODO: Connect to database
       // TODO: Poll outbox table
       // TODO: Publish to MQTT
       // TODO: Listen for Kafka confirmations

       loop {
           info!("Polling outbox table...");
           // TODO: Implement polling logic
           sleep(Duration::from_secs(5)).await;
       }
   }
   ```

3. **Create Dockerfile** (same pattern as heatpump-settings-api)

### Testing Milestone 2

1. **Cargo build** succeeds
2. **Service starts** and logs "Polling outbox table..." every 5 seconds
3. **Docker image builds** successfully

### Commit Milestone 2

```bash
git add applications/heatpump-settings-outbox-processor/
git commit -m "feat(outbox): create outbox processor service structure

Phase 2 Milestone 2: Scaffolding

- Create new Rust service for outbox processing
- Basic polling loop (5 second interval)
- Database connection setup
- Dockerfile with multi-stage build

Next: Milestone 3 - Implement MQTT publishing

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Milestone 3: MQTT Publishing Logic

### Goal

Implement the logic to poll pending outbox entries and publish to MQTT.

### Implementation

1. **Add MQTT connection** in main.rs:
   ```rust
   use rumqttc::{Client, MqttOptions, QoS};

   let mut mqtt_options = MqttOptions::new("outbox-processor", "mosquitto.mosquitto.svc.cluster.local", 1883);
   mqtt_options.set_keep_alive(Duration::from_secs(30));

   let (client, mut eventloop) = Client::new(mqtt_options, 10);
   ```

2. **Implement polling and publishing**:
   ```rust
   async fn process_pending_entries(pool: &PgPool, mqtt_client: &Client) {
       let entries = get_pending_entries(pool, 10).await;

       for entry in entries {
           // Build MQTT message
           let topic = format!("heatpump/{}/command", entry.aggregate_id);
           let payload = entry.payload.to_string();

           // Publish
           match mqtt_client.publish(topic, QoS::AtLeastOnce, false, payload) {
               Ok(_) => {
                   mark_published(pool, entry.id).await;
                   info!("Published outbox entry {}", entry.id);
               }
               Err(e) => {
                   error!("Failed to publish entry {}: {}", entry.id, e);
                   increment_retry(pool, entry.id).await;
               }
           }
       }
   }
   ```

3. **Add retry logic**:
   - Check retry_count < max_retries
   - If exceeded, mark as 'failed'

### Testing Milestone 3

1. **Manual test**:
   - Insert test row into outbox table with status='pending'
   - Start processor
   - Verify MQTT message published (use MQTT client to subscribe)
   - Verify outbox status updated to 'published'

2. **Verify retry logic**:
   - Stop MQTT broker
   - Insert pending entry
   - Verify retry_count increments
   - After 3 retries, verify status='failed'

### Commit Milestone 3

```bash
git commit -m "feat(outbox): implement MQTT publishing logic

Phase 2 Milestone 3: Publish pending commands

- Poll outbox table for pending entries every 5 seconds
- Publish to MQTT topic: heatpump/{device_id}/command
- Update outbox status to 'published' on success
- Retry logic: up to 3 attempts, then mark as 'failed'
- Structured logging for observability

Tested:
- Manual insertion into outbox table
- MQTT message published successfully
- Retry logic handles MQTT broker downtime

Next: Milestone 4 - Kafka confirmation listener

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Milestone 4: Kafka Confirmation Listener

### Goal

Listen to Kafka topic for heatpump telemetry responses and mark outbox entries as 'confirmed'.

### Implementation

1. **Add Kafka consumer**:
   ```rust
   use rdkafka::config::ClientConfig;
   use rdkafka::consumer::{Consumer, StreamConsumer};

   let consumer: StreamConsumer = ClientConfig::new()
       .set("bootstrap.servers", "redpanda-v2.redpanda-v2.svc.cluster.local:9092")
       .set("group.id", "heatpump-settings-outbox-processor")
       .set("enable.auto.commit", "true")
       .create()?;

   consumer.subscribe(&["homelab-heatpump-telemetry"])?;
   ```

2. **Match telemetry to outbox entries**:
   ```rust
   async fn process_kafka_messages(consumer: &StreamConsumer, pool: &PgPool) {
       while let Some(message) = consumer.recv().await {
           match message {
               Ok(msg) => {
                   let payload: TelemetryMessage = serde_json::from_slice(msg.payload().unwrap())?;

                   // Find matching outbox entry
                   // Match by device_id and timestamp proximity (within last 60 seconds)
                   let entry = find_matching_outbox_entry(
                       pool,
                       &payload.device_id,
                       payload.timestamp
                   ).await?;

                   if let Some(entry) = entry {
                       mark_confirmed(pool, entry.id).await?;
                       info!("Confirmed outbox entry {}", entry.id);
                   }
               }
               Err(e) => error!("Kafka error: {}", e),
           }
       }
   }
   ```

3. **Matching logic**:
   - Device ID must match
   - Telemetry timestamp within 60 seconds of `published_at`
   - Compare setting values to confirm match

### Testing Milestone 4

1. **End-to-end test**:
   - PATCH /settings (creates outbox entry)
   - Processor publishes to MQTT
   - Heatpump responds
   - Telemetry appears in Kafka
   - Processor marks outbox as 'confirmed'

2. **Verify GET /outbox/{id}** returns `confirmed` status

### Commit Milestone 4

```bash
git commit -m "feat(outbox): add Kafka confirmation listener

Phase 2 Milestone 4: Close the loop

- Consume homelab-heatpump-telemetry Kafka topic
- Match telemetry to published outbox entries
- Mark entries as 'confirmed' when heatpump responds
- Matching logic: device_id + timestamp proximity

Flow complete:
1. API creates outbox entry (pending)
2. Processor publishes to MQTT (published)
3. Heatpump responds via Kafka
4. Processor confirms (confirmed)

Tested:
- End-to-end flow with real heatpump
- Confirmation within 60 seconds of publish
- GET /outbox/{id} returns 'confirmed' status

Next: Milestone 5 - Deployment and GitOps

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Milestone 5: Deployment & GitOps Integration

### Goal

Deploy outbox processor to Kubernetes and integrate with FluxCD.

### Files to Create

1. **gitops/apps/base/heatpump-settings/outbox-processor-deployment.yaml**
2. **gitops/apps/base/heatpump-settings/outbox-processor-configmap.yaml**
3. **.github/workflows/outbox-processor.yml** (CI/CD)
4. Update **gitops/apps/base/heatpump-settings/kustomization.yaml**

### Implementation

1. **Deployment manifest**:
   ```yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: heatpump-settings-outbox-processor
     namespace: heatpump-settings
   spec:
     replicas: 1
     selector:
       matchLabels:
         app: heatpump-settings-outbox-processor
     template:
       metadata:
         labels:
           app: heatpump-settings-outbox-processor
       spec:
         containers:
           - name: processor
             image: ghcr.io/bokristoffersson/heatpump-settings-outbox-processor:main
             env:
               - name: DATABASE_URL
                 value: "postgresql://postgres:postgres@postgres.heatpump-settings.svc.cluster.local:5432/heatpump_settings"
               - name: MQTT_BROKER
                 value: "mosquitto.mosquitto.svc.cluster.local:1883"
               - name: KAFKA_BROKERS
                 value: "redpanda-v2.redpanda-v2.svc.cluster.local:9092"
               - name: KAFKA_TOPIC
                 value: "homelab-heatpump-telemetry"
               - name: KAFKA_GROUP_ID
                 value: "heatpump-settings-outbox-processor"
               - name: RUST_LOG
                 value: "info"
   ```

2. **GitHub Actions workflow** (similar to heatpump-settings-api.yml)

3. **Update kustomization.yaml**:
   ```yaml
   resources:
     - namespace.yaml
     - postgres-deployment.yaml
     - postgres-service.yaml
     - heatpump-settings-api-deployment.yaml
     - heatpump-settings-api-service.yaml
     - outbox-processor-deployment.yaml  # NEW
   ```

### Testing Milestone 5

1. **Build and push image** via GitHub Actions
2. **FluxCD reconciliation**:
   ```bash
   flux reconcile kustomization heatpump-settings
   ```
3. **Verify pod running**:
   ```bash
   kubectl get pods -n heatpump-settings -l app=heatpump-settings-outbox-processor
   ```
4. **Check logs**:
   ```bash
   kubectl logs -n heatpump-settings -l app=heatpump-settings-outbox-processor --tail=50
   ```

### Commit Milestone 5

```bash
git commit -m "feat(outbox): deploy outbox processor to Kubernetes

Phase 2 Milestone 5: Production deployment

- Kubernetes deployment manifest
- ConfigMap for environment variables
- GitHub Actions CI/CD workflow
- Integrated with FluxCD GitOps

Configuration:
- Single replica (sufficient for homelab scale)
- Connects to PostgreSQL, MQTT, Kafka
- RUST_LOG=info for observability

Next: Milestone 6 - Frontend edit UI

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Milestone 6: Frontend Edit UI & Status Polling

### Goal

Add edit form to Settings page with status polling to show confirmation.

### Files to Update

1. **applications/heatpump-web/src/components/Settings/Settings.tsx**
2. **applications/heatpump-web/src/components/Settings/Settings.css**
3. **applications/heatpump-web/src/services/settings.ts** (already has updateSetting)
4. **applications/heatpump-web/src/types/settings.ts** (add OutboxStatus type)

### Implementation Steps

1. **Add state for edit mode**:
   ```tsx
   const [editMode, setEditMode] = useState(false);
   const [editValues, setEditValues] = useState<SettingPatch>({});
   const [pendingOutboxId, setPendingOutboxId] = useState<number | null>(null);
   ```

2. **Create edit form**:
   ```tsx
   {editMode ? (
     <form onSubmit={handleSubmit}>
       <div className="edit-form">
         <label>
           Indoor Target Temp:
           <input
             type="number"
             step="0.5"
             min="15"
             max="30"
             value={editValues.indoor_target_temp ?? setting.indoor_target_temp}
             onChange={(e) => setEditValues({
               ...editValues,
               indoor_target_temp: parseFloat(e.target.value)
             })}
           />
         </label>
         {/* More fields... */}
         <button type="submit">Save Changes</button>
         <button type="button" onClick={() => setEditMode(false)}>Cancel</button>
       </div>
     </form>
   ) : (
     <button onClick={() => setEditMode(true)}>Edit Settings</button>
   )}
   ```

3. **Handle submit with status polling**:
   ```tsx
   const handleSubmit = async (e: React.FormEvent) => {
     e.preventDefault();

     try {
       const response = await updateSetting(setting.device_id, editValues);

       if (response.outbox_id) {
         setPendingOutboxId(response.outbox_id);
         setEditMode(false);

         // Start polling for status
         pollOutboxStatus(response.outbox_id);
       }
     } catch (error) {
       // Handle error
     }
   };
   ```

4. **Implement status polling**:
   ```tsx
   const pollOutboxStatus = async (outboxId: number) => {
     const maxAttempts = 30; // 30 attempts = 60 seconds
     let attempts = 0;

     const interval = setInterval(async () => {
       attempts++;

       try {
         const status = await getOutboxStatus(outboxId);

         if (status.status === 'confirmed') {
           clearInterval(interval);
           setPendingOutboxId(null);
           refetch(); // Refresh settings
           showSuccess("Settings updated successfully!");
         } else if (status.status === 'failed' || attempts >= maxAttempts) {
           clearInterval(interval);
           setPendingOutboxId(null);
           showError("Settings update failed or timed out");
         }
       } catch (error) {
         clearInterval(interval);
         setPendingOutboxId(null);
         showError("Error checking status");
       }
     }, 2000); // Poll every 2 seconds
   };
   ```

5. **Show status indicator**:
   ```tsx
   {pendingOutboxId && (
     <div className="status-indicator">
       <div className="spinner"></div>
       <span>Sending to heatpump...</span>
     </div>
   )}
   ```

### Testing Milestone 6

1. **Edit settings** via UI
2. **Verify status changes**:
   - "Sending to heatpump..." appears
   - After ~5-10 seconds: "Settings updated successfully!"
   - Settings refresh with new values

3. **Test failure case**:
   - Stop outbox processor
   - Edit settings
   - Verify timeout after 60 seconds

### Commit Milestone 6

```bash
git commit -m "feat(frontend): add edit form with status polling

Phase 2 Milestone 6: Complete UI

- Add edit mode toggle to Settings page
- Form validation (temp 15-30°C, mode 0-3)
- Submit calls PATCH endpoint
- Status polling every 2 seconds
- Visual feedback: pending → confirmed → success
- Timeout after 60 seconds if no confirmation

UX Flow:
1. Click 'Edit Settings'
2. Modify values
3. Click 'Save Changes'
4. Spinner: 'Sending to heatpump...'
5. Success: 'Settings updated successfully!'
6. Settings auto-refresh

Tested:
- Edit and save settings
- Confirmation received from heatpump
- Error handling for timeout/failure

Phase 2 Complete! 🎉

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Current Status

**Milestone 1:** ⏳ In Progress (85% complete)
- Need to update routes.rs and main.rs
- Ready to commit after those changes

**Milestones 2-6:** 📋 Planned

---

## Quick Reference: Resume Points

### To Continue from Milestone 1

```bash
# 1. Update routes.rs (add outbox status route)
# 2. Update main.rs (add OutboxRepository to AppState)
# 3. Build and test:
cd applications/heatpump-settings-api
cargo build
cargo test

# 4. Run migration (when ready):
kubectl port-forward -n heatpump-settings svc/postgres 5432:5432
psql -h localhost -U postgres -d heatpump_settings -f migrations/002_create_outbox_table.sql

# 5. Commit Milestone 1 (see commit message above)
```

### To Start Milestone 2

```bash
# Create new service directory
mkdir -p applications/heatpump-settings-outbox-processor/src

# Follow "Milestone 2: Implementation Steps" above
```

---

## Dependencies & Prerequisites

- ✅ PostgreSQL in heatpump-settings namespace
- ✅ MQTT broker (Mosquitto) running
- ✅ Redpanda with homelab-heatpump-telemetry topic
- ✅ heatpump-settings-api deployed
- ⏳ Outbox table migration (Milestone 1)
- ⏳ Outbox processor service (Milestones 2-4)

---

## Monitoring & Observability

After deployment, monitor:

1. **Outbox table depth**:
   ```sql
   SELECT status, COUNT(*) FROM outbox GROUP BY status;
   ```

2. **Processor logs**:
   ```bash
   kubectl logs -n heatpump-settings -l app=heatpump-settings-outbox-processor -f
   ```

3. **Metrics to track**:
   - Pending entries count
   - Time from pending → confirmed (p50, p95, p99)
   - Failed entry rate
   - Retry count distribution

---

## Rollback Plan

If something breaks:

1. **Milestone 1**: Revert API changes, settings still work (no outbox entries created)
2. **Milestone 3**: Stop processor pod, outbox entries stay pending (can retry later)
3. **Milestone 4**: Kafka issues don't affect MQTT publishing (entries stay in 'published')
4. **Milestone 6**: Frontend edit can be disabled (read-only mode still works)

---

## Documentation Updates Needed

After completion:

1. Update `docs/ARCHITECTURE_PRINCIPLES.md` with transactional outbox example
2. Update `applications/heatpump-settings-api/README.md` with new endpoints
3. Create runbook: `docs/runbooks/heatpump-settings-troubleshooting.md`
4. Update Backstage catalog-info.yaml for outbox processor
5. Add Grafana dashboard for outbox metrics

---

## Next Session Checklist

Before resuming, review:

- [ ] Phase 2 ADR: `docs/adr/001-transactional-outbox-for-heatpump-settings.md`
- [ ] This plan: `docs/heatpump-settings-phase2-implementation-plan.md`
- [ ] Summary: `docs/heatpump-settings-implementation-summary.md`
- [ ] Current milestone progress (check git log)
- [ ] Any open questions or blockers

**Questions to resolve:**
1. ✅ Editable fields scope (Answer: All fields editable)
2. ❓ Timeout duration (Suggested: 60 seconds)
3. ❓ Retry strategy (Suggested: 3 retries with exponential backoff)
4. ❓ Cleanup policy (Suggested: Delete confirmed entries after 30 days)

---

**Happy hacking! 🚀**
