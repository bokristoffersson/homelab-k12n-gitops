# Complete Automated Cloudflare Access Setup Guide

## Yes, Everything is Automated! ✅

You can use Kubernetes Jobs with your Cloudflare API token to **fully automate** the setup of:
1. ✅ Cloudflare Access Application (protects your API)
2. ✅ Service Token (for mobile app authentication)
3. ✅ Access Policy (allows Service Token access)
4. ✅ Cleanup of duplicates and conflicts

**No manual dashboard configuration needed** (except initial API token setup).

## What You Already Have

### 1. Cloudflare API Token Secret
**Location**: `gitops/infrastructure/controllers/cloudflare-tunnel/cloudflare-api-token-secret-sealed.yaml`

- Contains your Cloudflare API token
- Contains your Account ID
- Stored in `cloudflare-tunnel` namespace
- Already configured and working

### 2. Setup Job (Idempotent)
**File**: `gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml`

**What it does**:
- Creates/updates Access Application for `api.k12n.com`
- Creates Service Token if it doesn't exist
- Creates Access Policy linking Service Token to Application
- Outputs Service Token value for your mobile app
- Safe to run multiple times

**Usage**:
```bash
kubectl apply -f gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api -f
```

### 3. Cleanup Job (Fresh Start)
**File**: `gitops/apps/base/heatpump-api/cleanup-and-fix-access-job.yaml`

**What it does**:
- Deletes duplicate Service Tokens
- Deletes old Access Policies
- Creates fresh Service Token
- Creates fresh Access Policy
- Outputs new Service Token value

**Usage**:
```bash
kubectl apply -f gitops/apps/base/heatpump-api/cleanup-and-fix-access-job.yaml
kubectl logs -n cloudflare-tunnel -l job-name=cleanup-cloudflare-access-heatpump-api -f
```

## Complete Setup Flow for Mobile App

### Step 1: Run the Setup Job

```bash
# Apply the job
kubectl apply -f gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml

# Watch it complete
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api -f
```

### Step 2: Get Your Service Token

```bash
# Extract the Service Token from logs
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api | grep -A 3 "Token Value"
```

You'll see:
```
=== IMPORTANT: Save this Service Token ===
Token Value: <your-token-here>
Use this in your API requests: curl -H "CF-Access-Token: <token>" https://api.k12n.com/health
==========================================
```

### Step 3: Use in Your Mobile App

**Swift (iOS)**:
```swift
let serviceToken = "your-token-value-from-job-output"
let url = URL(string: "https://api.k12n.com/api/v1/heatpump")!
var request = URLRequest(url: url)
request.setValue(serviceToken, forHTTPHeaderField: "CF-Access-Token")

URLSession.shared.dataTask(with: request) { data, response, error in
    // Handle response
}.resume()
```

**Kotlin (Android)**:
```kotlin
val serviceToken = "your-token-value-from-job-output"
val client = OkHttpClient()
val request = Request.Builder()
    .url("https://api.k12n.com/api/v1/heatpump")
    .addHeader("CF-Access-Token", serviceToken)
    .build()

client.newCall(request).enqueue(object : Callback {
    override fun onResponse(call: Call, response: Response) {
        // Handle response
    }
    override fun onFailure(call: Call, e: IOException) {
        // Handle error
    }
})
```

## What Gets Configured Automatically

### Access Application
- **Domain**: `api.k12n.com`
- **Type**: Self-hosted
- **Session Duration**: 24 hours
- **Auto-redirect**: Disabled (for API)
- **Status**: Active and protected

### Service Token
- **Name**: `heatpump-api-service-token`
- **Duration**: 1 year (8760 hours)
- **Purpose**: Mobile app authentication
- **Format**: Long hexadecimal string
- **Usage**: Single header `CF-Access-Token: <token>`

### Access Policy
- **Name**: `heatpump-api-service-token-policy`
- **Action**: Allow
- **Includes**: Service Token (the one created)
- **Excludes**: None
- **Requires**: None

## Configuration Options

You can customize the setup by modifying environment variables in the job:

```yaml
env:
  - name: ACCESS_APP_DOMAIN
    value: "api.k12n.com"  # Your API domain
  - name: ACCESS_APP_NAME
    value: "heatpump-api"   # Application name
```

## For New API Endpoints

To set up a different endpoint (e.g., `api2.k12n.com`):

1. **Copy the job file**:
   ```bash
   cp gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml \
      gitops/apps/base/heatpump-api/setup-cloudflare-access-api2-job.yaml
   ```

2. **Update the job**:
   - Change `name` to `setup-cloudflare-access-api2`
   - Change `ACCESS_APP_DOMAIN` to `api2.k12n.com`
   - Change `ACCESS_APP_NAME` to `api2`

3. **Run it**:
   ```bash
   kubectl apply -f gitops/apps/base/heatpump-api/setup-cloudflare-access-api2-job.yaml
   ```

## Security Best Practices

### Service Token Storage
- ✅ Store in secure storage (iOS Keychain, Android Keystore)
- ✅ Never commit to version control
- ✅ Use environment variables in CI/CD
- ✅ Rotate every 6-12 months

### Token Rotation

To rotate the token:
```bash
# Delete old job
kubectl delete job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel

# Run cleanup job to remove old token
kubectl apply -f gitops/apps/base/heatpump-api/cleanup-and-fix-access-job.yaml

# Or just re-run setup (it will create new if old is deleted)
kubectl apply -f gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml
```

## Monitoring

### Check Access Logs
1. Go to Cloudflare Zero Trust Dashboard
2. Access → Applications → `heatpump-api`
3. View access logs to see:
   - Successful authentications
   - Failed attempts
   - Token usage

### Test from Command Line

```bash
# Without token (should fail)
curl https://api.k12n.com/health
# Expected: 403 Forbidden

# With token (should succeed)
curl -H "CF-Access-Token: <your-token>" https://api.k12n.com/health
# Expected: {"status":"ok"}
```

## Troubleshooting

### Job Fails
```bash
# Check logs
kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api

# Check job status
kubectl get job setup-cloudflare-access-heatpump-api -n cloudflare-tunnel
```

### Token Not Working
1. Verify token value matches job output exactly
2. Check policy includes the Service Token
3. Verify Access Application domain is correct
4. Check Cloudflare Access logs in dashboard

### Need Fresh Start
```bash
# Run cleanup job
kubectl apply -f gitops/apps/base/heatpump-api/cleanup-and-fix-access-job.yaml

# Then run setup
kubectl apply -f gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml
```

## Summary

✅ **Fully Automated**: Everything can be done via Kubernetes Jobs  
✅ **No Manual Steps**: Except initial API token creation  
✅ **Idempotent**: Safe to run multiple times  
✅ **Mobile-Ready**: Service Token perfect for mobile apps  
✅ **Secure**: Protected by Cloudflare Access  
✅ **Production-Ready**: Handles errors and edge cases  

## Quick Start for Mobile App

1. **Run setup job**:
   ```bash
   kubectl apply -f gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml
   ```

2. **Get Service Token**:
   ```bash
   kubectl logs -n cloudflare-tunnel -l job-name=setup-cloudflare-access-heatpump-api | grep -A 1 "Token Value"
   ```

3. **Use in mobile app**:
   - Add `CF-Access-Token` header to all API requests
   - Store token securely (Keychain/Keystore)
   - API endpoint: `https://api.k12n.com`

That's it! Your API is now protected and ready for your mobile app.

