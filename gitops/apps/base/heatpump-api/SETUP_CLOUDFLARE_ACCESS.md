# Automated Cloudflare Access Setup

This guide explains how to use the automated Job to set up Cloudflare Access for the heatpump-api application.

## Overview

The `setup-cloudflare-access` Job automatically:
1. Creates or updates a Cloudflare Access Application for `api.k12n.com`
2. Creates a Service Token for API authentication
3. Creates an Access Policy that allows the Service Token
4. Outputs the Service Token value for use in your applications

## Prerequisites

1. **Cloudflare API Token** with required permissions (see [CLOUDFLARE_API_TOKEN_PERMISSIONS.md](./CLOUDFLARE_API_TOKEN_PERMISSIONS.md))
2. **Cloudflare Account ID**
3. **kubeseal** installed for creating sealed secrets
4. **kubectl** access to your cluster

## Step-by-Step Setup

### 1. Create Cloudflare API Token

Follow the instructions in [CLOUDFLARE_API_TOKEN_PERMISSIONS.md](./CLOUDFLARE_API_TOKEN_PERMISSIONS.md) to create an API token with the required permissions.

**Required Permissions:**
- Account - Cloudflare Access:Apps - Edit
- Account - Cloudflare Access:Service Tokens - Edit
- Account - Cloudflare Access:Policies - Edit
- Zone - Zone:Zone - Read (for zone lookup)

### 2. Get Your Account ID

Find your Cloudflare Account ID:
- From dashboard URL: `https://dash.cloudflare.com/<account-id>/...`
- Or from account settings in the dashboard

### 3. Create Sealed Secret

The secret is stored in the `cloudflare-tunnel` namespace (infrastructure-level). Create the sealed secret:

```bash
kubectl create secret generic cloudflare-api-token \
  --from-literal=api-token='<your-api-token>' \
  --from-literal=account-id='<your-account-id>' \
  --namespace=cloudflare-tunnel \
  --dry-run=client -o yaml | \
  kubeseal -o yaml > ../../infrastructure/controllers/cloudflare-tunnel/cloudflare-api-token-secret-sealed.yaml
```

**Important**: 
- The secret is stored in `cloudflare-tunnel` namespace (infrastructure)
- The setup job in `heatpump-api` namespace references it cross-namespace
- Replace the placeholder values in the sealed secret file with the actual sealed secret output

### 4. Add to Kustomization (Optional)

**For the secret** (in cloudflare-tunnel infrastructure):
The secret is already included in `gitops/infrastructure/controllers/cloudflare-tunnel/kustomization.yaml`

**For the job** (in heatpump-api application):
If you want the job to be part of your GitOps deployment, add to `gitops/apps/base/heatpump-api/kustomization.yaml`:

```yaml
resources:
  - setup-cloudflare-access-job.yaml
```

**Note**: The job will run once when applied. To run it again, delete the job first:
```bash
kubectl delete job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel
```

### 5. Run the Job

#### Option A: Apply via GitOps (if added to kustomization)
```bash
# Commit and push the changes
# Flux will apply them automatically
```

#### Option B: Run Manually
```bash
# Apply the secret first (in cloudflare-tunnel namespace)
kubectl apply -f ../../infrastructure/controllers/cloudflare-tunnel/cloudflare-api-token-secret-sealed.yaml

# Apply the job (in heatpump-api namespace)
kubectl apply -f setup-cloudflare-access-job.yaml

# Watch the job
kubectl get job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel -w

# Check logs
kubectl logs -n cloudflare-tunnel job/setup-cloudflare-access-heatpump-api -f
```

### 6. Get the Service Token

After the job completes successfully, check the logs for the Service Token:

```bash
kubectl logs -n cloudflare-tunnel job/setup-cloudflare-access-heatpump-api | grep "Token Value"
```

**Important**: Save this token securely! You'll need it for API authentication.

The output will look like:
```
=== IMPORTANT: Save this Service Token ===
Token Value: <long-token-string>
Use this in your API requests: curl -H "CF-Access-Token: <long-token-string>" https://api.k12n.com/health
==========================================
```

## Configuration

The job uses the following environment variables (configurable in the job YAML):

- `CLOUDFLARE_API_TOKEN`: From secret `cloudflare-api-token`, key `api-token`
- `CLOUDFLARE_ACCOUNT_ID`: From secret `cloudflare-api-token`, key `account-id`
- `CLOUDFLARE_ZONE_NAME`: Default `k12n.com` (for zone lookup)
- `ACCESS_APP_DOMAIN`: Default `api.k12n.com`
- `ACCESS_APP_NAME`: Default `heatpump-api`

## What the Job Does

1. **Gets Zone ID**: Looks up the Zone ID for `k12n.com`
2. **Checks/Creates Access Application**: 
   - Checks if an Access Application for `api.k12n.com` exists
   - Creates it if it doesn't exist
   - Uses existing one if found
3. **Creates Service Token**:
   - Creates a Service Token named `heatpump-api-service-token`
   - Outputs the token value (only shown once!)
4. **Creates Access Policy**:
   - Creates a policy that allows the Service Token
   - Policy name: `heatpump-api-service-token-policy`

## Idempotency

The job is idempotent:
- Won't create duplicate Access Applications
- Won't create duplicate Service Tokens
- Won't create duplicate Policies
- Safe to run multiple times

**Note**: If a Service Token already exists, the job won't show the token value again. You'll need to get it from the Cloudflare dashboard or regenerate it.

## Troubleshooting

### Job Fails with "Insufficient permissions"

1. Verify API token has all required permissions
2. Check token hasn't expired
3. Verify Account ID is correct

### Job Fails with "Zone not found"

1. Verify `CLOUDFLARE_ZONE_NAME` is correct
2. Check API token has zone read permission
3. Or hardcode Zone ID in the job script

### Service Token Not Shown

If the Service Token already exists, the job won't display it. To get it:
1. Go to Cloudflare Zero Trust Dashboard
2. Access → Applications → `heatpump-api`
3. Service Tokens tab
4. Find `heatpump-api-service-token`
5. Click to view (if available) or regenerate

### Job Stuck in "Running"

Check the pod logs:
```bash
kubectl logs -n cloudflare-tunnel job/setup-cloudflare-access-heatpump-api
```

If there are errors, fix them and delete/recreate the job:
```bash
kubectl delete job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel
kubectl apply -f setup-cloudflare-access-job.yaml
```

## Testing the Setup

After the job completes, test the API with the Service Token:

```bash
# Replace <token> with the Service Token from job logs
curl -H "CF-Access-Token: <token>" \
     https://api.k12n.com/health
```

Expected response:
```json
{"status":"ok"}
```

## Cleanup

To remove the Access Application and Service Token:

1. **Via Cloudflare Dashboard**:
   - Go to Cloudflare Zero Trust Dashboard
   - Access → Applications → `heatpump-api`
   - Delete the application (this also removes associated tokens and policies)

2. **Via API** (if needed):
   ```bash
   # Get Application ID from job logs or dashboard
   # Then use Cloudflare API to delete
   ```

## Security Notes

- **API Token**: Store securely, rotate periodically
- **Service Token**: Long-lived, store securely in your application
- **Sealed Secrets**: Never commit unencrypted secrets
- **Token Value**: Only displayed once in job logs - save it immediately!

## Next Steps

After successful setup:
1. ✅ Save the Service Token securely
2. ✅ Test API access with the token
3. ✅ Configure your application to use the token
4. ✅ Monitor Access logs in Cloudflare dashboard
5. ✅ Set up token rotation schedule

