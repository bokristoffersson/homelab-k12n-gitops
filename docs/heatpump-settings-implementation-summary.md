# Heatpump Settings Implementation Summary

## Phase 1: Read-Only Settings Tab ✅ COMPLETE

### What Was Implemented

1. **Layout Component with Tab Navigation**
   - Created `Layout.tsx` component with navbar
   - Added tab navigation: Dashboard | Settings
   - Moved theme toggle and timestamp to layout (shared across all pages)

2. **Settings Page (Read-Only)**
   - Created `Settings.tsx` component displaying all heatpump settings
   - Shows settings in organized sections:
     - Temperature Control (target temp, mode)
     - Heating Curve (curve parameters at different outdoor temperatures)
     - Other Settings (heat stop)
   - Automatic refresh every 30 seconds
   - Manual refresh button

3. **TypeScript Types**
   - Created `types/settings.ts` with:
     - `HeatpumpSetting` interface
     - `SettingPatch` interface for updates
     - `HeatpumpMode` enum for display (Off, Heating, Cooling, Auto)

4. **API Service**
   - Created `services/settings.ts` with:
     - `getAllSettings()` - fetch all device settings
     - `getSettingByDevice(deviceId)` - fetch specific device
     - `updateSetting(deviceId, patch)` - update settings (for Phase 2)

5. **Routing Updates**
   - Updated `App.tsx` to use nested routes
   - `/dashboard` - Dashboard page
   - `/settings` - Settings page
   - `/` - Redirects to dashboard

6. **Dashboard Updates**
   - Removed header (now in Layout)
   - Removed theme toggle (now in Layout)
   - Cleaner component focusing on data display

### Commit

```
commit 1c4d4cd
feat: add Settings tab to heatpump-web (Phase 1 - read-only)
```

---

## Phase 2: Editable Settings with Transactional Outbox ⏳ DESIGN COMPLETE

### Architecture Decision

I've created **ADR-001** documenting the transactional outbox pattern for settings updates.

**Location:** `docs/adr/001-transactional-outbox-for-heatpump-settings.md`

### Why Transactional Outbox?

The naive approach (update DB → publish to MQTT) has critical flaws:
- Settings saved to DB but MQTT publish fails → lost update
- MQTT succeeds but DB rollback → ghost command sent to heatpump
- No way to confirm heatpump actually applied the settings
- No retry mechanism if MQTT broker is down

**Transactional Outbox Pattern solves this:**
```
1. User submits settings update
2. API transaction:
   - Update settings table
   - Insert command into outbox table
   - COMMIT (both succeed or both fail)
3. Outbox processor polls outbox table
4. Publishes commands to MQTT
5. Listens for confirmation from Kafka
6. Updates outbox status (pending → published → confirmed)
```

### Components to Build

#### 1. Database Migration

New table: `outbox`

```sql
CREATE TABLE outbox (
    id BIGSERIAL PRIMARY KEY,
    aggregate_type VARCHAR(255) NOT NULL,    -- 'heatpump_setting'
    aggregate_id VARCHAR(255) NOT NULL,      -- device_id
    event_type VARCHAR(255) NOT NULL,        -- 'setting_update'
    payload JSONB NOT NULL,                  -- {indoor_target_temp: 21.5, ...}
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    confirmed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3
);
```

**Status Flow:** `pending` → `published` → `confirmed` (or `failed`)

#### 2. Outbox Processor Service

New Rust service: `heatpump-settings-outbox-processor`

**Responsibilities:**
- Poll outbox table every 5 seconds for `status = 'pending'`
- Publish commands to MQTT topic `heatpump/{device_id}/command`
- Update status to `'published'`
- Consume Kafka topic `homelab-heatpump-telemetry`
- Match incoming telemetry to published commands
- Update status to `'confirmed'` when heatpump responds
- Retry failed publishes (up to 3 times)

**Similar to:** `redpanda-sink` service (same poll-and-process pattern)

#### 3. API Updates

`heatpump-settings-api` PATCH endpoint changes:

```rust
pub async fn update_setting(...) -> Result<(StatusCode, ...)> {
    let mut tx = state.pool.begin().await?;

    // Update settings table
    let setting = update_setting_in_tx(&mut tx, &device_id, &patch).await?;

    // Insert outbox command
    insert_outbox_command(&mut tx, &device_id, &patch).await?;

    tx.commit().await?;  // Atomic: both succeed or both fail

    // Return 202 Accepted (not 200 OK) - pending confirmation
    Ok((StatusCode::ACCEPTED, Json(SettingResponse { setting })))
}
```

New endpoint for status polling:

```rust
GET /api/v1/heatpump/settings/{device_id}/outbox/{id}
→ { status: "confirmed", confirmed_at: "2026-01-11T..." }
```

#### 4. Frontend Updates

**Edit Form:**
- Add form inputs for each setting field
- Validation (temp 15-30°C, mode 0-3, etc.)
- Submit button triggers PATCH request

**Status Display:**
```
Submitting → 202 Accepted → Poll status endpoint
  → "pending" → Show spinner: "Sending to heatpump..."
  → "published" → Show spinner: "Waiting for confirmation..."
  → "confirmed" → Show success: "Settings updated!"
  → "failed" → Show error: "Update failed. Retry?"
```

**Implementation Options:**
1. **Polling:** Call status endpoint every 2 seconds until confirmed
2. **WebSocket:** Real-time push when status changes (requires WebSocket server)
3. **Server-Sent Events (SSE):** HTTP streaming (simpler than WebSocket)

**Recommendation:** Start with polling (simplest), add SSE later if needed.

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (heatpump-web)                 │
│  Settings page → Edit form → PATCH /api/v1/.../settings/{id}  │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                   heatpump-settings-api                         │
│  BEGIN TX                                                       │
│    → UPDATE settings SET ...                                   │
│    → INSERT INTO outbox VALUES (...)                           │
│  COMMIT                                                         │
│  → Return 202 Accepted                                         │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
                          ┌─────────────┐
                          │  PostgreSQL │
                          │   settings  │
                          │   outbox    │
                          └──────┬──────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│             heatpump-settings-outbox-processor                  │
│  Every 5s:                                                      │
│    → SELECT * FROM outbox WHERE status = 'pending'            │
│    → MQTT publish: heatpump/{device_id}/command               │
│    → UPDATE outbox SET status = 'published'                   │
│                                                                 │
│  Kafka consumer (homelab-heatpump-telemetry):                 │
│    → Match telemetry to outbox commands                       │
│    → UPDATE outbox SET status = 'confirmed'                   │
└─────────────────────────────────────────────────────────────────┘
                    │                           ▲
                    ▼                           │
              ┌──────────┐              ┌──────────────┐
              │   MQTT   │              │   Redpanda   │
              │ Mosquitto│              │homelab-heat..│
              └────┬─────┘              └──────▲───────┘
                   │                           │
                   ▼                           │
              ┌──────────────────────────────────┐
              │       Physical Heatpump          │
              │  Receives command → Updates      │
              │  settings → Publishes telemetry  │
              └──────────────────────────────────┘
```

### Failure Scenarios Handled

1. **MQTT Broker Down**
   - Commands stay in outbox as `'pending'`
   - Outbox processor retries when broker comes back
   - No lost updates

2. **Heatpump Offline**
   - Command published to MQTT but no confirmation
   - Stays in `'published'` state
   - Can implement timeout → `'failed'` after N minutes
   - Frontend shows "Waiting for heatpump..."

3. **Database Crash During Transaction**
   - Transaction rolls back
   - Nothing written to settings or outbox
   - API returns error to frontend
   - User can retry

4. **Outbox Processor Crash**
   - Unprocessed commands remain in outbox
   - When processor restarts, picks up where it left off
   - No lost updates

5. **Duplicate Submissions**
   - Frontend can debounce/disable submit button
   - Outbox processor can deduplicate based on timestamp
   - Heatpump should be idempotent

### Observability

**Metrics to Track:**
- Outbox depth (number of pending commands)
- Average time from pending → confirmed
- Failed command rate
- Retry count distribution

**Grafana Dashboard:**
- Outbox status distribution (pending, published, confirmed, failed)
- Latency histogram (submission → confirmation)
- Error rate over time

**Alerts:**
- Outbox depth > 10 for > 5 minutes (processor stuck?)
- Failed commands > 5 in last hour (investigate)

### Migration Path

**Step 1:** Database migration (add outbox table)
**Step 2:** Implement outbox processor service
**Step 3:** Update API to use outbox (keep PATCH working)
**Step 4:** Deploy and test with API only (no frontend changes yet)
**Step 5:** Add edit form to frontend
**Step 6:** Add status polling to frontend
**Step 7:** Monitor and tune (polling interval, retry logic, etc.)

---

## Questions for Review

Before I implement Phase 2, please confirm:

1. **Scope:** Are all settings editable, or just a subset (e.g., only indoor_target_temp and mode)?

2. **Confirmation Flow:** Should the frontend wait for full confirmation from heatpump before showing success, or just show "Sent to heatpump" immediately?

3. **Timeout:** How long should we wait for heatpump confirmation before marking as failed? (e.g., 60 seconds, 5 minutes?)

4. **Retry Logic:** Should failed commands be manually retried by user, or automatically retried with exponential backoff?

5. **Deployment:** Should the outbox processor run in the same namespace as heatpump-settings-api, or separate?

6. **Database:** The outbox table will be in the same database as settings. Is this OK, or should it be a separate database?

---

## Next Steps

Once you approve the design, I'll implement:

1. ✅ Database migration SQL
2. ✅ Outbox processor service (Rust)
3. ✅ API changes (transaction + outbox insert)
4. ✅ Frontend edit form
5. ✅ Frontend status polling
6. ✅ Deployment manifests (outbox processor)
7. ✅ Documentation and runbook

**Estimated Time:** Phase 2 implementation will take significantly longer than Phase 1 due to the additional service and complexity.
