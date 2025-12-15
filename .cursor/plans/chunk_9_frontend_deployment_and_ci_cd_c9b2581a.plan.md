---
name: Chunk 9 Frontend Deployment and CI CD
overview: Create Dockerfile, Kubernetes manifests, and GitHub Actions CI/CD workflow for frontend deployment
todos:
  - id: create-dockerfile
    content: Create multi-stage Dockerfile with Node builder and nginx
    status: pending
  - id: create-nginx-config
    content: Create nginx.conf for SPA routing and API proxying
    status: pending
  - id: test-docker-build
    content: Test Docker build and run locally
    status: pending
  - id: create-k8s-manifests
    content: Create namespace, deployment, service manifests
    status: pending
  - id: create-kustomization
    content: Create kustomization.yaml for base and overlay
    status: pending
  - id: create-ghcr-secret
    content: Create ghcr-secret-sealed.yaml using kubeseal (manual)
    status: pending
  - id: create-ci-workflow
    content: Create GitHub Actions workflow for CI/CD
    status: pending
  - id: update-apps-yaml
    content: Add heatpump-web to GitOps apps.yaml
    status: pending
  - id: verify-deployment
    content: Deploy and verify frontend is accessible
    status: pending
---

# Chunk 9: Frontend Deployment and CI/CD

## Overview

Containerize the frontend application, create Kubernetes deployment manifests, and set up CI/CD pipeline for automated building and deployment.

## Files to Create

### 1. Docker Configuration

**File**: `applications/heatpump-web/Dockerfile`

- Multi-stage build
- Stage 1: Node.js builder - install deps, build production bundle
- Stage 2: nginx - copy built files, serve with nginx
- Use nginx:alpine as base
- Copy nginx.conf

**File**: `applications/heatpump-web/nginx.conf`

- Nginx configuration for SPA routing
- Serve index.html for all routes (try_files)
- Set VITE_API_URL via environment variable substitution
- Proper MIME types
- Gzip compression

### 2. Kubernetes Manifests

**File**: `gitops/apps/base/heatpump-web/namespace.yaml`

- Create heatpump-web namespace

**File**: `gitops/apps/base/heatpump-web/deployment.yaml`

- Deployment with nginx container
- VITE_API_URL env var pointing to backend service (redpanda-sink.redpanda-sink.svc.cluster.local:8080)
- Resource limits appropriate for nginx
- Readiness and liveness probes
- Image pull secret reference

**File**: `gitops/apps/base/heatpump-web/service.yaml`

- ClusterIP service on port 80
- Selector matching deployment labels

**File**: `gitops/apps/base/heatpump-web/kustomization.yaml`

- Kustomize configuration
- Include all resources
- Set namespace
- Image reference (can be overridden in overlay)

**File**: `gitops/apps/base/heatpump-web/ghcr-secret-sealed.yaml` (CREATE MANUALLY)

- Sealed secret for GHCR image pull
- Copy pattern from other apps (redpanda-sink, mqtt-input)
- Use kubeseal to create

**File**: `gitops/apps/homelab/heatpump-web/kustomization.yaml` (overlay)

- Reference base
- Apply homelab-specific configurations if needed

### 3. CI/CD Workflow

**File**: `.github/workflows/heatpump-web-ci.yml` (NEW)

- Trigger: On push/PR to `applications/heatpump-web/**`
- Jobs:
  - **test**: Run TypeScript type check, linting, build verification
  - **build**: Build production bundle (verify it works)
  - **docker**: Build and push multi-arch Docker image (linux/amd64, linux/arm64) to GHCR
- Use Docker Buildx with QEMU for multi-arch builds
- Tags: latest (main branch), branch name, PR number, semver tags
- Follow same pattern as existing workflows (redpanda-sink.yml, mqtt-input-ci.yml)

### 4. Add to GitOps

**File**: `gitops/clusters/homelab/apps.yaml`

- Add heatpump-web kustomization entry

## Implementation Steps

1. Create Dockerfile with multi-stage build
2. Create nginx.conf for SPA routing
3. Test Docker build locally
4. Create all Kubernetes manifests (namespace, deployment, service)
5. Create kustomization.yaml for base and overlay
6. Create instructions for ghcr-secret-sealed.yaml (manual step)
7. Create GitHub Actions CI/CD workflow
8. Update apps.yaml to include heatpump-web
9. Test Docker image locally
10. Push and verify CI/CD workflow runs
11. Verify deployment in cluster

## Docker Build Test

```bash
cd applications/heatpump-web
docker build -t heatpump-web:test .
docker run -p 8080:80 -e VITE_API_URL=http://localhost:8081 heatpump-web:test
```

## Nginx Config Notes

Use envsubst for environment variable substitution:

```nginx
location /api {
    proxy_pass http://${VITE_API_URL};
}
```

Or build-time injection of VITE_API_URL into JavaScript bundle.

## Verification

```bash
# Local Docker test
docker build -t heatpump-web:test .
docker run -p 8080:80 heatpump-web:test

# After deployment
kubectl get pods -n heatpump-web
kubectl port-forward svc/heatpump-web 8080:80 -n heatpump-web
curl http://localhost:8080
```

## Dependencies

- Chunk 8: All frontend components must be complete
- Chunk 6: Backend must be deployed (for API URL reference)

## Next Chunk

Chunk 10: Integration and Ingress