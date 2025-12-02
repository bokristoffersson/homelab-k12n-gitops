# Contract-First Examples

This document provides concrete examples of contracts for existing services in the homelab.

## Example: mqtt-to-timescale Service

### Directory Structure
```
contracts/
└── mqtt-to-timescale/
    ├── README.md
    ├── data-models/
    │   ├── telemetry-row.schema.json
    │   └── database/
    │       └── telemetry.sql
    └── asyncapi/
        └── mqtt-ingestion.yaml
```

### 1. Service README.md
```markdown
# mqtt-to-timescale Service Contracts

## Overview
Service that ingests MQTT messages and stores them in TimescaleDB.

## Contracts Defined

### AsyncAPI
- **mqtt-ingestion.yaml**: Defines MQTT topics consumed and message schemas

### Data Models
- **telemetry-row.schema.json**: JSON Schema for telemetry data rows
- **database/telemetry.sql**: TimescaleDB schema definition

## MQTT Topics Consumed

- `home/heatpump/telemetry` - Heat pump telemetry data
- `home/sensors/+/state` - Sensor state data (wildcard)

## Message Format

All messages are JSON with the following structure:
- Timestamp (RFC3339, Unix ms/s, or ISO8601)
- Device/sensor identification
- Telemetry fields (temperature, power, etc.)

## Database Schema

See `data-models/database/telemetry.sql` for the TimescaleDB schema.

## Version History

- **v1.0.0** (2025-01-XX): Initial contract definition
```

### 2. JSON Schema for Telemetry Row
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://homelab.example.com/contracts/mqtt-to-timescale/telemetry-row",
  "title": "Telemetry Row",
  "description": "Schema for telemetry data stored in TimescaleDB",
  "type": "object",
  "properties": {
    "ts": {
      "type": "string",
      "format": "date-time",
      "description": "Timestamp in UTC"
    },
    "device_id": {
      "type": "string",
      "description": "Device identifier"
    },
    "room": {
      "type": "string",
      "description": "Room identifier (optional)"
    },
    "sensor": {
      "type": "string",
      "description": "Sensor identifier (optional)"
    },
    "location": {
      "type": "string",
      "description": "Location identifier (optional)"
    },
    "flow_temp_c": {
      "type": "number",
      "description": "Flow temperature in Celsius"
    },
    "return_temp_c": {
      "type": "number",
      "description": "Return temperature in Celsius"
    },
    "power_w": {
      "type": "integer",
      "description": "Power consumption in Watts"
    },
    "temperature_c": {
      "type": "number",
      "description": "Temperature in Celsius"
    },
    "humidity_pct": {
      "type": "number",
      "description": "Humidity percentage"
    }
  },
  "required": ["ts"]
}
```

### 3. AsyncAPI Specification
```yaml
asyncapi: '3.0.0'
info:
  title: MQTT Ingestion API
  version: '1.0.0'
  description: |
    MQTT topics consumed by mqtt-to-timescale service.
    Messages are ingested and stored in TimescaleDB.

servers:
  mosquitto:
    host: p0.local
    protocol: mqtt
    port: 1883
    description: Mosquitto MQTT broker

channels:
  heatpumpTelemetry:
    address: home/heatpump/telemetry
    description: Heat pump telemetry data
    messages:
      heatpumpMessage:
        $ref: '#/components/messages/HeatpumpTelemetry'
    bindings:
      mqtt:
        qos: 1
        retain: false

  sensorState:
    address: home/sensors/{sensorId}/state
    description: Sensor state data (wildcard topic)
    parameters:
      sensorId:
        $ref: '#/components/parameters/SensorId'
    messages:
      sensorMessage:
        $ref: '#/components/messages/SensorState'
    bindings:
      mqtt:
        qos: 1
        retain: false

components:
  messages:
    HeatpumpTelemetry:
      name: HeatpumpTelemetry
      title: Heat Pump Telemetry Message
      contentType: application/json
      payload:
        $ref: '#/components/schemas/HeatpumpTelemetryPayload'
      examples:
        - payload:
            device_id: "hp-01"
            room: "living-room"
            timestamp: "2025-01-15T10:30:00Z"
            flow_temp: 45.5
            return_temp: 40.2
            power: 950

    SensorState:
      name: SensorState
      title: Sensor State Message
      contentType: application/json
      payload:
        $ref: '#/components/schemas/SensorStatePayload'
      examples:
        - payload:
            name: "kitchen-sensor"
            location: "kitchen"
            timestamp: "2025-01-15T10:30:00Z"
            temperature: 21.5
            humidity: 45.2

  schemas:
    HeatpumpTelemetryPayload:
      type: object
      required:
        - device_id
        - timestamp
      properties:
        device_id:
          type: string
          description: Heat pump device identifier
        room:
          type: string
          description: Room where heat pump is located
        timestamp:
          type: string
          format: date-time
          description: RFC3339 timestamp
        flow_temp:
          type: number
          description: Flow temperature in Celsius
        return_temp:
          type: number
          description: Return temperature in Celsius
        power:
          type: integer
          description: Power consumption in Watts
        status_byte:
          type: integer
          minimum: 0
          maximum: 255
          description: Status byte with bit flags

    SensorStatePayload:
      type: object
      required:
        - name
        - timestamp
      properties:
        name:
          type: string
          description: Sensor name/identifier
        location:
          type: string
          description: Sensor location
        timestamp:
          type: string
          format: date-time
          description: RFC3339 timestamp (or use_now if not provided)
        temperature:
          type: number
          description: Temperature in Celsius
        humidity:
          type: number
          description: Humidity percentage

  parameters:
    SensorId:
      description: Sensor identifier (wildcard in topic)
      schema:
        type: string
```

### 4. Database Schema Contract
```sql
-- contracts/mqtt-to-timescale/data-models/database/telemetry.sql
-- TimescaleDB schema for telemetry data
-- Version: 1.0.0

CREATE TABLE IF NOT EXISTS telemetry
(
    ts          TIMESTAMPTZ       NOT NULL,
    device_id   TEXT,
    room        TEXT,
    sensor      TEXT,
    location    TEXT,
    flow_temp_c DOUBLE PRECISION,
    return_temp_c DOUBLE PRECISION,
    power_w     BIGINT,
    temperature_c DOUBLE PRECISION,
    humidity_pct DOUBLE PRECISION
);

-- Create hypertable for time-series optimization
SELECT create_hypertable('telemetry', 'ts', if_not_exists => TRUE);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_telemetry_device_id ON telemetry(device_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_room ON telemetry(room);
CREATE INDEX IF NOT EXISTS idx_telemetry_sensor ON telemetry(sensor);
```

## Example: heatpump-mqtt Service

### Directory Structure
```
contracts/
└── heatpump-mqtt/
    ├── README.md
    ├── asyncapi/
    │   └── mqtt-publisher.yaml
    └── examples/
        └── sample-messages.json
```

### AsyncAPI for Publisher
```yaml
asyncapi: '3.0.0'
info:
  title: Heat Pump MQTT Publisher API
  version: '1.0.0'
  description: |
    MQTT topics published by heatpump-mqtt service.

servers:
  mosquitto:
    host: p0.local
    protocol: mqtt
    port: 1883

channels:
  telemetry:
    address: home/heatpump/telemetry
    description: Publishes heat pump telemetry data
    messages:
      telemetryMessage:
        $ref: '#/components/messages/HeatpumpTelemetry'
    bindings:
      mqtt:
        qos: 1
        retain: false

components:
  messages:
    HeatpumpTelemetry:
      name: HeatpumpTelemetry
      contentType: application/json
      payload:
        $ref: '#/components/schemas/HeatpumpTelemetryPayload'

  schemas:
    HeatpumpTelemetryPayload:
      type: object
      required:
        - device_id
        - timestamp
      properties:
        device_id:
          type: string
        room:
          type: string
        timestamp:
          type: string
          format: date-time
        flow_temp:
          type: number
        return_temp:
          type: number
        power:
          type: integer
        status_byte:
          type: integer
          minimum: 0
          maximum: 255
```

## Example: Shared Contracts

### Directory Structure
```
contracts/
└── shared/
    ├── common-types/
    │   ├── timestamp.schema.json
    │   └── device-id.schema.json
    └── mqtt-topics/
        └── topic-registry.yaml
```

### Topic Registry
```yaml
# contracts/shared/mqtt-topics/topic-registry.yaml
topics:
  - name: heatpump-telemetry
    pattern: home/heatpump/telemetry
    description: Heat pump telemetry data
    publisher: heatpump-mqtt
    subscribers:
      - mqtt-to-timescale
    qos: 1
    schema: heatpump-telemetry-payload

  - name: sensor-state
    pattern: home/sensors/+/state
    description: Sensor state data (wildcard)
    publisher: sensor-service
    subscribers:
      - mqtt-to-timescale
    qos: 1
    schema: sensor-state-payload

naming-convention:
  pattern: "{domain}/{service}/{device}/{message-type}"
  examples:
    - home/heatpump/hp-01/telemetry
    - home/sensors/kitchen/state
    - home/energy/meter-01/reading
```

## Code Generation from Contracts

### Rust Example
You can generate Rust structs from JSON Schema using tools like:
- `schemars` for generating schemas from Rust types
- `typify` for generating Rust types from JSON Schema

### TypeScript Example
Generate TypeScript types from JSON Schema:
```bash
npm install -g json-schema-to-typescript
json2ts -i contracts/mqtt-to-timescale/data-models/telemetry-row.schema.json -o src/types/telemetry.ts
```

## Validation

### Validate AsyncAPI Spec
```bash
npm install -g @asyncapi/cli
asyncapi validate contracts/mqtt-to-timescale/asyncapi/mqtt-ingestion.yaml
```

### Validate JSON Schema
```bash
npm install -g ajv-cli
ajv validate -s contracts/mqtt-to-timescale/data-models/telemetry-row.schema.json -d examples/telemetry.json
```

