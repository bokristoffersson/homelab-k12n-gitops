# Monorepo Migration Plan

## 📋 Executive Summary

This document outlines the step-by-step plan to migrate from a GitOps-only repository to a full monorepo structure that includes both GitOps configurations and application source code.

**Migration Type**: In-place restructuring with zero-downtime  
**Estimated Time**: 2-4 hours  
**Risk Level**: Low (with proper testing)

---

## 🎯 Goals

1. Consolidate GitOps configs and application source code into a single repository
2. Maintain zero-downtime during migration
3. Preserve all Git history
4. Update FluxCD paths to work with new structure
5. Establish clear directory structure for future growth

---

## 📊 Current State Analysis

### Current Repository Structure
```
homelab-k12n-gitops/
├── clusters/
│   └── homelab/
│       ├── apps.yaml
│       └── infrastructure.yaml
├── infrastructure/
│   ├── controllers/
│   ├── crds/
│   ├── sources/
│   └── storage/
└── apps/
    ├── base/
    └── homelab/
```

### Current FluxCD Configuration
- **GitRepository**: `flux-system` (created by FluxInstance)
- **Kustomization Paths**: 
  - Apps: `./apps/homelab/*`
  - Infrastructure: `./infrastructure/*`
- **All paths are relative to repository root**

### Applications Identified
1. **heatpump-mqtt** - Custom application (has source code)
2. **whoami** - Test application
3. **grafana** - Helm release
4. **loki** - Helm release
5. **monitoring** - Helm release
6. **alloy** - Helm release
7. **mosquitto** - Custom deployment
8. **redpanda** - Helm release

---

## 🏗️ Target Monorepo Structure

```
homelab-monorepo/
├── .github/                    # GitHub workflows (if needed)
│   └── workflows/
├── gitops/                     # GitOps configurations (current repo content)
│   ├── clusters/
│   │   └── homelab/
│   │       ├── apps.yaml
│   │       └── infrastructure.yaml
│   ├── infrastructure/
│   │   ├── controllers/
│   │   ├── crds/
│   │   ├── sources/
│   │   └── storage/
│   └── apps/
│       ├── base/
│       └── homelab/
├── applications/               # Application source code
│   ├── heatpump-mqtt/         # MQTT to TimescaleDB service
│   │   ├── src/
│   │   ├── Dockerfile
│   │   ├── Cargo.toml          # (if Rust)
│   │   └── README.md
│   └── [future-apps]/
├── scripts/                    # Utility scripts
│   ├── build.sh
│   ├── deploy.sh
│   └── validate.sh
├── docs/                       # Documentation
│   ├── architecture.md
│   └── deployment.md
├── .gitignore
├── README.md
└── Makefile                    # Build automation (optional)
```

---

## 📝 Migration Steps

### Phase 1: Preparation (30 minutes)

#### Step 1.1: Create Backup Branch
```bash
# Ensure you're on main and up to date
git checkout main
git pull origin main

# Create backup branch
git checkout -b backup/pre-monorepo-migration
git push origin backup/pre-monorepo-migration
```

#### Step 1.2: Document Current State
```bash
# Export current FluxCD state
kubectl get gitrepository -n flux-system -o yaml > flux-state-backup.yaml
kubectl get kustomization -n flux-system -o yaml >> flux-state-backup.yaml
kubectl get fluxinstance -n flux-system -o yaml >> flux-state-backup.yaml

# Save to repo
git add flux-state-backup.yaml
git commit -m "docs: backup FluxCD state before monorepo migration"
```

#### Step 1.3: Verify Cluster Health
```bash
# Check all applications are running
kubectl get pods --all-namespaces

# Check FluxCD status
flux check
flux get all

# Verify GitRepository is healthy
kubectl get gitrepository flux-system -n flux-system
```

---

### Phase 2: Repository Restructuring (45 minutes)

#### Step 2.1: Create New Directory Structure
```bash
# Create new directories
mkdir -p gitops
mkdir -p applications
mkdir -p scripts
mkdir -p docs

# Move existing GitOps content to gitops/
git mv clusters gitops/
git mv infrastructure gitops/
git mv apps gitops/
```

#### Step 2.2: Update Root README
```bash
# Update README.md to reflect monorepo structure
# (See Step 2.3 for content)
```

#### Step 2.3: Create/Update .gitignore
```bash
# Add application-specific ignores
cat >> .gitignore << 'EOF'

# Application build artifacts
applications/*/target/
applications/*/dist/
applications/*/build/
applications/*/node_modules/
applications/*/.venv/
applications/*/__pycache__/
applications/*/*.egg-info/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
EOF

git add .gitignore
```

#### Step 2.4: Commit Restructuring
```bash
git add -A
git commit -m "refactor: restructure repository for monorepo

- Move GitOps configs to gitops/ directory
- Create applications/ directory for source code
- Add scripts/ and docs/ directories
- Update .gitignore for monorepo structure"
```

---

### Phase 3: Update FluxCD Configuration (30 minutes)

#### Step 3.1: Update Kustomization Paths in apps.yaml
All paths need to be prefixed with `gitops/`:

**Before:**
```yaml
path: ./apps/homelab/whoami
```

**After:**
```yaml
path: ./gitops/apps/homelab/whoami
```

**Files to update:**
- `gitops/clusters/homelab/apps.yaml`
- `gitops/clusters/homelab/infrastructure.yaml`

#### Step 3.2: Update Infrastructure Paths
**Before:**
```yaml
path: "./infrastructure/crds"
```

**After:**
```yaml
path: "./gitops/infrastructure/crds"
```

#### Step 3.3: Update FluxInstance Path (if exists)
If you have a FluxInstance resource, update the path:

**Before:**
```yaml
spec:
  sync:
    path: "clusters/homelab"
```

**After:**
```yaml
spec:
  sync:
    path: "gitops/clusters/homelab"
```

**Note:** The FluxInstance might be managed outside the repo. Check with:
```bash
kubectl get fluxinstance -n flux-system -o yaml
```

#### Step 3.4: Commit FluxCD Updates
```bash
git add gitops/clusters/
git commit -m "fix: update FluxCD paths for monorepo structure

- Update all Kustomization paths to include gitops/ prefix
- Update infrastructure paths
- Maintains backward compatibility with existing resources"
```

---

### Phase 4: Apply Changes to Cluster (15 minutes)

#### Step 4.1: Push Changes
```bash
git push origin main
```

#### Step 4.2: Monitor FluxCD Reconciliation
```bash
# Watch FluxCD reconcile
watch -n 2 'flux get kustomizations && echo "---" && kubectl get pods -n flux-system'

# Or use flux events
flux events --watch
```

#### Step 4.3: Verify GitRepository Update
```bash
# Check GitRepository status
kubectl get gitrepository flux-system -n flux-system -o yaml

# Check if it's using the new path
kubectl describe gitrepository flux-system -n flux-system
```

#### Step 4.4: Verify Kustomizations
```bash
# Check all Kustomizations are reconciling
flux get kustomizations

# Check for any errors
kubectl get kustomization -n flux-system
kubectl describe kustomization -n flux-system <name>
```

#### Step 4.5: Verify Applications
```bash
# Check all pods are still running
kubectl get pods --all-namespaces

# Test a sample application
kubectl get deployment -A
```

---

### Phase 5: Migrate Application Source Code (60 minutes)

#### Step 5.1: Identify Application Source Locations
For each custom application (not Helm releases), locate the source code:

**heatpump-mqtt:**
- Current image: `ghcr.io/bokristoffersson/mqtt-to-timescale:main-7450a3d`
- Source likely in separate repo or local

#### Step 5.2: Move/Copy Source Code
```bash
# If source is in another repo
git clone <source-repo-url> applications/heatpump-mqtt
cd applications/heatpump-mqtt
rm -rf .git  # Remove separate git history (or keep if preferred)

# If source is local
cp -r /path/to/source applications/heatpump-mqtt
```

#### Step 5.3: Create Application README
Each application should have a README:
```markdown
# heatpump-mqtt

MQTT to TimescaleDB service for heatpump data collection.

## Building

```bash
docker build -t ghcr.io/bokristoffersson/mqtt-to-timescale:latest .
```

## Deployment

Managed via GitOps in `gitops/apps/base/heatpump-mqtt/`
```

#### Step 5.4: Update GitOps to Reference Local Source (Optional)
If you want to build from local source, you can add build automation:

```bash
# Create scripts/build-heatpump-mqtt.sh
#!/bin/bash
set -e

cd applications/heatpump-mqtt
docker build -t ghcr.io/bokristoffersson/mqtt-to-timescale:$(git rev-parse --short HEAD) .
docker push ghcr.io/bokristoffersson/mqtt-to-timescale:$(git rev-parse --short HEAD)
```

#### Step 5.5: Commit Application Source
```bash
git add applications/
git commit -m "feat: add application source code to monorepo

- Add heatpump-mqtt source code
- Add application documentation
- Prepare for unified build/deploy workflow"
```

---

### Phase 6: Testing & Validation (30 minutes)

#### Step 6.1: Verify FluxCD Health
```bash
# Full FluxCD check
flux check

# Check all resources
flux get all

# Check for any suspended resources
flux get kustomizations --all-namespaces | grep Suspended
```

#### Step 6.2: Test Application Functionality
```bash
# Test each application
# Example: heatpump-mqtt
kubectl get pods -n heatpump-mqtt
kubectl logs -n heatpump-mqtt -l app=heatpump-mqtt --tail=50

# Test whoami
kubectl port-forward service/homelab-whoami 8080:80 -n default &
curl http://localhost:8080
```

#### Step 6.3: Test GitOps Workflow
```bash
# Make a small change to test GitOps
# Edit gitops/apps/base/whoami/deployment.yaml (e.g., add a label)

git add gitops/apps/base/whoami/deployment.yaml
git commit -m "test: verify GitOps workflow after monorepo migration"
git push origin main

# Watch FluxCD pick up the change
flux events --watch

# Verify change is applied
kubectl get deployment homelab-whoami -o yaml | grep -A 5 labels
```

#### Step 6.4: Verify Path Resolution
```bash
# Check that all Kustomizations can resolve their paths
for kust in $(kubectl get kustomization -n flux-system -o name); do
  echo "Checking $kust"
  kubectl describe $kust -n flux-system | grep -A 3 "Path:"
done
```

---

### Phase 7: Documentation & Cleanup (30 minutes)

#### Step 7.1: Update Main README
Update `README.md` to reflect monorepo structure:

```markdown
# Homelab Monorepo

This repository contains both GitOps configurations and application source code for the homelab infrastructure.

## Structure

- `gitops/` - FluxCD GitOps configurations
- `applications/` - Application source code
- `scripts/` - Utility and automation scripts
- `docs/` - Documentation

## Quick Start

### GitOps
All Kubernetes deployments are managed via FluxCD. See `gitops/README.md` for details.

### Applications
Application source code is in `applications/`. Each application has its own README.

## Development Workflow

1. Make changes to application source in `applications/`
2. Build and push container image
3. Update image tag in `gitops/apps/base/<app>/deployment.yaml`
4. Commit and push - FluxCD will deploy automatically
```

#### Step 7.2: Create Migration Summary
```bash
# Create docs/MIGRATION_COMPLETE.md
cat > docs/MIGRATION_COMPLETE.md << 'EOF'
# Monorepo Migration Complete

**Date**: $(date)
**Status**: ✅ Complete

## Changes Made
- Restructured repository with gitops/ directory
- Updated all FluxCD paths
- Migrated application source code
- All applications verified working

## Verification
- [x] FluxCD health check passed
- [x] All Kustomizations reconciling
- [x] All applications running
- [x] GitOps workflow tested
EOF
```

#### Step 7.3: Clean Up Temporary Files
```bash
# Remove backup files if migration successful
# (Keep backup branch for safety)
rm -f flux-state-backup.yaml
```

---

## 🔄 Rollback Plan

If issues occur during migration:

### Quick Rollback (if caught early)
```bash
# Revert the restructuring commit
git revert HEAD
git push origin main

# FluxCD will automatically reconcile with old paths
```

### Full Rollback (if needed)
```bash
# Switch back to backup branch
git checkout backup/pre-monorepo-migration
git push origin main --force

# Or restore from backup
git checkout main
git reset --hard backup/pre-monorepo-migration
git push origin main --force
```

### Manual Path Fix (if only paths are wrong)
```bash
# Edit paths back to original
# Then update FluxInstance/GitRepository if needed
kubectl edit fluxinstance flux -n flux-system
# Change path back to: clusters/homelab
```

---

## ✅ Post-Migration Checklist

- [ ] All FluxCD Kustomizations are healthy
- [ ] All applications are running
- [ ] GitOps workflow tested and working
- [ ] Application source code migrated
- [ ] Documentation updated
- [ ] Team/self informed of new structure
- [ ] CI/CD pipelines updated (if applicable)
- [ ] Backup branch kept for safety

---

## 🚀 Future Enhancements

After successful migration, consider:

1. **Build Automation**
   - GitHub Actions for building applications
   - Automatic image tagging based on git commit
   - Integration with FluxCD image automation

2. **Development Scripts**
   - `scripts/build-all.sh` - Build all applications
   - `scripts/deploy-local.sh` - Deploy to local cluster
   - `scripts/validate.sh` - Validate GitOps configs

3. **Documentation**
   - Architecture diagrams
   - Deployment guides per application
   - Troubleshooting guides

4. **Testing**
   - Unit tests for applications
   - Integration tests
   - GitOps config validation

---

## 📞 Troubleshooting

### Issue: FluxCD can't find paths
**Solution**: Verify paths in Kustomizations match new structure. Check:
```bash
kubectl get kustomization -n flux-system -o yaml | grep path:
```

### Issue: Applications not updating
**Solution**: Check GitRepository is watching correct branch and path:
```bash
kubectl describe gitrepository flux-system -n flux-system
```

### Issue: Build failures
**Solution**: Ensure application dependencies are in .gitignore and documented in README.

---

## 📚 References

- [FluxCD Documentation](https://fluxcd.io/docs/)
- [Kustomize Documentation](https://kustomize.io/)
- [Monorepo Best Practices](https://monorepo.tools/)

---

**Migration Date**: _[To be filled after completion]_  
**Migrated By**: _[Your name]_  
**Status**: _[Pending/In Progress/Complete]_

