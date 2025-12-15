````javascript
---
name: Chunk 7 Frontend Setup and Services
overview: Initialize React+TypeScript frontend project and create API client and authentication services
todos:
    - id: init-project
    content: Initialize Vite + React + TypeScript project
    status: pending
    - id: install-deps
    content: Install react-router-dom, axios, recharts, @tanstack/react-query
    status: pending
    - id: create-api-client
    content: Create api.ts with axios client and interceptors
    status: pending
    - id: create-auth-service
    content: Create auth.ts with login, logout, token management
    status: pending
    - id: create-types
    content: Create TypeScript type definitions
    status: pending
    - id: setup-project-structure
    content: Create directory structure and basic App.tsx
    status: pending
    - id: test-services
    content: Verify API client and auth service work correctly
    status: pending
---

# Chunk 7: Frontend Setup and Services

## Overview

Set up the React+TypeScript frontend project with Vite, install dependencies, and create the API client and authentication service layer that will be used by components.

## Files to Create

### 1. Project Initialization

**File**: `applications/heatpump-web/package.json`

- React 18, TypeScript, Vite
- Dependencies: react-router-dom, axios, recharts, @tanstack/react-query
- Dev dependencies: @types/node, @types/react, @types/react-dom
- Build and dev scripts

**File**: `applications/heatpump-web/vite.config.ts`

- Vite config with React plugin
- Environment variable handling
- Build configuration

**File**: `applications/heatpump-web/tsconfig.json`

- TypeScript configuration
- Path aliases if needed
- React JSX settings

**File**: `applications/heatpump-web/index.html`

- HTML entry point
- Root div for React app

### 2. API Client Service

**File**: `applications/heatpump-web/src/services/api.ts`

- Axios client with base URL from env (VITE_API_URL)
- Request interceptor to add JWT token from localStorage
- Response interceptor for 401 auto-logout
- Export axios instance

### 3. Authentication Service

**File**: `applications/heatpump-web/src/services/auth.ts`

- `login(username, password)` - Call login endpoint, store token
- `logout()` - Clear token and redirect
- `getToken()` - Get token from localStorage
- `isAuthenticated()` - Check if token exists and is valid
- Token storage in localStorage
- Token validation (check expiry)

### 4. Type Definitions

**File**: `applications/heatpump-web/src/types/index.ts`

- API response types
- Energy data types
- Heatpump status types
- Auth types

## Implementation Steps

1. Initialize Vite + React + TypeScript project
2. Install all required dependencies
3. Set up project structure (src/services, src/types, src/components)
4. Create API client with axios and interceptors
5. Create authentication service with login/logout/token management
6. Create TypeScript type definitions
7. Test API client can make requests (mock backend or use actual)
8. Test auth service login/logout flow

## Environment Variables

**File**: `applications/heatpump-web/.env.example`

```
VITE_API_URL=http://localhost:8080
```

## Project Structure

```
applications/heatpump-web/
├── src/
│   ├── services/
│   │   ├── api.ts
│   │   └── auth.ts
│   ├── types/
│   │   └── index.ts
│   ├── components/
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Verification

```bash
cd applications/heatpump-web
npm install
npm run dev
# Check browser console for any errors
# Test API client in browser console
```

## Dependencies

- Chunk 6: Backend API should be deployed and accessible (for testing)

## Next Chunk

Chunk 8: Frontend Components


````