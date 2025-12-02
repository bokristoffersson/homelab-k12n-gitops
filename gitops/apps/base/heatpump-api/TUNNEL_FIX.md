# Cloudflare Tunnel Troubleshooting - API Not Accessible

## Problem Identified

The API at `api.k12n.com` is returning **403 Forbidden** from Cloudflare Access. The tunnel is working correctly, but authentication is blocking requests.

### Current Status

✅ **Tunnel**: Connected and registered with Cloudflare  
✅ **Service**: Running and healthy (`heatpump-api.heatpump-api.svc.cluster.local:3000`)  
✅ **DNS**: Resolving correctly to Cloudflare IPs (188.114.96.1, 188.114.97.1)  
❌ **Access**: Cloudflare Access is blocking requests (403 Forbidden)

### Error Response

When accessing `https://api.k12n.com/health` without authentication:
```json
{
  "message": "Forbidden. You don't have permission to view this. Please contact your system administrator.",
  "status_code": 403,
  "aud": "d6bad9f795af4ee44b6f8d31d0dfecf4b3d171f87deec976c6a4fdb5979fcbf8",
  "ray_id": "9a58038ffebd2687"
}
```

## Hostname Mismatch Issue

There's a **hostname mismatch** between configuration and documentation:

- **Tunnel Config**: `api.k12n.com` ✅ (currently active)
- **Documentation**: `api.heatpump.k12n.com` ❌ (not configured)

**Decision needed**: Choose one hostname and update both the tunnel config and Cloudflare Access application.

## Solutions

### Option 1: Use `api.k12n.com` (Current Configuration)

If you want to keep `api.k12n.com`:

1. **Verify Cloudflare Access Application**:
   - Go to [Cloudflare Zero Trust Dashboard](https://one.dash.cloudflare.com/)
   - Navigate to **Access** → **Applications**
   - Find the application for `api.k12n.com` (or create one)
   - Ensure the application domain matches: `api.k12n.com`

2. **Configure Access Policy**:
   - Add a policy that allows your requests
   - For testing, you can temporarily allow all requests
   - For production, use Service Tokens or email-based auth

3. **Test with Authentication**:
   ```bash
   # With Service Token
   curl -H "CF-Access-Token: <your-service-token>" \
        https://api.k12n.com/health
   
   # Should return: {"status":"ok"}
   ```

### Option 2: Switch to `api.heatpump.k12n.com`

If you prefer `api.heatpump.k12n.com`:

1. **Update Tunnel Config**:
   ```bash
   kubectl edit configmap cloudflared-config -n cloudflare-tunnel
   ```
   
   Change:
   ```yaml
   - hostname: api.k12n.com
     service: http://heatpump-api.heatpump-api.svc.cluster.local:3000
   ```
   
   To:
   ```yaml
   - hostname: api.heatpump.k12n.com
     service: http://heatpump-api.heatpump-api.svc.cluster.local:3000
   ```

2. **Update DNS**:
   - In Cloudflare DNS, ensure `api.heatpump.k12n.com` CNAME points to your tunnel
   - Or create an A record pointing to Cloudflare IPs

3. **Update Cloudflare Access**:
   - Update the Access application domain to `api.heatpump.k12n.com`
   - Or create a new application for this hostname

4. **Restart Tunnel Pods** (to pick up config change):
   ```bash
   kubectl rollout restart deployment cloudflared -n cloudflare-tunnel
   ```

## Quick Fix: Temporarily Disable Access (Testing Only)

⚠️ **WARNING**: Only for testing! Do not use in production.

1. Go to Cloudflare Zero Trust Dashboard
2. Navigate to **Access** → **Applications**
3. Find the application for `api.k12n.com`
4. Temporarily disable the application or remove all policies
5. Test: `curl https://api.k12n.com/health`

**Remember to re-enable Access after testing!**

## Verify Tunnel Connectivity

The tunnel connection errors in logs are normal - they're connection retries. The tunnel is working. Verify with:

```bash
# Check tunnel pods
kubectl get pods -n cloudflare-tunnel

# Check tunnel logs (should show "Registered tunnel connection")
kubectl logs -n cloudflare-tunnel cloudflared-fd5d66bf8-246bt | grep "Registered tunnel"

# Check service connectivity from tunnel pod
kubectl exec -n cloudflare-tunnel cloudflared-fd5d66bf8-246bt -- \
  sh -c 'echo "GET /health HTTP/1.1\r\nHost: heatpump-api.heatpump-api.svc.cluster.local\r\n\r\n" | nc heatpump-api.heatpump-api.svc.cluster.local 3000'
```

## Recommended Setup for Production

1. **Use Service Tokens** for API access (mobile apps, scripts)
2. **Use Email Auth** for web browser access
3. **Keep Access enabled** for security
4. **Standardize on one hostname** (`api.k12n.com` or `api.heatpump.k12n.com`)

## Next Steps

1. ✅ Decide on hostname (`api.k12n.com` or `api.heatpump.k12n.com`)
2. ✅ Configure Cloudflare Access application for chosen hostname
3. ✅ Create Service Token for API access
4. ✅ Test with: `curl -H "CF-Access-Token: <token>" https://api.k12n.com/health`
5. ✅ Update documentation to match chosen hostname

## Additional Notes

- The QUIC connection timeouts in tunnel logs are normal - Cloudflare automatically retries
- The tunnel is successfully registered and routing requests
- The 403 error confirms the tunnel is working - it's just Access blocking unauthenticated requests
- DNS is correctly pointing to Cloudflare (188.114.96.1, 188.114.97.1)

