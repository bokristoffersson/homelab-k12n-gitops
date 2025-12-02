# Contract-First Development Strategy

## Overview

Contract-first development is a methodology where service interfaces, data models, and communication protocols are defined **before** implementation. This approach ensures:

- **Clear boundaries** between services
- **Type safety** and validation
- **Parallel development** of services
- **Versioning** and **backward compatibility**
- **Documentation** as code
- **Testing** against contracts

## Contract Types

### 1. **API Contracts (OpenAPI/Swagger)**
For REST/HTTP APIs:
- Request/response schemas
- Endpoints and methods
- Authentication/authorization
- Error responses
- Status codes

### 2. **AsyncAPI Contracts**
For event-driven/messaging systems:
- MQTT topics and message schemas
- Kafka/Redpanda topics
- Message payloads
- QoS levels
- Subscription patterns

### 3. **Data Model Contracts**
- Database schemas (SQL DDL)
- JSON Schema definitions
- Protobuf schemas (if used)
- TypeScript types
- Rust structs/enums

### 4. **Service Contracts**
- Service dependencies
- Health check endpoints
- Metrics endpoints
- Service discovery configuration

## Repository Structure

```
homelab-k12n-gitops/
├── contracts/                          # Root contracts directory
│   ├── README.md                       # Contract overview and guidelines
│   │
│   ├── mqtt-to-timescale/              # Service owner: mqtt-to-timescale
│   │   ├── README.md                   # Service contract overview
│   │   ├── data-models/
│   │   │   ├── telemetry.schema.json   # JSON Schema for telemetry data
│   │   │   └── database/
│   │   │       └── telemetry.sql       # Database schema (DDL)
│   │   ├── asyncapi/
│   │   │   └── mqtt-ingestion.yaml     # AsyncAPI spec for MQTT topics consumed
│   │   └── openapi/                    # If service exposes HTTP API
│   │       └── api.yaml
│   │
│   ├── heatpump-mqtt/                  # Service owner: heatpump-mqtt
│   │   ├── README.md
│   │   ├── data-models/
│   │   │   └── heatpump-telemetry.schema.json
│   │   ├── asyncapi/
│   │   │   └── mqtt-publisher.yaml     # AsyncAPI spec for MQTT topics published
│   │   └── examples/
│   │       └── sample-messages.json    # Example message payloads
│   │
│   ├── mosquitto/                      # Service owner: mosquitto
│   │   ├── README.md
│   │   ├── asyncapi/
│   │   │   └── broker-config.yaml      # MQTT broker configuration schema
│   │   └── topics/
│   │       └── topic-naming-convention.md
│   │
│   ├── grafana/                        # Service owner: grafana
│   │   ├── README.md
│   │   ├── openapi/
│   │   │   └── grafana-api.yaml        # Grafana API contracts (if custom)
│   │   └── data-models/
│   │       └── dashboard-schema.json   # Dashboard data model
│   │
│   └── shared/                         # Shared contracts across services
│       ├── common-types/
│       │   ├── timestamp.schema.json
│       │   └── device-id.schema.json
│       └── mqtt-topics/
│           └── topic-registry.yaml     # Central registry of all MQTT topics
│
└── [existing structure...]
```

## Best Practices

### 1. **Service Ownership**
- Each service owns its contracts in its own folder
- Service owner is responsible for:
  - Defining contracts before implementation
  - Versioning contracts
  - Maintaining backward compatibility
  - Documenting breaking changes

### 2. **Contract Versioning**
Use semantic versioning for contracts:
```
contracts/
└── mqtt-to-timescale/
    ├── v1/
    │   ├── asyncapi/
    │   └── data-models/
    └── v2/
        ├── asyncapi/
        └── data-models/
```

Or use version in filenames:
```
asyncapi/
├── mqtt-ingestion-v1.yaml
└── mqtt-ingestion-v2.yaml
```

### 3. **Schema Validation**
- Use JSON Schema for data validation
- Reference schemas in AsyncAPI/OpenAPI specs
- Validate at runtime (if possible)
- Generate code from schemas (e.g., Rust structs from JSON Schema)

### 4. **Documentation**
Each contract folder should include:
- `README.md` explaining:
  - What the service does
  - What contracts it defines
  - How to use the contracts
  - Version history
  - Breaking changes

### 5. **Examples**
Include example payloads and usage:
```
contracts/
└── heatpump-mqtt/
    └── examples/
        ├── telemetry-message.json
        └── status-message.json
```

### 6. **Shared Contracts**
- Common types go in `contracts/shared/`
- Services reference shared contracts
- Avoid duplication

### 7. **Topic Naming Conventions**
For MQTT/event-driven systems:
- Document topic structure: `{domain}/{service}/{device}/{message-type}`
- Example: `home/heatpump/hp-01/telemetry`
- Document wildcards: `home/heatpump/+/telemetry`

### 8. **Database Schema Contracts**
- Keep DDL in contracts
- Version migrations
- Document schema changes
- Include indexes and constraints

## Contract-First Workflow

### 1. **Design Phase**
1. Identify service boundaries
2. Define data models
3. Design API/event contracts
4. Document in contract files

### 2. **Review Phase**
1. Review contracts with team
2. Validate against requirements
3. Check for conflicts with other services
4. Update contracts based on feedback

### 3. **Implementation Phase**
1. Generate code from contracts (if possible)
2. Implement service according to contract
3. Validate implementation against contract
4. Write tests against contract

### 4. **Evolution Phase**
1. Propose contract changes
2. Document breaking changes
3. Version new contracts
4. Deprecate old versions with timeline

## Tools and Standards

### Recommended Tools
- **OpenAPI 3.x** for REST APIs
- **AsyncAPI 3.x** for event-driven APIs
- **JSON Schema** for data validation
- **Protobuf** (optional) for binary serialization
- **TypeScript** types (can generate from JSON Schema)
- **Rust** types (can generate from JSON Schema)

### Validation Tools
- **Spectral** for linting OpenAPI/AsyncAPI specs
- **Ajv** for JSON Schema validation
- **Contract testing** tools (e.g., Pact)

## Example: MQTT-to-Timescale Contract

Based on your existing service, here's how a contract would look:

### Structure
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

### Key Contracts to Define
1. **MQTT Topic Patterns** consumed
2. **Message Payload Schema** (JSON Schema)
3. **Database Schema** (TimescaleDB)
4. **Field Mapping Rules** (JSONPath expressions)
5. **Timestamp Formats** supported

## Benefits

1. **Clear Communication**: Contracts serve as documentation
2. **Type Safety**: Generate types from schemas
3. **Testing**: Test against contracts, not implementations
4. **Parallel Development**: Teams can work independently
5. **Versioning**: Clear versioning strategy
6. **Compatibility**: Easier to maintain backward compatibility
7. **Onboarding**: New developers understand interfaces quickly

## Migration Strategy

If you have existing services:

1. **Document existing contracts** retroactively
2. **Start contract-first** for new services
3. **Gradually migrate** existing services
4. **Validate** existing implementations against contracts

## Next Steps

1. Create the `contracts/` directory structure
2. Document existing services' contracts
3. Set up validation tooling
4. Establish review process for contract changes
5. Generate code from contracts where possible

