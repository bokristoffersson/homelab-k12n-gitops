# Architecture

## System Overview

Heatpump Web is a frontend application that communicates with backend services via REST API and WebSocket connections.

```
┌─────────────────┐
│  Heatpump Web   │
│     (SPA)       │
└────────┬────────┘
         │
         ├──HTTP──► Homelab API (REST)
         │           ├─ /api/v1/energy/*
         │           ├─ /api/v1/heatpump/*
         │           └─ /api/v1/temperature/*
         │
         └──WS────► Energy WS (WebSocket)
                     └─ Real-time energy stream
```

## Components

### PowerGauge
- Displays real-time power consumption
- Connects to Energy WS via WebSocket
- Uses JWT token for authentication
- Shows circular progress ring (0-17.5 kW)

### Dashboard
- Main container component
- Manages multiple data queries
- Handles error states and loading
- Displays hourly totals, status, and charts

### Authentication Flow
1. User accesses https://heatpump.k12n.com
2. Traefik routes to heatpump-web service (public)
3. Frontend makes API call to /api/v1/user/info
4. oauth2-proxy middleware validates session
5. Backend returns JWT access token
6. Frontend uses token for WebSocket connection

## Deployment Architecture

```
Cloudflare Tunnel
       │
       ▼
   Traefik
       │
       ├─► heatpump-web (Nginx) - Public
       │
       └─► homelab-api - Protected by oauth2-proxy
```

## Runtime Configuration

Environment configuration is injected at container startup via nginx envsubst:

- `VITE_API_URL`: Backend API base URL
- `VITE_AUTHENTIK_URL`: Authentik OIDC provider URL
- `VITE_OAUTH_CLIENT_ID`: OAuth2 client ID
- `VITE_OAUTH_REDIRECT_URI`: OAuth2 redirect URI

See `nginx.conf` and Dockerfile for implementation details.
