# Contract-First Quick Reference

## Core Principles

1. **Define contracts BEFORE implementation**
2. **Service owns its contracts** in dedicated folder
3. **Version contracts** for backward compatibility
4. **Validate** implementations against contracts
5. **Document** everything in contracts

## Contract Types Checklist

For each service, define:

- [ ] **AsyncAPI** - For MQTT/Kafka/event-driven APIs
- [ ] **OpenAPI** - For REST/HTTP APIs  
- [ ] **JSON Schema** - For data models
- [ ] **Database Schema** - SQL DDL files
- [ ] **Examples** - Sample payloads/messages
- [ ] **README** - Service contract documentation

## Folder Structure Template

```
contracts/
└── {service-name}/
    ├── README.md
    ├── data-models/
    │   ├── {model}.schema.json
    │   └── database/
    │       └── {table}.sql
    ├── asyncapi/
    │   └── {api-name}.yaml
    ├── openapi/          # If HTTP API exists
    │   └── api.yaml
    └── examples/
        └── sample-*.json
```

## Workflow

1. **Design** → Create contract files
2. **Review** → Get team approval
3. **Implement** → Build service to match contract
4. **Validate** → Test against contract
5. **Evolve** → Version new contracts, deprecate old

## Standards

- **OpenAPI**: 3.x
- **AsyncAPI**: 3.x
- **JSON Schema**: Draft 2020-12
- **Versioning**: Semantic versioning (v1, v2, etc.)

## Tools

- **Validation**: Spectral (OpenAPI/AsyncAPI), Ajv (JSON Schema)
- **Code Generation**: typify (Rust), json-schema-to-typescript
- **Linting**: Spectral rules

## Quick Commands

```bash
# Validate AsyncAPI
asyncapi validate contracts/{service}/asyncapi/*.yaml

# Validate JSON Schema
ajv validate -s contracts/{service}/data-models/*.schema.json

# Generate TypeScript types
json2ts -i contracts/{service}/data-models/*.schema.json -o src/types/
```

## See Also

- [Full Strategy Guide](./CONTRACT_FIRST_STRATEGY.md)
- [Concrete Examples](./CONTRACT_EXAMPLES.md)

