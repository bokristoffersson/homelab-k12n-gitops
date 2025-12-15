````javascript
---
name: Chunk 5 Backend API Endpoints
overview: Implement REST API endpoints, handlers, routes, models, and middleware for the heatpump monitoring API
todos:
    - id: create-api-models
    content: Create models.rs with request/response types for all endpoints
    status: pending
    - id: create-auth-handler
    content: Implement auth.rs handler for login endpoint
    status: pending
    - id: create-energy-handlers
    content: Implement energy.rs handlers for all energy endpoints
    status: pending
    - id: create-heatpump-handler
    content: Implement heatpump.rs handler for latest endpoint
    status: pending
    - id: create-health-handler
    content: Implement health.rs handler for health check
    status: pending
    - id: create-auth-middleware
    content: Implement auth middleware for JWT validation
    status: pending
    - id: create-routes
    content: Wire up all routes with middleware in routes.rs
    status: pending
    - id: create-api-mod
    content: Create api/mod.rs to export modules and create router
    status: pending
---

# Chunk 5: Backend API Endpoints

## Overview

Implement the complete API layer with routes, handlers, models, and authentication middleware. This exposes the data access layer through REST endpoints.

## Files to Create

### 1. API Models

**File**: `applications/redpanda-sink/src/api/models.rs`

- Request/response models for all endpoints
- LoginRequest, LoginResponse
- EnergyResponse, HeatpumpResponse types
- Error response types

### 2. API Handlers

**File**: `applications/redpanda-sink/src/api/handlers/auth.rs`

- `login()` handler for POST /api/v1/auth/login

**File**: `applications/redpanda-sink/src/api/handlers/energy.rs`

- `get_latest()` - GET /api/v1/energy/latest
- `get_hourly_total()` - GET /api/v1/energy/hourly-total
- `get_hourly_history()` - GET /api/v1/energy/history

**File**: `applications/redpanda-sink/src/api/handlers/heatpump.rs`

- `get_latest()` - GET /api/v1/heatpump/latest

**File**: `applications/redpanda-sink/src/api/handlers/health.rs`

- `health()` - GET /health

### 3. API Middleware

**File**: `applications/redpanda-sink/src/api/middleware/auth.rs`

- JWT authentication middleware
- Extract token from Authorization header
- Validate token and attach user to request

### 4. API Routes

**File**: `applications/redpanda-sink/src/api/routes.rs`

- Define all API routes
- Public routes: /health, /api/v1/auth/login
- Protected routes: /api/v1/energy/*, /api/v1/heatpump/*
- Apply auth middleware to protected routes

### 5. API Module

**File**: `applications/redpanda-sink/src/api/mod.rs`

- Export all API modules
- Create router function that returns Axum Router

## API Endpoints

### Public

- `POST /api/v1/auth/login` - Authenticate and get JWT token
- `GET /health` - Health check endpoint

### Protected (require JWT)

- `GET /api/v1/energy/latest` - Latest energy reading
- `GET /api/v1/energy/hourly-total` - Total energy this hour
- `GET /api/v1/energy/history?from=...&to=...` - Hourly history
- `GET /api/v1/heatpump/latest?device_id=...` - Latest heatpump status

## Implementation Steps

1. Create models.rs with all request/response types
2. Create handlers for each endpoint (auth, energy, heatpump, health)
3. Create auth middleware for JWT validation
4. Create routes.rs to wire up all routes with middleware
5. Create api/mod.rs to export and create router
6. Update main.rs to use API router (in next chunk)
7. Test endpoints with curl or Postman

## Verification

```bash
cd applications/redpanda-sink
cargo check
cargo test api

# Manual testing (after starting server)
curl http://localhost:8080/health
curl -X POST http://localhost:8080/api/v1/auth/login -d '{"username":"admin","password":"..."}'
```

## Dependencies

- Chunk 2: Config and module structure
- Chunk 3: Repository layer (handlers use repositories)
- Chunk 4: Authentication module (JWT and password)

## Next Chunk

Chunk 6: Backend Deployment


````