# API Documentation

Specifications for the homelab APIs. Every service is reached at
`https://homelab.k12n.com`.

| File | Format | Covers |
|---|---|---|
| `openapi.json` | OpenAPI 3 | homelab-api (read-only telemetry), homelab-settings-api (heat pump settings, power plugs and schedules) and the `/ws/energy` endpoint |
| `energy-ws.yaml` | AsyncAPI 2.6 | The WebSocket message protocol for real-time energy data |
| [IOS_MACOS_GUIDE.md](IOS_MACOS_GUIDE.md) | Guide | Building a native Swift client |

The spot price API is described in
[../SPOTPRICE_APP_INTEGRATION.md](../SPOTPRICE_APP_INTEGRATION.md).

## Services

| Service | Paths | Notes |
|---|---|---|
| homelab-api | `/api/v1/energy/*`, `/api/v1/heatpump/*`, `/api/v1/temperature/*` | Read-only, served from TimescaleDB |
| homelab-settings-api | `/api/v1/heatpump/settings/*`, `/api/v1/plugs/*` | Writes go through a transactional outbox to MQTT; command status is available from the outbox endpoints |
| energy-ws | `wss://homelab.k12n.com/ws/energy?token=<access token>` | Streams energy readings from Redpanda |

## Authentication

All APIs take access tokens from Authelia (`https://auth.k12n.com`) and authorize by
scope, for example `read:energy` or `write:plugs`.

| Caller | Flow |
|---|---|
| Browser | Session cookie through oauth2-proxy |
| Native app | Authorization Code with PKCE, client `homelab-macos` |
| Service | Client Credentials |

See [../AUTHENTICATION.md](../AUTHENTICATION.md).

## Working with the specs

```bash
# Validate
npx @redocly/cli lint openapi.json
npx @asyncapi/cli validate energy-ws.yaml

# Render HTML
npx @redocly/cli build-docs openapi.json -o homelab-api.html

# Generate a client (TypeScript shown; see the iOS guide for Swift)
openapi-generator generate -i openapi.json -g typescript-fetch -o ts-client
```
