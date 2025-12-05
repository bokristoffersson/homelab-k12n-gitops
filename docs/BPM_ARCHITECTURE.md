# BPM (Business Process Management) Architecture Plan

## Current Architecture

Your current data pipeline:
```
MQTT → mqtt-input → Redpanda → redpanda-sink → TimescaleDB
```

**Components you have:**
- ✅ **Data Ingestion**: `mqtt-input` - MQTT to Redpanda
- ✅ **Message Transport**: Redpanda (Kafka-compatible)
- ✅ **Data Persistence**: `redpanda-sink` - Redpanda to TimescaleDB
- ✅ **Time-series Storage**: TimescaleDB hypertables
- ✅ **Static Data Storage**: PostgreSQL tables with upsert

---

## Missing Components for BPM

### 1. **Process Engine / Orchestrator** ⭐ Core Component
**Purpose**: Execute business process definitions, manage process instances, handle state transitions

**Key Features Needed:**
- Process definition parser (BPMN, YAML, or custom DSL)
- Process instance lifecycle management
- State persistence (in TimescaleDB or separate process DB)
- Event-driven process triggering
- Parallel and sequential task execution
- Error handling and retry logic

**Technology Options:**
- **Custom Rust service** (matches your stack)
- **Temporal** (workflow orchestration platform)
- **Zeebe** (BPMN engine)
- **Conductor** (Netflix's workflow engine)

**Recommended**: Start with a custom Rust service that consumes from Redpanda and manages process state in TimescaleDB.

---

### 2. **Event Router / Trigger Service**
**Purpose**: Route events from your data pipeline to trigger business processes

**Key Features:**
- Subscribe to Redpanda topics (or query TimescaleDB)
- Match events to process definitions (e.g., "when temperature > 25°C, start cooling process")
- Trigger process instances
- Support for complex event patterns (CEP)

**Integration Points:**
- Consumes from: Redpanda topics or TimescaleDB queries
- Publishes to: Process engine (via Redpanda or direct API)

---

### 3. **Decision Engine / Rules Engine**
**Purpose**: Evaluate business rules and make routing decisions in processes

**Key Features:**
- Rule definition language (YAML/JSON or DSL)
- Conditional logic evaluation
- Data transformation rules
- Integration with process engine for gateways/decisions

**Example Use Cases:**
- "If temperature > threshold, route to alert process"
- "If device status = 'error', escalate to maintenance"
- "If power consumption > 1000W, trigger optimization process"

---

### 4. **Task Management Service**
**Purpose**: Handle human tasks, service tasks, and external integrations

**Key Features:**
- Human task assignment and completion
- Service task execution (HTTP calls, database operations)
- Task queues and prioritization
- Task notifications (email, webhooks, MQTT)

**Task Types:**
- **User Tasks**: Require human interaction
- **Service Tasks**: Automated service calls
- **Script Tasks**: Execute scripts/code
- **Timer Tasks**: Scheduled/delayed execution

---

### 5. **Process Definition Repository**
**Purpose**: Store and version control process definitions

**Key Features:**
- Process definition storage (database or Git)
- Versioning and deployment
- Validation and testing
- Process templates

**Storage Options:**
- PostgreSQL table for process definitions
- Git repository (GitOps approach - matches your setup!)
- Separate service with API

---

### 6. **API Gateway / Process API**
**Purpose**: REST/GraphQL API for process management and interaction

**Key Features:**
- Start/stop process instances
- Query process status and history
- Complete user tasks
- Process definition management
- Metrics and monitoring endpoints

**Endpoints Needed:**
- `POST /processes/{id}/start` - Start process instance
- `GET /processes/{id}` - Get process status
- `GET /processes/{id}/history` - Get execution history
- `POST /tasks/{id}/complete` - Complete user task
- `GET /definitions` - List process definitions

---

### 7. **Process State Store**
**Purpose**: Persist process instance state, variables, and execution history

**Key Features:**
- Process instance metadata
- Variable storage (JSONB in PostgreSQL)
- Execution history/audit log
- Process instance queries

**Database Schema:**
```sql
-- Process definitions
CREATE TABLE process_definitions (
    id TEXT PRIMARY KEY,
    version INTEGER,
    definition JSONB,  -- BPMN/YAML process definition
    created_at TIMESTAMPTZ
);

-- Process instances
CREATE TABLE process_instances (
    id UUID PRIMARY KEY,
    definition_id TEXT,
    status TEXT,  -- 'running', 'completed', 'failed', 'suspended'
    variables JSONB,  -- Process variables
    current_activity TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Execution history (audit log)
CREATE TABLE process_history (
    id UUID PRIMARY KEY,
    instance_id UUID REFERENCES process_instances(id),
    activity_id TEXT,
    event_type TEXT,  -- 'started', 'completed', 'failed'
    timestamp TIMESTAMPTZ,
    data JSONB
);
```

---

### 8. **Timer / Scheduler Service**
**Purpose**: Handle time-based events (delays, scheduled tasks, timeouts)

**Key Features:**
- Timer creation and management
- Scheduled task execution
- Process timeout handling
- Recurring task support

**Integration:**
- Can use PostgreSQL `pg_cron` or separate scheduler
- Publishes timer events to Redpanda or directly triggers processes

---

### 9. **Monitoring & Observability**
**Purpose**: Track process execution, performance, and errors

**Key Features:**
- Process metrics (execution time, success rate, throughput)
- Error tracking and alerting
- Process visualization
- Integration with existing monitoring (Grafana)

**Metrics to Track:**
- Process instance count (running/completed/failed)
- Average execution time per process type
- Task completion rates
- Error rates by process/activity

---

### 10. **UI / Dashboard** (Optional but Recommended)
**Purpose**: Visual process management and monitoring

**Key Features:**
- Process definition editor (visual or code-based)
- Process instance monitoring
- Task management interface
- Process execution history viewer
- Real-time process status

**Technology Options:**
- **Backstage** (you already have this!) - Add BPM plugins
- **Custom React/Vue dashboard**
- **Grafana dashboards** for metrics
- **Process visualization** (BPMN.js, React Flow)

---

## Recommended Architecture

```
┌─────────────────┐
│   MQTT Devices  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   mqtt-input    │ (existing)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Redpanda     │ (existing)
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌─────────┐ ┌──────────────────┐
│redpanda │ │  Event Router    │ (NEW)
│  -sink  │ │  / Trigger       │
└────┬────┘ └────────┬─────────┘
     │               │
     ▼               ▼
┌─────────┐   ┌──────────────────┐
│Timescale│   │ Process Engine   │ (NEW)
│   DB    │   │ / Orchestrator   │
└────┬────┘   └────────┬─────────┘
     │                 │
     └────────┬───────┘
              │
              ▼
     ┌──────────────────┐
     │ Process State DB │ (NEW - can be same TimescaleDB)
     └──────────────────┘
              │
    ┌─────────┴─────────┐
    │                   │
    ▼                   ▼
┌─────────┐      ┌──────────────┐
│Task Mgmt│      │ Process API  │ (NEW)
│ Service │      │   Gateway    │
└─────────┘      └──────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │   UI/Dash    │ (NEW - or use Backstage)
                 └──────────────┘
```

---

## Common Business Processes for Testing

### 1. **Temperature Control Process** 🌡️
**Trigger**: Temperature reading exceeds threshold
**Process Flow**:
```
Start → Check Temperature → [Decision: > threshold?]
  ├─ Yes → Activate Cooling → Wait 5 min → Re-check → [Decision: Normal?]
  │   ├─ Yes → Log Success → End
  │   └─ No → Escalate Alert → Notify User → End
  └─ No → Log Normal → End
```

**Test Scenarios:**
- Normal operation (temperature within range)
- Threshold exceeded (trigger cooling)
- Cooling fails (escalation)
- Multiple concurrent instances (different rooms)

---

### 2. **Device Onboarding Process** 📱
**Trigger**: New device registration event
**Process Flow**:
```
Start → Validate Device Info → Check Compatibility
  ├─ Compatible → Provision Device → Configure Settings
  │   → Activate Monitoring → Send Welcome Notification → End
  └─ Incompatible → Log Error → Notify Admin → End
```

**Test Scenarios:**
- Successful onboarding
- Invalid device data
- Network failure during provisioning
- Duplicate device registration

---

### 3. **Maintenance Alert Process** 🔧
**Trigger**: Device error/alarm event
**Process Flow**:
```
Start → Categorize Alert → [Decision: Severity]
  ├─ Critical → Immediate Notification → Create Ticket
  │   → Assign Technician → Wait for Resolution → Verify Fix → Close → End
  ├─ Warning → Log Alert → Schedule Check → [Decision: Resolved?]
  │   ├─ Yes → Close → End
  │   └─ No → Escalate to Critical → (continue critical flow)
  └─ Info → Log Only → End
```

**Test Scenarios:**
- Critical alert (immediate action)
- Warning that auto-resolves
- Warning that escalates
- Multiple alerts for same device

---

### 4. **Energy Optimization Process** ⚡
**Trigger**: High energy consumption detected
**Process Flow**:
```
Start → Analyze Consumption → Identify High Consumers
  → Optimize Settings → Wait 15 min → Re-analyze
  → [Decision: Improved?]
    ├─ Yes → Log Optimization → End
    └─ No → Further Optimization → Notify User → End
```

**Test Scenarios:**
- Successful optimization
- Optimization fails (requires manual intervention)
- Multiple optimization cycles

---

### 5. **Data Quality Check Process** 📊
**Trigger**: New data received
**Process Flow**:
```
Start → Validate Data Format → Check Completeness
  → [Decision: Valid?]
    ├─ Yes → Transform Data → Store → Publish Success Event → End
    └─ No → Log Error → Attempt Correction → [Decision: Corrected?]
        ├─ Yes → Store → Publish Warning → End
        └─ No → Reject → Notify Source → End
```

**Test Scenarios:**
- Valid data (normal flow)
- Invalid format (correction attempt)
- Missing required fields (rejection)
- High error rate (alert admin)

---

### 6. **Approval Workflow** ✅
**Trigger**: Manual request or automated threshold
**Process Flow**:
```
Start → Create Request → Assign Approver → Wait for Approval
  → [Decision: Approved?]
    ├─ Yes → Execute Action → Notify Requester → End
    └─ No → Notify Requester → Log Rejection → End
```

**Test Scenarios:**
- Approval granted
- Approval denied
- Approval timeout (escalation)
- Multiple approvers (parallel)

---

### 7. **Scheduled Maintenance Process** 📅
**Trigger**: Timer/scheduler (daily/weekly)
**Process Flow**:
```
Start → Check Device Status → Generate Report
  → [Decision: Issues Found?]
    ├─ Yes → Create Maintenance Tasks → Schedule → Notify → End
    └─ No → Log Healthy Status → End
```

**Test Scenarios:**
- Routine check (no issues)
- Issues detected (maintenance scheduled)
- Maintenance completion verification

---

## Implementation Priority

### Phase 1: Core Foundation (MVP)
1. ✅ **Process Engine** - Basic process execution
2. ✅ **Process State Store** - Database schema and persistence
3. ✅ **Event Router** - Simple event-to-process triggering
4. ✅ **Process API** - Basic REST API

**Goal**: Run a simple process triggered by an event

---

### Phase 2: Enhanced Features
5. ✅ **Decision Engine** - Conditional routing
6. ✅ **Timer Service** - Delays and scheduled tasks
7. ✅ **Task Management** - User and service tasks
8. ✅ **Process Definition Repository** - Store and version definitions

**Goal**: Support complex processes with decisions and tasks

---

### Phase 3: Production Ready
9. ✅ **Monitoring** - Metrics and observability
10. ✅ **UI/Dashboard** - Visual management
11. ✅ **Error Handling** - Retry, compensation, error recovery
12. ✅ **Performance Optimization** - Caching, batching, scaling

**Goal**: Production-ready BPM system

---

## Technology Stack Recommendations

**Keep Consistent with Your Stack:**
- **Language**: Rust (matches your existing services)
- **Database**: TimescaleDB/PostgreSQL (you already have this)
- **Message Queue**: Redpanda (you already have this)
- **API Framework**: Axum or Actix-web (Rust)
- **Process Definition**: YAML (simple, GitOps-friendly) or BPMN (standard)
- **UI**: React + TypeScript (or extend Backstage)

**Why Rust:**
- Matches your existing services (`mqtt-input`, `redpanda-sink`)
- High performance for process orchestration
- Strong type safety for process definitions
- Good async support (Tokio)

---

## Next Steps

1. **Start Small**: Implement a simple process engine that can:
   - Parse YAML process definitions
   - Execute sequential tasks
   - Persist state to TimescaleDB
   - Trigger from Redpanda events

2. **Choose First Process**: Pick one of the common processes above (recommend **Temperature Control** - simple and relevant)

3. **Design Process Definition Format**: YAML-based, simple and extensible

4. **Build Incrementally**: Add features as you need them

5. **Test with Real Data**: Use your existing MQTT/Redpanda pipeline

---

## Example Process Definition (YAML)

```yaml
id: temperature-control
name: Temperature Control Process
version: 1.0

trigger:
  type: event
  source: redpanda
  topic: heatpump-telemetry
  condition: |
    fields.flow_temp_c > 25.0

variables:
  - name: device_id
    source: tags.device_id
  - name: temperature
    source: fields.flow_temp_c
  - name: threshold
    value: 25.0

activities:
  - id: check_temp
    type: decision
    condition: temperature > threshold
    true_path: activate_cooling
    false_path: log_normal

  - id: activate_cooling
    type: service
    action: mqtt_publish
    topic: home/heatpump/control
    payload:
      device_id: ${device_id}
      command: activate_cooling

  - id: wait_recheck
    type: timer
    duration: 5m

  - id: recheck_temp
    type: decision
    condition: temperature <= threshold
    true_path: log_success
    false_path: escalate

  - id: log_normal
    type: service
    action: database_log
    message: "Temperature normal: ${temperature}°C"

  - id: log_success
    type: service
    action: database_log
    message: "Cooling successful"

  - id: escalate
    type: service
    action: notify
    channel: alert
    message: "Temperature still high after cooling"
```

---

## References & Inspiration

- **Temporal**: https://temporal.io/ - Workflow orchestration
- **Zeebe**: https://zeebe.io/ - BPMN engine
- **Conductor**: https://conductor.netflix.com/ - Netflix workflow engine
- **BPMN 2.0**: Standard for process modeling
- **Camunda**: Commercial BPM platform (good for reference)

---

*This document provides a roadmap for building a BPM system on top of your existing data pipeline infrastructure.*
