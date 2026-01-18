#!/bin/bash
# Check vulnerabilities in both frontend and backend applications

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔍 Checking for vulnerabilities in codebase..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

EXIT_CODE=0

# Check frontend vulnerabilities
echo "📦 Checking frontend (heatpump-web) dependencies..."
cd "$REPO_ROOT/applications/heatpump-web"

if [ -f "package.json" ]; then
    echo "Running npm audit..."
    if npm audit --omit=dev 2>&1 | tee /tmp/npm-audit-output.txt; then
        echo -e "${GREEN}✅ Frontend: No critical vulnerabilities found${NC}"
    else
        AUDIT_EXIT=$?
        # npm audit exits with non-zero if vulnerabilities are found
        if grep -q "vulnerabilities" /tmp/npm-audit-output.txt; then
            echo -e "${RED}❌ Frontend: Vulnerabilities found!${NC}"
            echo "Run 'npm audit fix' in applications/heatpump-web/ to fix automatically"
            EXIT_CODE=1
        else
            # Other error (network, etc.)
            echo -e "${YELLOW}⚠️ Frontend: npm audit encountered an error${NC}"
        fi
    fi
    echo ""
else
    echo -e "${YELLOW}⚠️ Frontend: package.json not found${NC}"
    echo ""
fi

# Check backend vulnerabilities
echo "🦀 Checking backend (heatpump-settings-api) dependencies..."
cd "$REPO_ROOT/applications/heatpump-settings-api"

if [ -f "Cargo.toml" ]; then
    # Check if cargo-audit is installed
    if ! command -v cargo-audit &> /dev/null; then
        echo -e "${YELLOW}⚠️ cargo-audit not installed. Installing...${NC}"
        cargo install cargo-audit --locked
    fi
    
    echo "Running cargo audit..."
    if cargo audit 2>&1 | tee /tmp/cargo-audit-output.txt; then
        echo -e "${GREEN}✅ Backend: No vulnerabilities found${NC}"
    else
        AUDIT_EXIT=$?
        if grep -qi "vulnerability\|unsound\|yanked" /tmp/cargo-audit-output.txt; then
            echo -e "${RED}❌ Backend: Vulnerabilities found!${NC}"
            echo "Run 'cargo audit fix' in applications/heatpump-settings-api/ to fix automatically (if available)"
            EXIT_CODE=1
        else
            echo -e "${YELLOW}⚠️ Backend: cargo audit encountered an error${NC}"
        fi
    fi
    echo ""
else
    echo -e "${YELLOW}⚠️ Backend: Cargo.toml not found${NC}"
    echo ""
fi

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Vulnerability check complete - No issues found!${NC}"
else
    echo -e "${RED}❌ Vulnerability check complete - Issues found!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Review the vulnerabilities above"
    echo "2. Run 'npm audit fix' in applications/heatpump-web/ for frontend"
    echo "3. Update Cargo.toml dependencies in applications/heatpump-settings-api/ for backend"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

exit $EXIT_CODE
