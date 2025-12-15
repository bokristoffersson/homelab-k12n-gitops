# Improved Error Logging for redpanda-sink API

## Changes Made

### 1. Enhanced Logging Configuration

Updated `deployment.yaml` to include:
- **RUST_LOG**: Set to `debug` level with specific module logging
  - `redpanda_sink=debug` - All application logs
  - `redpanda_sink::api=debug` - API-specific logs
  - `redpanda_sink::database=debug` - Database query logs
- **RUST_BACKTRACE**: Set to `full` to show complete stack traces on panics

### 2. API Configuration

Added `detailed_errors: true` to the API config in `configmap.yaml` to ensure error details are returned in API responses.

## How to Apply Changes

```bash
# Apply the updated deployment
kubectl apply -f gitops/apps/base/redpanda-sink/deployment.yaml
kubectl apply -f gitops/apps/base/redpanda-sink/configmap.yaml

# Or if using Flux, commit and push - Flux will sync automatically
git add gitops/apps/base/redpanda-sink/
git commit -m "Improve error logging and debugging for API"
git push
```

## Restart Deployment

After applying changes, restart the deployment to pick up new environment variables:

```bash
kubectl rollout restart deployment/redpanda-sink -n redpanda-sink

# Wait for rollout to complete
kubectl rollout status deployment/redpanda-sink -n redpanda-sink
```

## Viewing Enhanced Logs

### Real-time Log Monitoring

```bash
# Follow logs in real-time
kubectl logs -n redpanda-sink -f deployment/redpanda-sink

# In another terminal, make an API request to see logs
curl -H "Authorization: Bearer $TOKEN" https://api.k12n.com/api/v1/energy/latest
```

### Filter Logs by Type

```bash
# API-related logs
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "api\|endpoint\|request"

# Database-related logs
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "database\|query\|sql\|connection"

# Error logs
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "error\|panic\|failed\|500"

# All debug logs
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "debug"
```

## What You'll See Now

With the enhanced logging, you should see:

1. **Request Logging**: Every API request with method, path, and status
2. **Database Query Logs**: SQL queries being executed
3. **Error Details**: Full error messages and stack traces
4. **Response Details**: Response status codes and error messages

## Example Log Output

With debug logging enabled, you should see logs like:

```
[DEBUG] redpanda_sink::api: Handling GET /api/v1/energy/latest
[DEBUG] redpanda_sink::database: Executing query: SELECT ...
[ERROR] redpanda_sink::api: Database query failed: relation "energy" does not exist
[DEBUG] redpanda_sink::api: Returning 500 error: Internal server error
```

## Troubleshooting 500 Errors

Now when you get a 500 error:

1. **Check logs immediately**:
   ```bash
   kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=50
   ```

2. **Look for error messages** - They should now be much more detailed

3. **Check API response body** - With `detailed_errors: true`, the response should include error details:
   ```bash
   curl -H "Authorization: Bearer $TOKEN" https://api.k12n.com/api/v1/energy/latest
   # Should now return JSON with error details instead of empty response
   ```

## Production Considerations

For production, you may want to:
- Set `RUST_LOG` back to `info` to reduce log volume
- Set `detailed_errors: false` to hide internal error details from clients
- Keep `RUST_BACKTRACE=full` only in development

## Next Steps

1. Apply the changes
2. Restart the deployment
3. Make an API request
4. Check the logs to see the detailed error messages
5. Fix the underlying issue (likely missing database tables/views)
