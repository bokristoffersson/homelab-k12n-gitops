# API Reference

Base URL: `https://homelab.k12n.com`

The full specification, with parameters and response schemas, is
[`docs/api/openapi.json`](https://github.com/bokristoffersson/homelab-k12n-gitops/blob/main/docs/api/openapi.json).
This page is the overview.

## Authentication

Every `/api/v1` endpoint needs an Authelia access token:

```http
Authorization: Bearer <access token>
```

Browser sessions get the header added by oauth2-proxy; native apps and services send it
themselves. The API validates JWTs against Authelia's JWKS endpoint and falls back to
token introspection for opaque tokens. Each group of endpoints needs a scope.

## Endpoints

All endpoints are `GET`. The API never writes.

### Energy (`read:energy`)

| Path | Returns |
|---|---|
| `/api/v1/energy/latest` | The most recent reading |
| `/api/v1/energy/hourly-total` | Energy used so far in the current hour |
| `/api/v1/energy/history` | Hourly values over a time range |
| `/api/v1/energy/daily-summary` | Totals per day |
| `/api/v1/energy/monthly-summary` | Totals per month |
| `/api/v1/energy/yearly-summary` | Totals per year |

### Heat pump (`read:heatpump`)

| Path | Returns |
|---|---|
| `/api/v1/heatpump/latest` | The most recent status |
| `/api/v1/heatpump/daily-summary` | Statistics per day |

### Temperature (`read:temperature`)

| Path | Returns |
|---|---|
| `/api/v1/temperature/latest` | The latest reading from one sensor |
| `/api/v1/temperature/all-latest` | The latest reading from every sensor |
| `/api/v1/temperature/history` | Readings from one sensor over a time range |

### Other

| Path | Returns |
|---|---|
| `/api/v1/user/info` | The logged-in user and their access token, from the oauth2-proxy headers. The SPA uses it to open the WebSocket. |
| `/health` | `200` when the service is up. No authentication. |

Heat pump settings and smart plugs are served by homelab-settings-api, and spot prices
by spotprice-api, under the same host.

## Status codes

| Code | Meaning |
|---|---|
| `200` | Success |
| `401` | Missing or invalid token |
| `403` | The token lacks the scope for the endpoint |
| `404` | No data found |
| `500` | Database or server error |

## CORS

Handled by Traefik, not the API. Allowed origins: `https://homelab.k12n.com`,
`http://localhost:5173` and `http://localhost:8080`.
