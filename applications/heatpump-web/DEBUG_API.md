# Debugging 500 Errors

If you're seeing 500 errors on all dashboard panels, follow these steps:

## 1. Check Backend Status

### Verify Backend Pod is Running

```bash
kubectl get pods -n redpanda-sink
```

Should show a pod with status `Running`. If not:
- Check pod logs: `kubectl logs -n redpanda-sink -l app=redpanda-sink`
- Check pod events: `kubectl describe pod -n redpanda-sink <pod-name>`

### Verify Port-Forward is Active

```bash
# Check if port-forward is running
lsof -i :8080

# If not, start it:
kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080
```

## 2. Test API Directly

### Test with curl (replace TOKEN with your JWT token)

```bash
# Get your token from browser localStorage or login again
TOKEN="your-jwt-token-here"

# Test latest energy endpoint
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/energy/latest

# Test heatpump endpoint
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/heatpump/latest

# Test hourly total
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/energy/hourly-total
```

### Check Response Details

Look at the response body - 500 errors usually include error details:
```json
{
  "error": "Database connection failed",
  "details": "..."
}
```

## 3. Check Browser Network Tab

1. Open DevTools → Network tab
2. Click on a failed request (red, status 500)
3. Check:
   - **Headers**: Is Authorization header present?
   - **Response**: What's the error message?
   - **Request URL**: Is it correct?

## 4. Common 500 Error Causes

### Database Connection Issues

If backend can't connect to database:
```bash
# Check database pod
kubectl get pods -n <database-namespace>

# Check backend logs for database errors
kubectl logs -n redpanda-sink -l app=redpanda-sink | grep -i database
```

### Missing Data

If database is empty or tables don't exist:
- Check if migrations ran
- Check if data ingestion is working
- Verify database schema

### Authentication Issues

Even though login works, token might expire or be invalid:
- Check token expiry in browser localStorage
- Try logging out and back in
- Check backend logs for auth errors

### Backend Application Errors

Check backend logs:
```bash
kubectl logs -n redpanda-sink -l app=redpanda-sink --tail=100
```

Look for:
- Stack traces
- Error messages
- Database connection errors
- Missing environment variables

## 5. Quick Fixes

### Fix: Restart Backend Pod

```bash
kubectl rollout restart deployment/redpanda-sink -n redpanda-sink
```

### Fix: Check Environment Variables

```bash
kubectl get deployment redpanda-sink -n redpanda-sink -o yaml | grep -A 20 env:
```

Verify:
- DATABASE_USER
- DATABASE_PASSWORD
- REDPANDA_BROKERS
- JWT_SECRET

### Fix: Verify Database Connection

```bash
# Check if database service exists
kubectl get svc -n <database-namespace>

# Test connection from backend pod
kubectl exec -n redpanda-sink -it <pod-name> -- <test-command>
```

## 6. Check Frontend Error Handling

The frontend should show error details. Check:
- Browser console for error messages
- Network tab response body
- React Query error states

## 7. Expected API Responses

### Success Response (200)
```json
{
  "ts": "2024-01-01T12:00:00Z",
  "consumption_total_w": 1500,
  ...
}
```

### Error Response (500)
```json
{
  "error": "Internal server error",
  "message": "Database query failed: ..."
}
```

## 8. Next Steps

1. **Check backend logs** - Most important step
2. **Test API with curl** - Verify backend is responding
3. **Check database** - Verify data exists and is accessible
4. **Check authentication** - Verify JWT token is valid
5. **Check environment** - Verify all required env vars are set

Share the backend logs and API response details for more specific help!
