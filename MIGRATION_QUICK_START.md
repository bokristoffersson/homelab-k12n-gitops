# Monorepo Migration - Quick Start Guide

This is a condensed guide for migrating to a monorepo structure. For detailed information, see [MONOREPO_MIGRATION_PLAN.md](./MONOREPO_MIGRATION_PLAN.md).

## 🎯 What This Migration Does

- Moves all GitOps configs to `gitops/` directory
- Creates `applications/` directory for source code
- Updates FluxCD paths to work with new structure
- Maintains zero-downtime (if done correctly)

## ⚡ Quick Migration (Automated)

### Step 1: Run Migration Script
```bash
./scripts/migrate-to-monorepo.sh
```

This will:
- ✅ Create backup branch
- ✅ Create directory structure
- ✅ Move GitOps content
- ✅ Update .gitignore

### Step 2: Update FluxCD Paths
```bash
./scripts/update-flux-paths.sh
```

This updates all Kustomization paths to include `gitops/` prefix.

### Step 3: Validate
```bash
./scripts/validate-migration.sh
```

### Step 4: Review & Commit
```bash
# Review changes
git diff

# Commit
git add -A
git commit -m "refactor: restructure for monorepo"

# Push
git push origin main
```

### Step 5: Monitor
```bash
# Watch FluxCD reconcile
flux get kustomizations
flux events --watch

# Verify applications
kubectl get pods --all-namespaces
```

## 📋 Manual Checklist

After running scripts, verify:

- [ ] All paths in `gitops/clusters/homelab/apps.yaml` have `gitops/` prefix
- [ ] All paths in `gitops/clusters/homelab/infrastructure.yaml` have `gitops/` prefix
- [ ] FluxInstance path updated (if exists): `kubectl get fluxinstance -n flux-system`
- [ ] All applications still running after push
- [ ] GitOps workflow tested (make small change, verify it deploys)

## 🔄 Rollback

If something goes wrong:

```bash
# Quick revert
git revert HEAD
git push origin main

# Or restore from backup
git checkout backup/pre-monorepo-migration
git push origin main --force
```

## 📁 New Structure

```
homelab-k12n-gitops/
├── gitops/              # All GitOps configs (moved here)
│   ├── clusters/
│   ├── infrastructure/
│   └── apps/
├── applications/        # Application source code (new)
├── scripts/            # Migration & utility scripts
└── docs/               # Documentation
```

## ⚠️ Important Notes

1. **Update FluxInstance manually** if it exists:
   ```bash
   kubectl edit fluxinstance flux -n flux-system
   # Change: path: "clusters/homelab"
   # To:     path: "gitops/clusters/homelab"
   ```

2. **Test before committing** - Run validation script first

3. **Monitor after push** - Watch FluxCD reconcile for 5-10 minutes

4. **Keep backup branch** - Don't delete `backup/pre-monorepo-migration`

## 🆘 Need Help?

- See [MONOREPO_MIGRATION_PLAN.md](./MONOREPO_MIGRATION_PLAN.md) for detailed steps
- See [scripts/README.md](./scripts/README.md) for script documentation
- Check FluxCD status: `flux check` and `flux get all`

## ✅ Success Criteria

Migration is successful when:
- ✅ All Kustomizations show "Ready" status
- ✅ All applications are running
- ✅ GitOps workflow works (test with small change)
- ✅ No errors in FluxCD logs

---

**Estimated Time**: 30-60 minutes  
**Risk Level**: Low (with proper testing)

