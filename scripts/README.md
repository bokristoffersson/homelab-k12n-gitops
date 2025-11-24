# Migration Scripts

Helper scripts for migrating to monorepo structure.

## Scripts

### `migrate-to-monorepo.sh`
Automates the repository restructuring phase:
- Creates backup branch
- Creates new directory structure
- Moves GitOps content to `gitops/` directory
- Updates `.gitignore`

**Usage:**
```bash
./scripts/migrate-to-monorepo.sh
```

**Note:** This script does NOT update FluxCD paths. You must do that manually or use `update-flux-paths.sh`.

---

### `update-flux-paths.sh`
Updates all FluxCD Kustomization paths to include the `gitops/` prefix.

**Usage:**
```bash
./scripts/update-flux-paths.sh
```

**What it does:**
- Updates `gitops/clusters/homelab/apps.yaml`
- Updates `gitops/clusters/homelab/infrastructure.yaml`
- Changes paths from `./apps/` to `./gitops/apps/`
- Changes paths from `./infrastructure/` to `./gitops/infrastructure/`

---

### `validate-migration.sh`
Validates that the monorepo migration was successful.

**Usage:**
```bash
./scripts/validate-migration.sh
```

**Checks:**
- Directory structure exists
- Required files are present
- FluxCD paths are updated
- Git status is clean
- FluxCD connectivity (if cluster accessible)

---

## Quick Migration Workflow

1. **Run migration script:**
   ```bash
   ./scripts/migrate-to-monorepo.sh
   ```

2. **Update FluxCD paths:**
   ```bash
   ./scripts/update-flux-paths.sh
   ```

3. **Validate migration:**
   ```bash
   ./scripts/validate-migration.sh
   ```

4. **Review changes:**
   ```bash
   git diff
   ```

5. **Commit and push:**
   ```bash
   git add -A
   git commit -m "refactor: restructure for monorepo"
   git push origin main
   ```

6. **Monitor FluxCD:**
   ```bash
   flux get kustomizations
   flux events --watch
   ```

---

## Manual Steps

Some steps must be done manually:

1. **Update FluxInstance path** (if exists):
   ```bash
   kubectl edit fluxinstance flux -n flux-system
   # Change path to: gitops/clusters/homelab
   ```

2. **Migrate application source code:**
   - Copy source code to `applications/` directory
   - Add application-specific documentation

3. **Update main README.md:**
   - Document new monorepo structure
   - Update quick start guide

---

## Troubleshooting

### Script fails with "Not in a git repository"
Make sure you're running from the repository root.

### Paths not updating
Check that the files exist and have the correct format. You may need to update manually.

### Validation fails
Review the error messages and fix issues before proceeding. Common issues:
- Paths not updated
- Files in wrong location
- Uncommitted changes

---

## Safety

All scripts:
- Create backups before making changes
- Show what they're doing
- Allow you to review before committing
- Can be safely re-run (idempotent)

**Always review changes before committing!**

