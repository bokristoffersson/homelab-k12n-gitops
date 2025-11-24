#!/bin/bash
# Update FluxCD Paths Script
# This script updates all FluxCD Kustomization paths to include the gitops/ prefix

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GITOPS_DIR="$REPO_ROOT/gitops"

echo "🔧 Updating FluxCD Paths for Monorepo Structure"
echo ""

# Check if gitops directory exists
if [ ! -d "$GITOPS_DIR" ]; then
    echo "❌ Error: gitops/ directory not found"
    echo "   Run migrate-to-monorepo.sh first"
    exit 1
fi

# Function to update paths in a file
update_paths_in_file() {
    local file=$1
    local backup_file="${file}.backup"
    
    if [ ! -f "$file" ]; then
        echo "   ⚠️  File not found: $file"
        return
    fi
    
    # Create backup
    cp "$file" "$backup_file"
    
    # Update paths
    # Replace "./apps/" with "./gitops/apps/"
    sed -i.bak 's|path: \./apps/|path: ./gitops/apps/|g' "$file"
    # Replace "./infrastructure/" with "./gitops/infrastructure/"
    sed -i.bak 's|path: "\./infrastructure/|path: "./gitops/infrastructure/|g' "$file"
    sed -i.bak 's|path: \./infrastructure/|path: ./gitops/infrastructure/|g' "$file"
    
    # Remove .bak file created by sed
    rm -f "${file}.bak"
    
    # Check if changes were made
    if ! diff -q "$file" "$backup_file" > /dev/null; then
        echo "   ✅ Updated: $file"
        rm "$backup_file"
        return 0
    else
        echo "   ℹ️  No changes needed: $file"
        rm "$backup_file"
        return 1
    fi
}

# Update apps.yaml
echo "📝 Updating apps.yaml..."
APPS_FILE="$GITOPS_DIR/clusters/homelab/apps.yaml"
if update_paths_in_file "$APPS_FILE"; then
    CHANGES_MADE=true
fi

# Update infrastructure.yaml
echo ""
echo "📝 Updating infrastructure.yaml..."
INFRA_FILE="$GITOPS_DIR/clusters/homelab/infrastructure.yaml"
if update_paths_in_file "$INFRA_FILE"; then
    CHANGES_MADE=true
fi

echo ""
if [ "$CHANGES_MADE" = true ]; then
    echo "✅ Path updates complete!"
    echo ""
    echo "📋 Review the changes:"
    echo "   git diff gitops/clusters/"
    echo ""
    echo "⚠️  Verify the paths are correct before committing!"
else
    echo "ℹ️  No path updates needed (paths may already be updated)"
fi

echo ""
echo "🔍 Sample of updated paths:"
grep -h "path:" "$APPS_FILE" "$INFRA_FILE" 2>/dev/null | head -5 || echo "   (No paths found)"

