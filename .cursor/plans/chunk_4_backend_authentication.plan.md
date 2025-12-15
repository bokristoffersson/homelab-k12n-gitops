# Chunk 4: Backend Authentication

## Overview

Implement authentication module with JWT token generation/validation and password hashing. This provides the security layer for protected API endpoints.

## Files to Create

### 1. JWT Module

**File**: `applications/redpanda-sink/src/auth/jwt.rs`

- `create_token()` - Generate JWT token from username and expiry
- `validate_token()` - Validate JWT token and extract claims
- Token claims struct (username, exp, etc.)
- Error handling for invalid/expired tokens

### 2. Password Module

**File**: `applications/redpanda-sink/src/auth/password.rs`

- `hash_password()` - Hash password with bcrypt
- `verify_password()` - Verify password against hash
- Use bcrypt with appropriate cost factor (default 12)

### 3. Auth Module

**File**: `applications/redpanda-sink/src/auth/mod.rs`

- Export jwt and password modules
- Common auth types/errors if needed

## Implementation Details

### JWT Implementation

- Use jsonwebtoken crate
- Secret from config (AuthConfig.jwt_secret)
- Standard claims: username, exp (expiry), iat (issued at)
- Expiry duration from config (jwt_expiry_hours)

### Password Implementation

- Use bcrypt crate
- Default cost factor of 12
- Handle password verification errors gracefully

## Implementation Steps

1. Create auth/jwt.rs with token creation and validation
2. Create auth/password.rs with hashing and verification
3. Create auth/mod.rs to export modules
4. Update main.rs/lib.rs to include auth module
5. Write tests for auth functions
6. Verify JWT tokens can be created and validated

## Testing

Test password hashing and verification:

```rust
let hash = hash_password("test123")?;
assert!(verify_password("test123", &hash)?);
assert!(!verify_password("wrong", &hash)?);
```

Test JWT creation and validation:

```rust
let token = create_token("admin", 24)?;
let claims = validate_token(&token)?;
assert_eq!(claims.username, "admin");
```

## Verification

```bash
cd applications/redpanda-sink
cargo test auth
cargo check
```

## Dependencies

- Chunk 2: Config structure (needs AuthConfig)

## Next Chunk

Chunk 5: Backend API Endpoints