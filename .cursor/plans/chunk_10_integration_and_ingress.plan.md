````javascript
---
name: Chunk 10 Integration and Ingress
overview: Configure Cloudflare Tunnel ingress for public access to the frontend application
todos:
    - id: review-tunnel-config
    content: Review existing Cloudflare Tunnel deployment configuration
    status: pending
    - id: add-ingress-rule
    content: Add heatpump.k12n.com ingress rule to Cloudflare Tunnel config
    status: pending
    - id: verify-service-ref
    content: Verify service reference points to correct namespace and port
    status: pending
    - id: verify-sync
    content: Verify GitOps syncs the configuration change
    status: pending
    - id: test-dns
    content: Test DNS resolution for heatpump.k12n.com
    status: pending
    - id: test-https
    content: Test HTTPS access to frontend
    status: pending
    - id: test-full-flow
    content: Test complete application flow: login, dashboard, data display
    status: pending
---

# Chunk 10: Integration and Ingress

## Overview

Complete the integration by adding Cloudflare Tunnel ingress rule to expose the frontend application publicly at heatpump.k12n.com. This is the final step to make the application accessible.

## Files to Modify

### 1. Cloudflare Tunnel Configuration

**File**: `gitops/infrastructure/controllers/cloudflare-tunnel/deployment.yaml`

- Add ingress rule: `heatpump.k12n.com -> heatpump-web.heatpump-web.svc.cluster.local:80`
- Follow existing pattern for other ingress rules
- Ensure proper namespace and service reference

## Implementation Steps

1. Review existing Cloudflare Tunnel configuration
2. Add new ingress rule for heatpump.k12n.com
3. Verify service reference (heatpump-web.heatpump-web.svc.cluster.local:80)
4. Commit and push changes
5. Verify GitOps syncs the configuration
6. Verify Cloudflare Tunnel picks up the new route
7. Test public access at https://heatpump.k12n.com
8. Verify frontend can reach backend API (CORS if needed)

## Cloudflare Tunnel Ingress Format

The ingress rule should follow this pattern:

```yaml
ingress:
    - hostname: heatpump.k12n.com
    service: http://heatpump-web.heatpump-web.svc.cluster.local:80
```

Or add to existing ingress array in the deployment.

## Verification Steps

1. Check Cloudflare Tunnel logs for route registration:

```bash
kubectl logs -n cloudflare-tunnel -l app=cloudflare-tunnel
```

2. Test DNS resolution:

```bash
dig heatpump.k12n.com
```

3. Test HTTPS access:

```bash
curl -I https://heatpump.k12n.com
```

4. Test in browser:

- Navigate to https://heatpump.k12n.com
- Verify login page loads
- Test login flow
- Verify dashboard displays correctly

## CORS Configuration

If frontend and backend are on different domains, ensure backend CORS is configured to allow frontend domain. This should already be handled in Chunk 5 with tower-http CORS middleware.

## Dependencies

- Chunk 9: Frontend must be deployed and accessible in cluster
- Chunk 6: Backend API must be deployed
- Cloudflare Tunnel must be running in cluster

## Completion

After this chunk, the complete Phase 1 MVP is deployed and accessible:

- Backend API running and protected
- Frontend application running and accessible
- Database aggregates providing data
- Public access via Cloudflare Tunnel
- Full authentication flow working

## Post-Deployment Checklist

- [ ] Frontend accessible at https://heatpump.k12n.com
- [ ] Login flow works correctly
- [ ] Dashboard displays all data correctly
- [ ] All API endpoints accessible from frontend
- [ ] Authentication working (JWT tokens)
- [ ] No CORS errors in browser console
- [ ] All components render correctly
- [ ] Charts display data
- [ ] Real-time data updates working (if polling implemented)


````