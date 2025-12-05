# Common Business Processes for BPM Testing

This document provides detailed process definitions for common business scenarios you can use to test and validate your BPM system.

---

## 1. Temperature Control Process 🌡️

**Use Case**: Automatically manage temperature when it exceeds thresholds

**Trigger**: 
- Event: `heatpump-telemetry` topic
- Condition: `flow_temp_c > 25.0`

**Process Flow**:
```
[Start] 
  ↓
[Read Temperature]
  ↓
{Temperature > 25°C?}
  ├─ YES → [Activate Cooling] → [Wait 5 minutes] → [Re-check Temperature]
  │         ↓
  │         {Temperature Normal?}
  │         ├─ YES → [Log Success] → [End]
  │         └─ NO → [Escalate Alert] → [Notify User] → [End]
  │
  └─ NO → [Log Normal] → [End]
```

**Test Cases**:
- ✅ Normal operation (temp = 22°C, no action)
- ✅ Threshold exceeded (temp = 26°C, cooling activated)
- ✅ Cooling successful (temp drops to 23°C)
- ✅ Cooling fails (temp stays at 27°C, escalation)
- ✅ Concurrent instances (multiple rooms)

**Expected Outcomes**:
- Cooling command published to MQTT
- Process state persisted
- Alerts sent when needed
- Audit trail in database

---

## 2. Device Onboarding Process 📱

**Use Case**: Automatically onboard new IoT devices

**Trigger**:
- Event: `devices/register` topic
- Or: Manual API call

**Process Flow**:
```
[Start]
  ↓
[Validate Device Info]
  ↓
{Valid?}
  ├─ YES → [Check Compatibility] → {Compatible?}
  │         ├─ YES → [Provision Device] → [Configure Settings]
  │         │         → [Activate Monitoring] → [Send Welcome] → [End]
  │         └─ NO → [Log Incompatible] → [Notify Admin] → [End]
  │
  └─ NO → [Log Validation Error] → [Notify Source] → [End]
```

**Test Cases**:
- ✅ Valid compatible device (successful onboarding)
- ✅ Valid but incompatible device (rejection with notification)
- ✅ Invalid device data (validation error)
- ✅ Network failure during provisioning (retry logic)
- ✅ Duplicate registration (idempotency)

**Expected Outcomes**:
- Device added to database
- Monitoring configured
- Welcome notification sent
- Error handling for failures

---

## 3. Maintenance Alert Process 🔧

**Use Case**: Handle device errors and alarms with appropriate escalation

**Trigger**:
- Event: `device-alerts` topic
- Condition: `alarm_active = true`

**Process Flow**:
```
[Start]
  ↓
[Categorize Alert]
  ↓
{Severity?}
  ├─ CRITICAL → [Immediate Notification] → [Create Ticket]
  │              → [Assign Technician] → [Wait for Resolution]
  │              → [Verify Fix] → [Close Ticket] → [End]
  │
  ├─ WARNING → [Log Alert] → [Schedule Check in 1 hour]
  │           → {Resolved?}
  │           ├─ YES → [Close] → [End]
  │           └─ NO → [Escalate to Critical] → (continue critical flow)
  │
  └─ INFO → [Log Only] → [End]
```

**Test Cases**:
- ✅ Critical alert (immediate action, ticket created)
- ✅ Warning that auto-resolves (no escalation)
- ✅ Warning that escalates (becomes critical)
- ✅ Multiple alerts for same device (deduplication)
- ✅ Technician resolution (verification step)

**Expected Outcomes**:
- Appropriate notifications sent
- Tickets created in tracking system
- Escalation when needed
- Resolution verification

---

## 4. Energy Optimization Process ⚡

**Use Case**: Optimize energy consumption when usage is high

**Trigger**:
- Event: `energy-realtime` topic
- Condition: `consumption_total_w > 1000`

**Process Flow**:
```
[Start]
  ↓
[Analyze Consumption]
  ↓
[Identify High Consumers]
  ↓
[Optimize Settings]
  ↓
[Wait 15 minutes]
  ↓
[Re-analyze Consumption]
  ↓
{Improved?}
  ├─ YES → [Log Optimization] → [End]
  └─ NO → [Further Optimization] → [Notify User] → [End]
```

**Test Cases**:
- ✅ Successful optimization (consumption drops)
- ✅ Optimization fails (requires manual intervention)
- ✅ Multiple optimization cycles needed
- ✅ Optimization conflicts (multiple devices)

**Expected Outcomes**:
- Settings adjusted via MQTT/API
- Consumption monitored
- User notified if manual intervention needed
- Optimization history logged

---

## 5. Data Quality Check Process 📊

**Use Case**: Validate and clean incoming data before storage

**Trigger**:
- Event: Any data ingestion topic
- Or: Pre-storage hook

**Process Flow**:
```
[Start]
  ↓
[Validate Data Format]
  ↓
[Check Completeness]
  ↓
{Valid?}
  ├─ YES → [Transform Data] → [Store] → [Publish Success] → [End]
  │
  └─ NO → [Log Error] → [Attempt Correction]
          → {Corrected?}
            ├─ YES → [Store] → [Publish Warning] → [End]
            └─ NO → [Reject] → [Notify Source] → [End]
```

**Test Cases**:
- ✅ Valid data (normal flow)
- ✅ Invalid format (correction attempt)
- ✅ Missing required fields (rejection)
- ✅ High error rate (alert admin)
- ✅ Partial data (correction with warnings)

**Expected Outcomes**:
- Data validated before storage
- Errors logged and tracked
- Source notified of issues
- Quality metrics recorded

---

## 6. Approval Workflow ✅

**Use Case**: Require approval for critical actions

**Trigger**:
- Manual API call
- Or: Automated threshold (e.g., power > 2000W)

**Process Flow**:
```
[Start]
  ↓
[Create Request]
  ↓
[Assign Approver]
  ↓
[Wait for Approval] (with timeout)
  ↓
{Approved?}
  ├─ YES → [Execute Action] → [Notify Requester] → [End]
  ├─ NO → [Notify Requester] → [Log Rejection] → [End]
  └─ TIMEOUT → [Escalate] → [Notify Admin] → [End]
```

**Test Cases**:
- ✅ Approval granted (action executed)
- ✅ Approval denied (action blocked)
- ✅ Approval timeout (escalation)
- ✅ Multiple approvers (parallel approval)
- ✅ Approval delegation

**Expected Outcomes**:
- Request created and tracked
- Approver notified
- Action executed or blocked based on decision
- Audit trail maintained

---

## 7. Scheduled Maintenance Process 📅

**Use Case**: Periodic health checks and maintenance

**Trigger**:
- Timer: Daily at 2 AM
- Or: Weekly on Sunday

**Process Flow**:
```
[Start]
  ↓
[Check Device Status]
  ↓
[Generate Health Report]
  ↓
{Issues Found?}
  ├─ YES → [Create Maintenance Tasks] → [Schedule] → [Notify] → [End]
  └─ NO → [Log Healthy Status] → [End]
```

**Test Cases**:
- ✅ Routine check (no issues found)
- ✅ Issues detected (maintenance scheduled)
- ✅ Maintenance completion (verification)
- ✅ Multiple devices (batch processing)

**Expected Outcomes**:
- Health report generated
- Maintenance tasks created
- Notifications sent
- Status logged

---

## 8. Multi-Stage Data Processing Pipeline 🔄

**Use Case**: Complex data transformation with multiple stages

**Trigger**:
- Event: Raw data ingestion

**Process Flow**:
```
[Start]
  ↓
[Stage 1: Validate] → {Valid?} → [Stage 2: Transform]
  ↓                                    ↓
[Stage 3: Enrich] → [Stage 4: Aggregate] → [Stage 5: Store]
  ↓
[Publish Processed Event]
  ↓
[End]
```

**Test Cases**:
- ✅ Successful multi-stage processing
- ✅ Failure at intermediate stage (rollback)
- ✅ Parallel processing of multiple records
- ✅ Stage timeout handling

**Expected Outcomes**:
- Data processed through all stages
- Intermediate results stored
- Final output published
- Error recovery if stage fails

---

## 9. Incident Response Process 🚨

**Use Case**: Automated incident detection and response

**Trigger**:
- Event: Multiple error conditions detected
- Condition: Error rate > threshold

**Process Flow**:
```
[Start]
  ↓
[Detect Incident]
  ↓
[Assess Impact]
  ↓
{Severity?}
  ├─ HIGH → [Immediate Response] → [Notify Team] → [Create Incident]
  │         → [Execute Mitigation] → [Monitor] → {Resolved?}
  │         ├─ YES → [Close Incident] → [Post-Mortem] → [End]
  │         └─ NO → [Escalate] → [Wait] → (re-assess)
  │
  └─ LOW → [Log Incident] → [Monitor] → [Auto-Resolve] → [End]
```

**Test Cases**:
- ✅ High severity incident (immediate response)
- ✅ Low severity (auto-resolution)
- ✅ Incident escalation
- ✅ Multiple concurrent incidents

**Expected Outcomes**:
- Incident detected and categorized
- Appropriate response executed
- Team notified
- Resolution tracked

---

## 10. Device Firmware Update Process 🔄

**Use Case**: Coordinate firmware updates across devices

**Trigger**:
- Manual API call
- Or: Scheduled update window

**Process Flow**:
```
[Start]
  ↓
[Check Device Compatibility]
  ↓
[Create Update Plan]
  ↓
[Notify Users] (if required)
  ↓
[Execute Update] (staged rollout)
  ↓
[Verify Update]
  ↓
{Success?}
  ├─ YES → [Activate New Firmware] → [Log Success] → [End]
  └─ NO → [Rollback] → [Notify Admin] → [End]
```

**Test Cases**:
- ✅ Successful update
- ✅ Update failure (rollback)
- ✅ Staged rollout (batches)
- ✅ User notification required

**Expected Outcomes**:
- Update executed safely
- Rollback on failure
- Status tracked
- Users notified

---

## Process Complexity Matrix

| Process | Complexity | Good For Testing |
|---------|-----------|------------------|
| Temperature Control | ⭐ Low | First implementation |
| Data Quality Check | ⭐ Low | Validation logic |
| Approval Workflow | ⭐⭐ Medium | Human tasks |
| Device Onboarding | ⭐⭐ Medium | Multi-step process |
| Maintenance Alert | ⭐⭐⭐ High | Escalation logic |
| Energy Optimization | ⭐⭐⭐ High | Decision loops |
| Scheduled Maintenance | ⭐⭐ Medium | Timer handling |
| Multi-Stage Pipeline | ⭐⭐⭐ High | Complex orchestration |
| Incident Response | ⭐⭐⭐⭐ Very High | Advanced scenarios |
| Firmware Update | ⭐⭐⭐ High | Rollback logic |

---

## Recommended Testing Order

1. **Start Simple**: Temperature Control Process
   - Single decision point
   - Clear trigger condition
   - Easy to test with your existing data

2. **Add Complexity**: Device Onboarding
   - Multiple steps
   - Error handling
   - External integrations

3. **Test Decisions**: Energy Optimization
   - Multiple decision points
   - Loops and retries
   - State management

4. **Human Interaction**: Approval Workflow
   - User tasks
   - Timeouts
   - Notifications

5. **Advanced**: Maintenance Alert
   - Escalation
   - Multiple paths
   - Complex state

---

## Integration with Your Existing System

All these processes can be triggered by:
- **Redpanda events**: Subscribe to your existing topics
- **TimescaleDB queries**: Query for conditions (e.g., "find devices with temp > 25°C")
- **MQTT messages**: Direct MQTT triggers
- **API calls**: Manual process start
- **Timers**: Scheduled processes

**Example Integration**:
```yaml
# Temperature Control triggered by Redpanda
trigger:
  type: redpanda
  topic: heatpump-telemetry
  condition: fields.flow_temp_c > 25.0

# Maintenance Alert triggered by database query
trigger:
  type: database
  query: |
    SELECT device_id FROM telemetry 
    WHERE alarm_active = true 
    AND ts > NOW() - INTERVAL '5 minutes'
  schedule: every 1 minute
```

---

*Use these processes as templates and adapt them to your specific needs!*
