````javascript
---
name: Chunk 6 Backend Deployment
overview: Integrate API server into main application and create Kubernetes deployment manifests
todos:
    - id: update-main-rs
    content: Start API server in background task alongside consumer
    status: pending
    - id: handle-shutdown
    content: Implement graceful shutdown for both consumer and API
    status: pending
    - id: create-service-yaml
    content: Create service.yaml for Kubernetes ClusterIP service
    status: pending
    - id: create-auth-secret
    content: Create auth-secret-sealed.yaml using kubeseal (manual step)
    status: pending
    - id: update-deployment
    content: Add containerPort and env vars to deployment.yaml
    status: pending
    - id: update-kustomization
    content: Add service.yaml and auth-secret-sealed.yaml to resources
    status: pending
    - id: test-locally
    content: Test API server starts and responds to requests
    status: pending
    - id: verify-deployment
    content: Deploy and verify service is accessible in cluster
    status: pending
---

# Chunk 6: Backend Deployment

## Overview

Start the API server alongside the existing consumer in the main application, and create all Kubernetes manifests for deployment including service, secrets, and deployment updates.

## Files to Modify

### 1. Main Application

**File**: `applications/redpanda-sink/src/main.rs`

- Start API server in background task alongside existing consumer
- Pass config and database pool to API router
- Handle graceful shutdown for both consumer and API server
- Only start API if enabled in config

### 2. Update Deployment

**File**: `gitops/apps/base/redpanda-sink/deployment.yaml`

- Add containerPort 8080
- Add JWT_SECRET and ADMIN_PASSWORD_HASH env vars from secret
- Reference auth-secret-sealed.yaml

## Files to Create

### 3. Kubernetes Service

**File**: `gitops/apps/base/redpanda-sink/service.yaml` (NEW)

- ClusterIP service on port 8080
- Selector matching deployment labels
- Port 8080 -> 8080

### 4. Auth Secret (Manual Creation)

**File**: `gitops/apps/base/redpanda-sink/auth-secret-sealed.yaml` (CREATE MANUALLY)

- Use kubeseal to create sealed secret
- Contains JWT_SECRET (random string)
- Contains ADMIN_PASSWORD_HASH (bcrypt hash of admin password)
- Instructions for manual creation

### 5. Update Kustomization

**File**: `gitops/apps/base/redpanda-sink/kustomization.yaml`

- Add service.yaml to resources
- Add auth-secret-sealed.yaml to resources

## Implementation Steps

1. Update main.rs to start API server in background task
2. Handle graceful shutdown for both services
3. Create service.yaml for Kubernetes
4. Create instructions/documentation for creating auth-secret-sealed.yaml
5. Update deployment.yaml with port and env vars
6. Update kustomization.yaml to include new resources
7. Test locally (start API server, verify endpoints work)
8. Build Docker image and test
9. Commit and push for GitOps deployment

## Creating Auth Secret

```bash
# Create unsealed secret
kubectl create secret generic auth-secret \
  --from-literal=JWT_SECRET="$(openssl rand -hex 32)" \
  --from-literal=ADMIN_PASSWORD_HASH="$(cargo run --bin hash-password <admin-password>)" \
  --dry-run=client -o yaml > auth-secret.yaml

# Seal it
kubeseal -o yaml < auth-secret.yaml > auth-secret-sealed.yaml
```

## Verification

```bash
# Local testing
cd applications/redpanda-sink
cargo run
# In another terminal
curl http://localhost:8080/health

# After deployment
kubectl get svc -n redpanda-sink
kubectl port-forward svc/redpanda-sink 8080:8080 -n redpanda-sink
curl http://localhost:8080/health
```

## Dependencies

- Chunks 1-5: All backend implementation must be complete

## Next Chunk

Chunk 7: Frontend Setup and Services


````