#!/bin/bash
# Monorepo Migration Script
# This script automates the repository restructuring phase of the migration

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🚀 Starting Monorepo Migration"
echo "Repository root: $REPO_ROOT"
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "⚠️  Warning: Not on main branch (currently on $CURRENT_BRANCH)"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Warning: You have uncommitted changes"
    git status --short
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Step 1: Create backup branch
echo "📦 Step 1: Creating backup branch..."
if git show-ref --verify --quiet refs/heads/backup/pre-monorepo-migration; then
    echo "   Backup branch already exists, skipping..."
else
    git checkout -b backup/pre-monorepo-migration
    git push origin backup/pre-monorepo-migration || echo "   (Could not push backup branch - continue anyway)"
    git checkout main
    echo "   ✅ Backup branch created"
fi

# Step 2: Create directory structure
echo ""
echo "📁 Step 2: Creating directory structure..."
cd "$REPO_ROOT"

# Create new directories if they don't exist
mkdir -p gitops
mkdir -p applications
mkdir -p scripts
mkdir -p docs

echo "   ✅ Directories created"

# Step 3: Move GitOps content
echo ""
echo "📦 Step 3: Moving GitOps content to gitops/ directory..."

# Check if directories exist before moving
if [ -d "clusters" ] && [ ! -d "gitops/clusters" ]; then
    git mv clusters gitops/ || echo "   ⚠️  clusters/ already moved or doesn't exist"
fi

if [ -d "infrastructure" ] && [ ! -d "gitops/infrastructure" ]; then
    git mv infrastructure gitops/ || echo "   ⚠️  infrastructure/ already moved or doesn't exist"
fi

if [ -d "apps" ] && [ ! -d "gitops/apps" ]; then
    git mv apps gitops/ || echo "   ⚠️  apps/ already moved or doesn't exist"
fi

echo "   ✅ GitOps content moved"

# Step 4: Update .gitignore
echo ""
echo "📝 Step 4: Updating .gitignore..."
if [ -f ".gitignore" ]; then
    if ! grep -q "# Application build artifacts" .gitignore; then
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
        echo "   ✅ .gitignore updated"
    else
        echo "   ℹ️  .gitignore already contains monorepo entries"
    fi
else
    echo "   ⚠️  .gitignore not found, creating new one..."
    cat > .gitignore << 'EOF'
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
    echo "   ✅ .gitignore created"
fi

# Step 5: Show status
echo ""
echo "📊 Step 5: Current status..."
echo ""
echo "Directory structure:"
tree -L 2 -d -I '.git' "$REPO_ROOT" 2>/dev/null || find "$REPO_ROOT" -maxdepth 2 -type d | grep -v ".git" | sort

echo ""
echo "Git status:"
git status --short

echo ""
echo "✅ Repository restructuring complete!"
echo ""
echo "📋 Next steps:"
echo "   1. Review the changes: git status"
echo "   2. Update FluxCD paths (see MONOREPO_MIGRATION_PLAN.md Phase 3)"
echo "   3. Commit the changes: git commit -m 'refactor: restructure for monorepo'"
echo "   4. Push and monitor: git push origin main"
echo ""
echo "⚠️  IMPORTANT: Update FluxCD paths before pushing!"
echo "   - Update gitops/clusters/homelab/apps.yaml"
echo "   - Update gitops/clusters/homelab/infrastructure.yaml"
echo "   - All paths need 'gitops/' prefix"

