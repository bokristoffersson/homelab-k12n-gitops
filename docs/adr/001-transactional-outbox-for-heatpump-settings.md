# ADR-001: Transactional Outbox Pattern for Heatpump Settings Updates

**Date:** 2026-01-11
**Status:** Proposed
**Deciders:** Bo Kristoffersson
**Decision Type:** Type 1 (Hard to reverse)

## Context and Problem Statement

Users need to update heatpump settings through the heatpump-web interface. The update flow requires:

1. User submits settings change via web UI
2. Settings are persisted to PostgreSQL database
3. Command is published to MQTT topic for the physical heatpump
4. Heatpump processes the command and returns updated settings via Kafka
5. System validates that settings were applied correctly

**Challenge:** This is a distributed transaction spanning multiple systems:
- PostgreSQL (settings database)
- MQTT (command bus to heatpump)
- Kafka (telemetry stream from heatpump)

Without proper coordination, we risk:
- Settings saved to database but never sent to heatpump (lost updates)
- MQTT publish fails but database already committed (inconsistency)
- Duplicate MQTT messages if retries happen without deduplication
- No confirmation that heatpump actually applied the settings

## Decision Drivers

- **Reliability:** Settings updates must not be lost
- **Consistency:** Database and heatpump must eventually converge
- **Observability:** We need to track the status of each update
- **Simplicity:** Solution should be maintainable in a homelab environment
- **Failure Recovery:** System should recover automatically from failures

## Considered Options

### Option 1: Direct MQTT Publish (No Transactional Outbox)

**Flow:**
```
API → BEGIN TX
  → Update settings in DB
  → COMMIT TX
  → Publish to MQTT
```

**Pros:**
- Simple implementation
- Low latency
- No additional infrastructure

**Cons:**
- MQTT publish happens outside transaction (lost updates if publish fails)
- No retry mechanism if MQTT is down
- No audit trail of what was sent
- Can't track status of pending updates

**Verdict:** ❌ Too risky for production use

---

### Option 2: Transactional Outbox with Polling Processor

**Flow:**
```
API → BEGIN TX
  → Update settings in DB
  → Insert command into outbox table
  → COMMIT TX

Outbox Processor (separate service):
  → Poll outbox table every N seconds
  → Publish pending commands to MQTT
  → Mark as published in outbox
  → Listen for confirmation from Kafka
  → Mark as confirmed in outbox
```

**Pros:**
- Atomic write (settings + outbox in same transaction)
- Guaranteed delivery (retry mechanism built-in)
- Audit trail of all commands
- Can track status (pending → published → confirmed)
- Survives MQTT downtime (queued in DB)

**Cons:**
- Additional complexity (outbox processor service)
- Polling introduces latency (but acceptable for settings updates)
- Requires new database table and indexes
- Needs monitoring of outbox processor

**Verdict:** ✅ Recommended for reliability

---

### Option 3: CDC-Based Outbox (Debezium/Change Data Capture)

**Flow:**
```
API → BEGIN TX
  → Update settings in DB
  → Insert into outbox table
  → COMMIT TX

CDC Tool (Debezium):
  → Capture changes from outbox table
  → Publish to Kafka
  → Connector publishes to MQTT
```

**Pros:**
- No polling overhead (real-time CDC)
- Industry-standard pattern
- Scalable to high throughput

**Cons:**
- Significant infrastructure complexity (Debezium, Kafka Connect)
- Overkill for homelab scale (< 10 updates/day)
- Harder to debug and maintain
- Requires additional services running

**Verdict:** ❌ Too complex for current scale

---

## Decision Outcome

**Chosen Option:** Transactional Outbox with Polling Processor (Option 2)

### Rationale

1. **Reliability:** Atomic database write ensures settings and outbox command are saved together
2. **Simplicity:** Polling processor is straightforward Rust service (similar to redpanda-sink)
3. **Appropriate Scale:** Polling every 5-10 seconds is acceptable for settings updates
4. **Failure Recovery:** Unprocessed commands remain in outbox and are retried automatically
5. **Observability:** Outbox table provides audit trail and status tracking

### Implementation Plan

#### Database Schema

```sql
CREATE TABLE outbox (
    id BIGSERIAL PRIMARY KEY,
    aggregate_type VARCHAR(255) NOT NULL,          -- 'heatpump_setting'
    aggregate_id VARCHAR(255) NOT NULL,            -- device_id
    event_type VARCHAR(255) NOT NULL,              -- 'setting_update'
    payload JSONB NOT NULL,                        -- {indoor_target_temp: 21.5, ...}
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending → published → confirmed → failed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    confirmed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3
);

CREATE INDEX idx_outbox_status ON outbox(status) WHERE status IN ('pending', 'published');
CREATE INDEX idx_outbox_created ON outbox(created_at);
```

#### Outbox Processor Service

New Rust service: `heatpump-settings-outbox-processor`

**Responsibilities:**
1. Poll outbox table for `status = 'pending'` every 5 seconds
2. For each pending command:
   - Publish to MQTT topic `heatpump/{device_id}/command`
   - Update status to `'published'`, set `published_at`
3. Consume Kafka topic `homelab-heatpump-telemetry`
4. Match incoming telemetry to published commands
5. Update outbox status to `'confirmed'`, set `confirmed_at`
6. Handle failures:
   - Retry up to 3 times
   - Set status to `'failed'` if max retries exceeded
   - Log error message

#### API Changes

`heatpump-settings-api` PATCH endpoint:

```rust
pub async fn update_setting(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
    Json(patch): Json<SettingPatch>,
) -> Result<(StatusCode, Json<SettingResponse>)> {
    let mut tx = state.pool.begin().await?;

    // 1. Update settings table
    let setting = update_setting_in_tx(&mut tx, &device_id, &patch).await?;

    // 2. Insert outbox command
    insert_outbox_command(&mut tx, &device_id, &patch).await?;

    // 3. Commit both together
    tx.commit().await?;

    Ok((StatusCode::ACCEPTED, Json(SettingResponse { setting })))
}
```

Note: Returns `202 Accepted` instead of `200 OK` because the heatpump hasn't confirmed yet.

#### Frontend Changes

1. Display pending status: "Updating..."
2. Poll GET endpoint or use WebSocket for confirmation
3. Show success when confirmed: "Updated successfully"
4. Show error if failed: "Update failed, please try again"

### Consequences

**Positive:**
- Guaranteed delivery of settings updates to heatpump
- No lost updates due to MQTT failures
- Full audit trail in database
- Can implement retry logic and exponential backoff
- Frontend can show real-time status

**Negative:**
- Additional service to maintain (outbox processor)
- Database schema migration required
- Increased latency (polling interval + MQTT + heatpump processing)
- More complex deployment (additional container)

**Neutral:**
- Database size grows (one row per update), but at homelab scale this is negligible
- Can implement cleanup job to delete old confirmed records after 30 days

## Alternatives Considered and Rejected

- **Direct MQTT publish:** Too risky, no recovery mechanism
- **CDC-based outbox:** Over-engineered for current scale
- **Event sourcing:** Would require rewriting entire settings system
- **Two-phase commit:** MQTT doesn't support XA transactions

## Validation

Success criteria:
1. Settings updates survive MQTT broker restarts
2. Settings updates survive API restarts
3. Duplicate submissions are deduplicated
4. Frontend shows pending → confirmed → success flow
5. Failed updates can be manually retried from database

## References

- [Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)
- [Polling Publisher Pattern](https://microservices.io/patterns/data/polling-publisher.html)
- Inspiration: redpanda-sink service (similar poll-and-write pattern)

## Next Steps

1. Create outbox table migration
2. Implement outbox processor service
3. Update heatpump-settings-api to use outbox
4. Add status polling to frontend
5. Add monitoring and alerts for outbox processor
6. Document operational procedures (manual retry, cleanup, etc.)
