# API Documentation

This directory contains OpenAPI/AsyncAPI specifications for all Homelab APIs.

## Specifications

| File | Format | Description |
|------|--------|-------------|
| `homelab-api.yaml` | OpenAPI 3.0 | Read-only REST API for telemetry data |
| `heatpump-settings-api.yaml` | OpenAPI 3.0 | Settings management API with transactional outbox |
| `energy-ws.yaml` | AsyncAPI 2.6 | WebSocket API for real-time energy streaming |

## Usage

### Validate Specs

```bash
# Install OpenAPI tools
npm install -g @redocly/cli

# Validate OpenAPI specs
redocly lint homelab-api.yaml
redocly lint heatpump-settings-api.yaml

# Validate AsyncAPI spec
npm install -g @asyncapi/cli
asyncapi validate energy-ws.yaml
```

### Generate Documentation

```bash
# Generate HTML docs with Redoc
redocly build-docs homelab-api.yaml -o homelab-api.html
redocly build-docs heatpump-settings-api.yaml -o heatpump-settings-api.html

# Generate AsyncAPI docs
asyncapi generate fromTemplate energy-ws.yaml @asyncapi/html-template -o energy-ws-docs
```

### Generate Client Code

See [IOS_MACOS_GUIDE.md](./IOS_MACOS_GUIDE.md) for Swift client generation.

For other languages:

```bash
# TypeScript
openapi-generator generate -i homelab-api.yaml -g typescript-fetch -o ts-client

# Kotlin
openapi-generator generate -i homelab-api.yaml -g kotlin -o kotlin-client

# Python
openapi-generator generate -i homelab-api.yaml -g python -o python-client

# Go
openapi-generator generate -i homelab-api.yaml -g go -o go-client
```

## API Overview

### Homelab API

Base URL: `https://api.k12n.com`

Read-only REST API serving:
- Energy consumption data (smart meter)
- Heatpump telemetry (IVT 495)
- Temperature/humidity readings (Shelly sensors)

All `/api/v1/*` endpoints require JWT authentication.

### Heatpump Settings API

Base URL: `https://heatpump-settings.k12n.com`

Settings management using transactional outbox pattern:
1. `PATCH` requests queue commands
2. Background worker publishes to Kafka
3. Heatpump controller applies settings
4. Status tracked via outbox endpoints

### Energy WebSocket

URL: `wss://energy-ws.k12n.com/ws/energy?token=<jwt>`

Real-time streaming of energy data from Redpanda/Kafka.

## Authentication

All APIs use JWT tokens from Authentik (OIDC provider).

- **Web apps**: Authorization Code flow
- **Native apps**: Authorization Code with PKCE
- **Machine-to-machine**: Client Credentials flow

See [../AUTHENTICATION.md](../AUTHENTICATION.md) for details.
