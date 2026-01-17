# API Reference

Complete reference for all Homelab API endpoints.

## Authentication

All `/api/v1` endpoints require JWT authentication via Authentik OIDC.
The MCP endpoint (`/mcp`) is internal-only and does not require auth.
Do not expose `/mcp` on public ingress.

```http
Authorization: Bearer <jwt_token>
```

The API validates JWT signatures using Authentik's JWKS endpoint.

## MCP (Model Context Protocol)

MCP is available over HTTP+SSE for internal clients like Cursor or Claude.

### SSE Connection

```http
GET /mcp
```

Returns a Server-Sent Events stream with a ready event and periodic keep-alives.

### JSON-RPC Calls

```http
POST /mcp
Content-Type: application/json
```

Initialize:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "clientInfo": { "name": "cursor", "version": "1.0.0" }
  }
}
```

List tools:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

Call a tool (hourly energy):
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "energy_hourly_consumption",
    "arguments": {
      "from": "2026-01-15T00:00:00Z",
      "to": "2026-01-16T00:00:00Z"
    }
  }
}
```

Call a tool (daily peak hour):
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "energy_peak_hour_day",
    "arguments": {
      "day": "2026-01-15T00:00:00Z"
    }
  }
}
```

Call a tool (heatpump summary):
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "heatpump_daily_summary",
    "arguments": {
      "from": "2026-01-01T00:00:00Z",
      "to": "2026-01-08T00:00:00Z",
      "device_id": "heatpump-01"
    }
  }
}
```

Notes:
- All timestamps are interpreted as UTC unless an offset is included.
- Tools return JSON payloads in `content[].json`.

## Energy Endpoints

### Get Latest Energy Reading

```http
GET /api/v1/energy/latest
```

Returns the most recent power consumption measurement.

**Response**:
```json
{
  "timestamp": "2026-01-08T19:00:00Z",
  "power_w": 2450.5,
  "voltage": 230.2,
  "current": 10.64
}
```

### Get 24-Hour Energy History

```http
GET /api/v1/energy/24h
```

Returns energy readings from the last 24 hours, aggregated at 5-minute intervals using TimescaleDB continuous aggregates.

**Response**:
```json
[
  {
    "timestamp": "2026-01-08T19:00:00Z",
    "avg_power_w": 2450.5,
    "min_power_w": 2100.0,
    "max_power_w": 2800.0
  },
  ...
]
```

## Heat Pump Endpoints

### Get Latest Heat Pump Status

```http
GET /api/v1/heatpump/latest
```

Returns current heat pump operating status.

**Response**:
```json
{
  "timestamp": "2026-01-08T19:00:00Z",
  "mode": "heating",
  "supply_temp": 35.5,
  "return_temp": 28.2,
  "outdoor_temp": -5.0,
  "cop": 3.2
}
```

### Get 24-Hour Heat Pump History

```http
GET /api/v1/heatpump/24h
```

Returns heat pump metrics from the last 24 hours.

## Temperature Endpoints

### Get Latest Temperature Readings

```http
GET /api/v1/temperature/latest
```

Returns most recent temperature and humidity from Shelly H&T sensors.

**Response**:
```json
{
  "timestamp": "2026-01-08T19:00:00Z",
  "temperature_c": 21.5,
  "humidity_percent": 45.0,
  "sensor_id": "shellyhtg3-e4b32322a0f4"
}
```

### Get 24-Hour Temperature History

```http
GET /api/v1/temperature/24h
```

Returns temperature readings from the last 24 hours.

## Error Responses

All endpoints return standard HTTP status codes:

- `200 OK` - Success
- `401 Unauthorized` - Invalid or missing JWT
- `404 Not Found` - No data found
- `500 Internal Server Error` - Database or server error

Error response format:
```json
{
  "error": "Error description"
}
```

## Rate Limiting

No rate limiting is currently enforced, but authenticated access is required.

## CORS

The API allows cross-origin requests from:
- `https://heatpump.k12n.com` (production frontend)
- `http://localhost:5173` (local development)
