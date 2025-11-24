#!/bin/bash
# Migration Validation Script
# Validates that the monorepo migration was successful

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GITOPS_DIR="$REPO_ROOT/gitops"

echo "🔍 Validating Monorepo Migration"
echo ""

ERRORS=0
WARNINGS=0

# Check 1: Directory structure
echo "📁 Checking directory structure..."
if [ ! -d "$GITOPS_DIR" ]; then
    echo "   ❌ gitops/ directory not found"
    ERRORS=$((ERRORS + 1))
else
    echo "   ✅ gitops/ directory exists"
fi

if [ ! -d "$REPO_ROOT/applications" ]; then
    echo "   ⚠️  applications/ directory not found (optional)"
    WARNINGS=$((WARNINGS + 1))
else
    echo "   ✅ applications/ directory exists"
fi

# Check 2: GitOps files exist
echo ""
echo "📄 Checking GitOps files..."
REQUIRED_FILES=(
    "$GITOPS_DIR/clusters/homelab/apps.yaml"
    "$GITOPS_DIR/clusters/homelab/infrastructure.yaml"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✅ Found: $(basename $file)"
    else
        echo "   ❌ Missing: $file"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check 3: Paths in Kustomizations
echo ""
echo "🔗 Checking Kustomization paths..."
if [ -f "$GITOPS_DIR/clusters/homelab/apps.yaml" ]; then
    # Check if paths contain gitops/ prefix
    if grep -q "path: \./gitops/" "$GITOPS_DIR/clusters/homelab/apps.yaml" 2>/dev/null; then
        echo "   ✅ Apps paths include gitops/ prefix"
    elif grep -q "path: \./apps/" "$GITOPS_DIR/clusters/homelab/apps.yaml" 2>/dev/null; then
        echo "   ❌ Apps paths still use old structure (missing gitops/ prefix)"
        ERRORS=$((ERRORS + 1))
    else
        echo "   ⚠️  Could not verify apps paths"
        WARNINGS=$((WARNINGS + 1))
    fi
fi

if [ -f "$GITOPS_DIR/clusters/homelab/infrastructure.yaml" ]; then
    if grep -q "path: \"./gitops/infrastructure/" "$GITOPS_DIR/clusters/homelab/infrastructure.yaml" 2>/dev/null || \
       grep -q "path: \./gitops/infrastructure/" "$GITOPS_DIR/clusters/homelab/infrastructure.yaml" 2>/dev/null; then
        echo "   ✅ Infrastructure paths include gitops/ prefix"
    elif grep -q "path: \"./infrastructure/" "$GITOPS_DIR/clusters/homelab/infrastructure.yaml" 2>/dev/null || \
         grep -q "path: \./infrastructure/" "$GITOPS_DIR/clusters/homelab/infrastructure.yaml" 2>/dev/null; then
        echo "   ❌ Infrastructure paths still use old structure (missing gitops/ prefix)"
        ERRORS=$((ERRORS + 1))
    else
        echo "   ⚠️  Could not verify infrastructure paths"
        WARNINGS=$((WARNINGS + 1))
    fi
fi

# Check 4: Git status
echo ""
echo "📊 Checking git status..."
if git diff --quiet HEAD 2>/dev/null; then
    echo "   ✅ No uncommitted changes"
else
    echo "   ⚠️  You have uncommitted changes"
    WARNINGS=$((WARNINGS + 1))
    echo "      Run 'git status' to see them"
fi

# Check 5: FluxCD connectivity (if kubectl is available)
echo ""
echo "☸️  Checking FluxCD (if cluster is accessible)..."
if command -v kubectl &> /dev/null; then
    if kubectl cluster-info &> /dev/null; then
        if kubectl get gitrepository flux-system -n flux-system &> /dev/null; then
            echo "   ✅ GitRepository 'flux-system' exists"
            
            # Check GitRepository path
            REPO_PATH=$(kubectl get gitrepository flux-system -n flux-system -o jsonpath='{.spec.url}' 2>/dev/null || echo "")
            if [ -n "$REPO_PATH" ]; then
                echo "   ℹ️  Repository URL: $REPO_PATH"
            fi
        else
            echo "   ⚠️  GitRepository 'flux-system' not found (may be normal if not deployed yet)"
            WARNINGS=$((WARNINGS + 1))
        fi
        
        # Check Kustomizations
        KUST_COUNT=$(kubectl get kustomization -n flux-system --no-headers 2>/dev/null | wc -l || echo "0")
        if [ "$KUST_COUNT" -gt 0 ]; then
            echo "   ✅ Found $KUST_COUNT Kustomization(s) in cluster"
        else
            echo "   ⚠️  No Kustomizations found (may be normal if not deployed yet)"
            WARNINGS=$((WARNINGS + 1))
        fi
    else
        echo "   ⚠️  Cannot connect to cluster (skipping FluxCD checks)"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo "   ⚠️  kubectl not found (skipping FluxCD checks)"
    WARNINGS=$((WARNINGS + 1))
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "✅ Migration validation PASSED"
    echo ""
    echo "Next steps:"
    echo "  1. Review changes: git diff"
    echo "  2. Commit: git commit -m 'refactor: restructure for monorepo'"
    echo "  3. Push: git push origin main"
    echo "  4. Monitor: flux get kustomizations"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo "⚠️  Migration validation PASSED with warnings"
    echo "   Warnings: $WARNINGS"
    echo ""
    echo "Review warnings above and proceed if acceptable."
    exit 0
else
    echo "❌ Migration validation FAILED"
    echo "   Errors: $ERRORS"
    echo "   Warnings: $WARNINGS"
    echo ""
    echo "Please fix the errors above before proceeding."
    exit 1
fi

