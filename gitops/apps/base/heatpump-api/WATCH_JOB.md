# Watching the Cloudflare Access Setup Job

Quick reference for monitoring the `setup-cloudflare-access-heatpump-api` job.

## Quick Commands

### Watch Job Status

```bash
# Watch job status (updates every 2 seconds)
kubectl get job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel -w

# Or watch with more details
kubectl get job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel -o wide -w
```

**Expected output:**
```
NAME                                  COMPLETIONS   DURATION   AGE
setup-cloudflare-access-heatpump-api  0/1           5s         5s
setup-cloudflare-access-heatpump-api  1/1           12s        12s
```

### Watch Pod Status

```bash
# Get the pod name first
kubectl get pods -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api

# Watch pod status
kubectl get pods -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api -w
```

**Expected states:**
- `Pending` → `Running` → `Completed` (success)
- `Pending` → `Running` → `Error` (failure)

### Watch Logs in Real-Time

```bash
# Follow logs (like tail -f)
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api -f

# Or if you know the pod name
kubectl logs -n cloudflare-tunnel <pod-name> -f
```

### Get Current Status

```bash
# Check job status
kubectl get job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel

# Check pod status
kubectl get pods -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api

# Get recent logs (last 50 lines)
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api --tail=50
```

## One-Liner to Watch Everything

```bash
# Watch job and logs simultaneously (in separate terminals)
# Terminal 1:
kubectl get job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel -w

# Terminal 2:
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api -f
```

## Check for Service Token Output

After the job completes, extract the Service Token:

```bash
# Get the full logs and grep for the token
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api | grep -A 3 "Token Value"

# Or get everything between the markers
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api | grep -A 5 "IMPORTANT"
```

## Troubleshooting

### Job Stuck in Running

```bash
# Check pod events
kubectl describe pod -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api

# Check for errors in logs
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api | grep -i error
```

### Job Failed

```bash
# Get failure reason
kubectl describe job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel

# Get pod logs
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api

# Check pod events
kubectl describe pod -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api
```

### Restart the Job

```bash
# Delete the job (this also deletes completed pods)
kubectl delete job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel

# Re-apply (if using GitOps, just commit/push)
kubectl apply -f gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml
```

## Expected Output

When successful, you should see:

```
=== Cloudflare Access Setup Script ===
Setting up Access Application and Service Tokens for api.k12n.com
Getting Zone ID for k12n.com...
Zone ID: <zone-id>
Checking for existing Access Application...
Creating new Access Application...
Created Access Application with ID: <app-id>
Checking for existing Service Tokens...
Creating Service Token...
Created Service Token with ID: <token-id>

=== IMPORTANT: Save this Service Token ===
Token Value: <long-token-string>
Use this in your API requests: curl -H "CF-Access-Token: <token>" https://api.k12n.com/health
==========================================
Creating Access Policy for Service Token...
Created Access Policy with ID: <policy-id>

=== Setup Complete ===
Application: heatpump-api
Domain: api.k12n.com
Application ID: <app-id>
```

## Quick Status Check

```bash
# All-in-one status check
echo "=== Job Status ===" && \
kubectl get job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel && \
echo -e "\n=== Pod Status ===" && \
kubectl get pods -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api && \
echo -e "\n=== Recent Logs ===" && \
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api --tail=10
```

