````javascript
---
name: Chunk 2 Backend Foundation
overview: Set up backend API foundation: dependencies, configuration, and module structure
todos:
    - id: add-dependencies
    content: Add axum, tower, jsonwebtoken, bcrypt to Cargo.toml
    status: pending
    - id: create-modules
    content: Create api/, repositories/, auth/ directories with mod.rs files
    status: pending
    - id: update-config-structs
    content: Add ApiConfig, AuthConfig, User structs to config.rs
    status: pending
    - id: update-config-parsing
    content: Update config parsing logic to read API and auth config
    status: pending
    - id: update-configmap
    content: Add api and auth sections to configmap.yaml
    status: pending
    - id: verify-compilation
    content: Run cargo check to verify everything compiles
    status: pending
---

# Chunk 2: Backend Foundation

## Overview

Establish the foundation for the backend API by adding dependencies, updating configuration, and creating module structure. This enables subsequent chunks to build the API layer.

## Files to Create/Modify

### 1. Add Dependencies

**File**---
name: Chunk 2 Backend Foundation
overview: Set up backend API foundation: dependencies, configuration, and module structure
todos:
    - id: add-dependencies
    content: Add axum, tower, jsonwebtoken, bcrypt to Cargo.toml
    status: pending
    - id: create-modules
    content: Create api/, repositories/, auth/ directories with mod.rs files
    status: pending
    - id: update-config-structs
    content: Add ApiConfig, AuthConfig, User structs to config.rs
    status: pending
    - id: update-config-parsing
    content: Update config parsing logic to read API and auth config
    status: pending
    - id: update-configmap
    content: Add api and auth sections to configmap.yaml
    status: pending
    - id: verify-compilation
    content: Run cargo check to verify everything compiles
    status: pending
---

# Chunk 2: Backend Foundation

## Overview

Establish the foundation for the backend API by adding dependencies, updating configuration, and creating module structure. This enables subsequent chunks to build the API layer.

## Files to Create/Modify

### 1. Add Dependencies

**File**: `applications/redpanda-sink/Cargo.toml`

- Add: `axum = { version = "0.7", features = ["macros", "query"] }`
- Add: `tower = "0.4"`
- Add: `tower-http = { version = "0.5", features = ["cors", "trace"] }`
- Add: `jsonwebtoken = "9.2"`
- Add: `bcrypt = "0.15"`

### 2. Create Module Structure

Create new directories:

- `applications/redpanda-sink/src/api/` - API routes, handlers, models, middleware
- `applications/redpanda-sink/src/repositories/` - Data access layer (energy, heatpump)
- `applications/redpanda-sink/src/auth/` - JWT and password hashing

### 3. Update Configuration

**File**: `applications/redpanda-sink/src/config.rs`

- Add `ApiConfig` struct (enabled, host, port)
- Add `AuthConfig` struct (jwt_secret, jwt_expiry_hours, users)
- Add `User` struct (username, password_hash)
- Update Config struct to include api and auth fields
- Update config parsing to read API and auth configuration

**File**: `gitops/apps/base/redpanda-sink/configmap.yaml`

- Add `api:` section (enabled: true, host: "0.0.0.0", port: 8080)
- Add `auth:` section (jwt_secret from env, users array structure)

## Implementation Steps

1. Update Cargo.toml with new dependencies
2. Create module directories
3. Add mod.rs files to declare modules (empty for now)
4. Update config.rs with new structs and parsing logic
5. Update configmap.yaml with API/auth configuration structure
6. Run `cargo check` to verify dependencies compile
7. Test config loading with new fields

## Verification

```bash
cd applications/redpanda-sink
cargo check
cargo test
```

## Dependencies

- Chunk 1 (database migrations) should be completed first, but not strictly required for this chunk

## Next Chunk

Chunk 3: Backend Repository Layer

: `applications/redpanda-sink/Cargo.toml`

- Add: `axum = { version = "0.7", features = ["macros", "query"] }`
- Add: `tower = "0.4"`
- Add: `tower-http = { version = "0.5", features = ["cors", "trace"] }`
- Add: `jsonwebtoken = "9.2"`
- Add: `bcrypt = "0.15"`

### 2. Create Module Structure

Create new directories:

- `applications/redpanda-sink/src/api/` - API routes, handlers, models, middleware
- `applications/redpanda-sink/src/repositories/` - Data access layer (energy, heatpump)
- `applications/redpanda-sink/src/auth/` - JWT and password hashing

### 3. Update Configuration

**File**: `applications/redpanda-sink/src/config.rs`

- Add `ApiConfig` struct (enabled, host, port)
- Add `AuthConfig` struct (jwt_secret, jwt_expiry_hours, users)
- Add `User` struct (username, password_hash)
- Update Config struct to include api and auth fields
- Update config parsing to read API and auth configuration

**File**: `gitops/apps/base/redpanda-sink/configmap.yaml`

- Add `api:` section (enabled: true, host: "0.0.0.0", port: 8080)
- Add `auth:` section (jwt_secret from env, users array structure)

## Implementation Steps

1. Update Cargo.toml with new dependencies
2. Create module directories
3. Add mod.rs files to declare modules (empty for now)
4. Update config.rs with new structs and parsing logic
5. Update configmap.yaml with API/auth configuration structure
6. Run `cargo check` to verify dependencies compile
7. Test config loading with new fields

## Verification

```bash
cd applications/redpanda-sink
cargo check
cargo test
```

## Dependencies

- Chunk 1 (database migrations) should be completed first, but not strictly required for this chunk

## Next Chunk

Chunk 3: Backend Repository Layer

````