# How to Check GitRepository Source Path

## Checking GitRepository Information

To check the source details of a GitRepository, use:

```bash
kubectl describe gitrepository flux-system -n flux-system
```

### What to Look For

In the output, check the **`Spec`** section:

- **`URL`**: The repository URL (e.g., `https://github.com/username/repo.git`)
- **`Ref.Name`**: The branch/tag/commit reference (e.g., `refs/heads/main`)

### Finding the Path

**Important**: GitRepository resources don't have a `path` field. The `path` is specified in the **Kustomization** resources that reference the GitRepository.

To find all paths that use a specific GitRepository:

```bash
# Find all Kustomizations that reference the GitRepository
kubectl get kustomization -n flux-system -o yaml | grep -A 5 "sourceRef:"
```

Or more specifically:

```bash
# Get all Kustomizations and show their paths
kubectl get kustomization -n flux-system -o custom-columns=NAME:.metadata.name,PATH:.spec.path,SOURCE:.spec.sourceRef.name
```

### Alternative: Get Full Spec

To see the complete GitRepository spec:

```bash
kubectl get gitrepository flux-system -n flux-system -o yaml
```

This will show:
- `spec.url` - Repository URL
- `spec.ref.name` - Branch/ref
- `spec.interval` - Sync interval
- `status.artifact.revision` - Last synced commit

