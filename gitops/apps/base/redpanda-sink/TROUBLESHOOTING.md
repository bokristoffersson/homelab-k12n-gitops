# Troubleshooting redpanda-sink

## Quick Diagnostic Commands

### 1. Check Pod Status

```bash
# Check if pod is running
kubectl get pods -n redpanda-sink -l app=redpanda-sink

# Check pod status details
kubectl describe pod -n redpanda-sink -l app=redpanda-sink

# Check if pod is ready
kubectl get pods -n redpanda-sink -l app=redpanda-sink -o jsonpath='{.items[0].status.phase}'
echo ""
```

### 2. Check Application Logs

```bash
# View recent logs
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=100

# Follow logs in real-time
kubectl logs -n redpanda-sink -f deployment/redpanda-sink

# Check for errors
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "error\|panic\|failed"

# Check for API server startup
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "listening\|started\|server"
```

### 3. Check Configuration

```bash
# Check if API is enabled in config
kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep -A 5 "api:"

# Check auth configuration
kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep -A 10 "auth:"

# Check if secrets exist
kubectl get secret auth-secret -n redpanda-sink
kubectl get secret timescaledb-secret -n redpanda-sink
```

### 4. Test API Connectivity

```bash
# Test from within the cluster
kubectl run -it --rm test-curl --image=curlimages/curl:latest --restart=Never -n redpanda-sink -- \
  curl -v http://redpanda-sink:8080/health

# Test login endpoint
kubectl run -it --rm test-curl --image=curlimages/curl:latest --restart=Never -n redpanda-sink -- \
  curl -X POST http://redpanda-sink:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"admin","password":"your-password"}'
```

### 5. Check Service and Endpoints

```bash
# Check service exists
kubectl get svc -n redpanda-sink redpanda-sink

# Check endpoints (should show pod IP)
kubectl get endpoints -n redpanda-sink redpanda-sink

# Describe service
kubectl describe svc -n redpanda-sink redpanda-sink
```

### 6. Verify Port Forward

```bash
# Check if port-forward is running
ps aux | grep "kubectl.*port-forward.*8080"

# Kill existing port-forwards
pkill -f "kubectl.*port-forward.*8080"

# Start fresh port-forward
kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080
```

## Common Issues

### Issue: No Response / Empty Reply

**Symptoms:**
- `curl: (52) Empty reply from server`
- Port-forward shows "network namespace closed"

**Causes:**
1. Pod crashed or is restarting
2. Application failed to start
3. Port-forward connection broken

**Solutions:**
```bash
# 1. Check pod status
kubectl get pods -n redpanda-sink -l app=redpanda-sink

# 2. Check if pod is crashing
kubectl describe pod -n redpanda-sink -l app=redpanda-sink | grep -A 10 "Events:"

# 3. Check previous container logs (if pod restarted)
kubectl logs -n redpanda-sink deployment/redpanda-sink --previous

# 4. Restart port-forward
kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080
```

### Issue: 401 Unauthorized on Login

**Symptoms:**
- Login returns 401 even with correct password

**Causes:**
1. Password hash doesn't match
2. Username doesn't exist in config
3. Auth secret not properly loaded

**Solutions:**
```bash
# 1. Verify username in config
kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep username

# 2. Check if auth secret exists and has data
kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d
echo ""

# 3. Check application logs for auth errors
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "auth\|login\|unauthorized"

# 4. Verify password hash format (should start with $2b$)
kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d | head -c 10
echo ""
```

### Issue: Application Not Starting

**Symptoms:**
- Pod in CrashLoopBackOff
- No logs or logs show panic

**Solutions:**
```bash
# 1. Check pod events
kubectl describe pod -n redpanda-sink -l app=redpanda-sink

# 2. Check all logs
kubectl logs -n redpanda-sink deployment/redpanda-sink --previous

# 3. Check if config is valid
kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml

# 4. Check if secrets are properly sealed
kubectl get sealedsecret -n redpanda-sink auth-secret
```

### Issue: API Not Enabled

**Symptoms:**
- No API endpoints responding
- Logs show no API server startup

**Solutions:**
```bash
# 1. Check if API is enabled in config
kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep -A 3 "api:"

# Should show:
# api:
#   enabled: true
#   host: "0.0.0.0"
#   port: 8080
```

## Debugging Login Issues

### Step 1: Verify Configuration

```bash
# Check username in config
kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep -A 2 "username:"

# Should show: username: "admin"
```

### Step 2: Check Password Hash

```bash
# Get the stored password hash
STORED_HASH=$(kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d)
echo "Stored hash: $STORED_HASH"

# Verify it's a valid bcrypt hash (should start with $2b$)
echo "$STORED_HASH" | grep -q '^\$2[ab]\$' && echo "✓ Valid bcrypt hash" || echo "✗ Invalid hash format"
```

### Step 3: Test Password Verification Locally

If you have the application code, you can test password verification:

```bash
cd applications/redpanda-sink

# Create a test script
cat > /tmp/test_password.rs <<EOF
use redpanda_sink::auth::password::verify_password;

fn main() {
    let args: std::vec::Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: test_password <password> <hash>");
        std::process::exit(1);
    }
    
    let password = &args[1];
    let hash = &args[2];
    
    match verify_password(password, hash) {
        Ok(true) => {
            println!("✓ Password matches!");
            std::process::exit(0);
        }
        Ok(false) => {
            println!("✗ Password does not match");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
EOF

# Get the stored hash
STORED_HASH=$(kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d)

# Test (replace with your actual password)
cargo run --bin test_password -- "$STORED_HASH" "your-password"
```

### Step 4: Check Application Logs for Auth Errors

```bash
# Watch logs while attempting login
kubectl logs -n redpanda-sink -f deployment/redpanda-sink &

# In another terminal, try login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'

# Check for specific error messages
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "password\|verify\|auth\|unauthorized"
```

### Step 5: Enable Debug Logging

Temporarily increase log level to see more details:

```bash
# Update deployment to add debug logging
kubectl set env deployment/redpanda-sink -n redpanda-sink RUST_LOG=debug

# Or edit the deployment
kubectl edit deployment redpanda-sink -n redpanda-sink
# Change: RUST_LOG: info
# To: RUST_LOG: debug

# Restart and check logs
kubectl rollout restart deployment/redpanda-sink -n redpanda-sink
kubectl logs -n redpanda-sink -f deployment/redpanda-sink
```

## Quick Diagnostic Script

Save this as `diagnose-redpanda-sink.sh`:

```bash
#!/bin/bash

set -e

NAMESPACE="redpanda-sink"
DEPLOYMENT="redpanda-sink"

echo "=== Pod Status ==="
kubectl get pods -n $NAMESPACE -l app=$DEPLOYMENT

echo -e "\n=== Pod Phase ==="
kubectl get pods -n $NAMESPACE -l app=$DEPLOYMENT -o jsonpath='{.items[0].status.phase}'
echo ""

echo -e "\n=== Recent Logs (last 20 lines) ==="
kubectl logs -n $NAMESPACE deployment/$DEPLOYMENT --tail=20

echo -e "\n=== Errors in Logs ==="
kubectl logs -n $NAMESPACE deployment/$DEPLOYMENT | grep -i "error\|panic\|failed" | tail -5 || echo "No errors found"

echo -e "\n=== API Configuration ==="
kubectl get configmap redpanda-sink-config -n $NAMESPACE -o yaml | grep -A 5 "api:" || echo "API config not found"

echo -e "\n=== Auth Configuration ==="
kubectl get configmap redpanda-sink-config -n $NAMESPACE -o yaml | grep -A 3 "username:" || echo "Auth config not found"

echo -e "\n=== Secrets Status ==="
kubectl get secret auth-secret -n $NAMESPACE && echo "✓ auth-secret exists" || echo "✗ auth-secret missing"
kubectl get secret timescaledb-secret -n $NAMESPACE && echo "✓ timescaledb-secret exists" || echo "✗ timescaledb-secret missing"

echo -e "\n=== Service Status ==="
kubectl get svc -n $NAMESPACE $DEPLOYMENT
kubectl get endpoints -n $NAMESPACE $DEPLOYMENT

echo -e "\n=== Test Health Endpoint (from within cluster) ==="
kubectl run -it --rm test-health-$(date +%s) --image=curlimages/curl:latest --restart=Never -n $NAMESPACE -- \
  curl -s -o /dev/null -w "HTTP Status: %{http_code}\n" http://redpanda-sink:8080/health || echo "Health check failed"
```

Make it executable and run:
```bash
chmod +x diagnose-redpanda-sink.sh
./diagnose-redpanda-sink.sh
```
