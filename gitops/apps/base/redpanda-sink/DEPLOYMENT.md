# Deploying Latest Docker Image

This guide explains how to deploy the latest Docker image built by GitHub Actions and verify it's working.

## Prerequisites

- `kubectl` configured to access your cluster
- Access to the `redpanda-sink` namespace
- FluxCD installed (for GitOps automation)

## Deployment Methods

### Method 1: Automatic (FluxCD GitOps)

Since your deployment uses `imagePullPolicy: Always`, FluxCD will automatically pull the latest image when it reconciles. However, you may need to trigger a rollout to force the pod to restart.

**Option A: Restart the deployment (Recommended)**

```bash
# Restart the deployment to pull the latest image
kubectl rollout restart deployment/redpanda-sink -n redpanda-sink

# Watch the rollout status
kubectl rollout status deployment/redpanda-sink -n redpanda-sink
```

**Option B: Force Flux reconciliation**

```bash
# Reconcile the Flux kustomization
flux reconcile kustomization redpanda-sink -n flux-system

# Or trigger via annotation
kubectl annotate kustomization redpanda-sink -n flux-system \
  reconciler.fluxcd.io/requestedAt="$(date +%s)" --overwrite
```

### Method 2: Manual Image Update

If you want to deploy a specific image tag (not just `latest`):

```bash
# Update the deployment to use a specific tag
kubectl set image deployment/redpanda-sink \
  app=ghcr.io/bokristoffersson/redpanda-sink:main \
  -n redpanda-sink

# Or edit the deployment directly
kubectl edit deployment redpanda-sink -n redpanda-sink
# Change: image: ghcr.io/bokristoffersson/redpanda-sink:latest
# To: image: ghcr.io/bokristoffersson/redpanda-sink:main (or specific tag)
```

### Method 3: Update via Git (GitOps)

If you want to change the image tag in Git:

1. Edit `gitops/apps/base/redpanda-sink/deployment.yaml`
2. Update the image tag if needed (currently uses `:latest`)
3. Commit and push:
   ```bash
   git add gitops/apps/base/redpanda-sink/deployment.yaml
   git commit -m "Update redpanda-sink image"
   git push
   ```
4. FluxCD will automatically apply the changes

## Verification Steps

### Step 1: Check Deployment Status

```bash
# Check deployment status
kubectl get deployment redpanda-sink -n redpanda-sink

# Check pod status
kubectl get pods -n redpanda-sink -l app=redpanda-sink

# Check the image being used
kubectl get deployment redpanda-sink -n redpanda-sink -o jsonpath='{.spec.template.spec.containers[0].image}'
echo ""

# Check pod image
kubectl get pods -n redpanda-sink -l app=redpanda-sink -o jsonpath='{.items[0].spec.containers[0].image}'
echo ""
```

### Step 2: Check Application Logs

```bash
# View recent logs
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=50

# Follow logs in real-time
kubectl logs -n redpanda-sink -f deployment/redpanda-sink

# Check for successful startup
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "started\|ready\|listening"
```

### Step 3: Verify API Endpoints

The application exposes an API on port 8080. Test the endpoints:

```bash
# Port-forward to access the API locally
kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080

# In another terminal, test the health endpoint
curl http://localhost:8080/health

# Test login endpoint
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-admin-password"}'

# Test protected endpoint (requires token from login)
TOKEN="your-jwt-token-here"
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/energy/latest
```

### Step 4: Verify Data Processing

```bash
# Check if messages are being processed
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "batch\|processed\|flushed"

# Check database for new data (see VERIFICATION.md for detailed queries)
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Check recent data
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT COUNT(*) as total_rows, MAX(ts) as latest 
FROM heatpump 
WHERE ts > NOW() - INTERVAL '10 minutes';
"
```

### Step 5: Check Consumer Group Status

```bash
# Check Redpanda consumer group
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk group describe redpanda-sink --brokers localhost:9092

# Check topic offsets
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic describe heatpump-telemetry --brokers localhost:9092
```

## Quick Verification Script

Save this as `deploy-and-verify.sh`:

```bash
#!/bin/bash

set -e

NAMESPACE="redpanda-sink"
DEPLOYMENT="redpanda-sink"

echo "=== Deploying Latest Image ==="
kubectl rollout restart deployment/$DEPLOYMENT -n $NAMESPACE

echo -e "\n=== Waiting for Rollout ==="
kubectl rollout status deployment/$DEPLOYMENT -n $NAMESPACE --timeout=5m

echo -e "\n=== Checking Pod Status ==="
kubectl get pods -n $NAMESPACE -l app=$DEPLOYMENT

echo -e "\n=== Checking Image Version ==="
kubectl get pods -n $NAMESPACE -l app=$DEPLOYMENT -o jsonpath='{.items[0].spec.containers[0].image}'
echo ""

echo -e "\n=== Recent Logs ==="
kubectl logs -n $NAMESPACE deployment/$DEPLOYMENT --tail=20

echo -e "\n=== Testing Health Endpoint ==="
kubectl run -it --rm test-curl --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -s http://redpanda-sink:8080/health || echo "Health check failed"

echo -e "\n=== Deployment Complete ==="
```

Make it executable and run:
```bash
chmod +x deploy-and-verify.sh
./deploy-and-verify.sh
```

## Troubleshooting

### Image Not Updating

1. **Check image pull policy:**
   ```bash
   kubectl get deployment redpanda-sink -n redpanda-sink -o jsonpath='{.spec.template.spec.containers[0].imagePullPolicy}'
   ```
   Should be `Always` (which it is in your deployment).

2. **Force image pull:**
   ```bash
   kubectl delete pod -n redpanda-sink -l app=redpanda-sink
   # Deployment will automatically create a new pod
   ```

3. **Check image exists in registry:**
   ```bash
   # Verify the image exists (requires Docker login to GHCR)
   docker pull ghcr.io/bokristoffersson/redpanda-sink:latest
   ```

### Pod Not Starting

1. **Check pod events:**
   ```bash
   kubectl describe pod -n redpanda-sink -l app=redpanda-sink
   ```

2. **Check image pull secrets:**
   ```bash
   kubectl get deployment redpanda-sink -n redpanda-sink -o jsonpath='{.spec.template.spec.imagePullSecrets}'
   ```

3. **Check logs:**
   ```bash
   kubectl logs -n redpanda-sink -l app=redpanda-sink --previous
   ```

### API Not Responding

1. **Check service:**
   ```bash
   kubectl get svc redpanda-sink -n redpanda-sink
   ```

2. **Check port-forward:**
   ```bash
   kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080
   ```

3. **Test from within cluster:**
   ```bash
   kubectl run -it --rm test-curl --image=curlimages/curl:latest --restart=Never -n redpanda-sink -- \
     curl -v http://redpanda-sink:8080/health
   ```

## Additional Resources

- See `VERIFICATION.md` for detailed data verification steps
- See `SETUP.md` for initial setup and secret configuration
- Check GitHub Actions workflow for build status and image tags
