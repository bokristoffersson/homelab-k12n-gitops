#!/usr/bin/env bash
# Bootstrap script: setup environment and build index

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== RAG-K8S Bootstrap ==="
echo ""

cd "$REPO_ROOT"

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $PYTHON_VERSION"

if ! python3 -c "import sys; sys.exit(0 if sys.version_info >= (3, 9) else 1)"; then
    echo "ERROR: Python 3.9+ required"
    exit 1
fi

# Create virtual environment
echo ""
echo "Creating virtual environment..."
make venv

# Install dependencies
echo ""
echo "Installing dependencies..."
make deps

# Build index
echo ""
echo "Building FAISS index from command cards..."
make build-index

# Create logs directory
mkdir -p logs

# Copy .env.example if .env doesn't exist
if [ ! -f .env ]; then
    echo ""
    echo "Creating .env from .env.example..."
    cp .env.example .env
    echo "Please edit .env with your configuration"
fi

echo ""
echo "=== Bootstrap Complete ==="
echo ""
echo "Next steps:"
echo "  1. Activate virtual environment: source venv/bin/activate"
echo "  2. Configure .env with your KUBECONFIG path"
echo "  3. Run demo: ./scripts/demo_restart.sh"
echo "  4. Run tests: make test"
echo ""
