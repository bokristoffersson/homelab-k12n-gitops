# Rust DTO Usage Example

This document shows how to use generated DTOs in your Rust services.

## Setup

### 1. Add Dependency

In your service's `Cargo.toml`:

```toml
[dependencies]
homelab-contracts = { path = "../../generated/rust-dtos/homelab-contracts" }
serde_json = "1.0"
```

### 2. Use in Code

#### Example: Parsing MQTT Messages

```rust
// applications/mqtt-to-timescale/src/mapping.rs
use homelab_contracts::TelemetryRow;
use serde_json;

pub fn parse_telemetry_message(payload: &[u8]) -> Result<TelemetryRow, Box<dyn std::error::Error>> {
    let row: TelemetryRow = serde_json::from_slice(payload)?;
    Ok(row)
}
```

#### Example: Validating Before Processing

```rust
// applications/mqtt-to-timescale/src/ingest.rs
use homelab_contracts::TelemetryRow;

pub async fn process_message(topic: &str, payload: &[u8]) -> Result<(), AppError> {
    // Parse using generated DTO
    let telemetry: TelemetryRow = serde_json::from_slice(payload)
        .map_err(|e| AppError::Parse(format!("Invalid telemetry format: {}", e)))?;
    
    // Validate required fields
    if telemetry.ts.is_none() {
        return Err(AppError::Validation("timestamp is required".into()));
    }
    
    // Process...
    store_telemetry(telemetry).await?;
    
    Ok(())
}
```

#### Example: Type-Safe Database Operations

```rust
// applications/mqtt-to-timescale/src/db.rs
use homelab_contracts::TelemetryRow;
use sqlx::PgPool;

pub async fn insert_telemetry(
    pool: &PgPool,
    row: &TelemetryRow,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO telemetry (
            ts, device_id, room, sensor, location,
            flow_temp_c, return_temp_c, power_w,
            temperature_c, humidity_pct
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        row.ts,
        row.device_id,
        row.room,
        row.sensor,
        row.location,
        row.flow_temp_c,
        row.return_temp_c,
        row.power_w,
        row.temperature_c,
        row.humidity_pct,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
```

## Updating mqtt-to-timescale Service

Here's how to refactor your existing service to use generated DTOs:

### Before (Manual Structs)

```rust
// src/mapping.rs
#[derive(Debug, Clone)]
pub struct Row {
    pub ts: chrono::DateTime<Utc>,
    pub tags: BTreeMap<String, String>,
    pub fields: BTreeMap<String, FieldValue>
}
```

### After (Generated DTOs)

```rust
// src/mapping.rs
use homelab_contracts::TelemetryRow;

// Use TelemetryRow directly, or convert from it
pub fn extract_row(p: &Pipeline, topic: &str, payload: &[u8]) -> Result<Row, AppError> {
    // Parse using generated DTO
    let telemetry: TelemetryRow = serde_json::from_slice(payload)?;
    
    // Convert to internal Row format if needed
    let row = Row {
        ts: telemetry.ts,
        tags: extract_tags(&telemetry),
        fields: extract_fields(&telemetry),
    };
    
    Ok(row)
}
```

## Benefits

1. **Type Safety**: Compile-time checking of data structures
2. **Consistency**: Same types across all services
3. **Documentation**: Types serve as documentation
4. **Refactoring**: Changes propagate automatically
5. **Validation**: Can add validation logic to generated types

## Testing with Generated Types

```rust
// tests/telemetry_test.rs
use homelab_contracts::TelemetryRow;
use serde_json::json;

#[test]
fn test_parse_telemetry() {
    let payload = json!({
        "ts": "2025-01-15T10:30:00Z",
        "device_id": "hp-01",
        "room": "living-room",
        "flow_temp_c": 45.5,
        "return_temp_c": 40.2,
        "power_w": 950
    });
    
    let telemetry: TelemetryRow = serde_json::from_value(payload).unwrap();
    
    assert_eq!(telemetry.device_id, Some("hp-01".to_string()));
    assert_eq!(telemetry.flow_temp_c, Some(45.5));
}
```

## Migration Path

1. **Generate DTOs** from existing contracts
2. **Add dependency** to service
3. **Replace manual structs** with generated types
4. **Update tests** to use new types
5. **Remove old struct definitions**

## Troubleshooting

### "Cannot find crate `homelab_contracts`"

Make sure you've:
1. Generated the DTOs: `./scripts/generate-rust-dtos.sh`
2. Built the crate: `cd generated/rust-dtos/homelab-contracts && cargo build`
3. Added the dependency correctly in `Cargo.toml`

### "Type mismatch" errors

The generated types might have different field names or types. Check:
1. The JSON Schema contract
2. Regenerate DTOs: `./scripts/generate-rust-dtos.sh`
3. Update your code to match generated types

### "Serde deserialization errors"

Ensure your JSON payload matches the schema. Use `serde_json::from_slice` with proper error handling.

