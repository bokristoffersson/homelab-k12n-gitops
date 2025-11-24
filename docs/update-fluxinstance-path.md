# How to Update FluxInstance Path

## Quick Update Command

To change the FluxInstance sync path, use `kubectl patch`:

```bash
kubectl patch fluxinstance flux -n flux-system --type merge -p '{"spec":{"sync":{"path":"gitops/clusters/homelab"}}}'
```

## Alternative Methods

### Method 1: Using kubectl edit (Interactive)
```bash
kubectl edit fluxinstance flux -n flux-system
```

Then change:
```yaml
spec:
  sync:
    path: "gitops/clusters/homelab"  # Update from "clusters/homelab"
```

### Method 2: Using kubectl patch (Recommended)
```bash
kubectl patch fluxinstance flux -n flux-system --type merge -p '{"spec":{"sync":{"path":"gitops/clusters/homelab"}}}'
```

### Method 3: Export, Modify, and Apply
```bash
# Export current FluxInstance
kubectl get fluxinstance flux -n flux-system -o yaml > fluxinstance.yaml

# Edit fluxinstance.yaml and change spec.sync.path to "gitops/clusters/homelab"

# Apply the updated configuration
kubectl apply -f fluxinstance.yaml
```

## Verify the Change

After updating, verify the path has been changed:

```bash
# Check the FluxInstance spec
kubectl get fluxinstance flux -n flux-system -o jsonpath='{.spec.sync.path}{"\n"}'

# Or view the full spec
kubectl get fluxinstance flux -n flux-system -o yaml | grep -A 10 "sync:"
```

## What Happens Next

1. Flux will detect the path change
2. The FluxInstance will reconcile and update the GitRepository
3. Kustomizations will continue to work, but they should also be updated to use the new `gitops/` prefix in their paths

## Important Notes

- The FluxInstance is managed by the Flux operator and is **not** typically stored in git
- After changing the path, verify that your Kustomization resources in `apps.yaml` and `infrastructure.yaml` also have paths updated to include the `gitops/` prefix
- The GitRepository itself doesn't need to change - only the path in the FluxInstance

