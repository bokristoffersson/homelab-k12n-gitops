#!/bin/bash
# Generate Rust DTOs from contracts
# This script generates Rust types from JSON Schema contracts

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$REPO_ROOT/contracts"
OUTPUT_DIR="$REPO_ROOT/generated/rust-dtos/homelab-contracts/src"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Generating Rust DTOs from contracts...${NC}"

# Check if contracts directory exists
if [ ! -d "$CONTRACTS_DIR" ]; then
    echo -e "${YELLOW}⚠️  Contracts directory not found. Creating structure...${NC}"
    mkdir -p "$CONTRACTS_DIR"
fi

# Install typify if not available
if ! command -v typify &> /dev/null; then
    echo -e "${YELLOW}📦 Installing typify...${NC}"
    cargo install typify-cli --quiet
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Counter for generated files
GENERATED_COUNT=0

# Function to generate Rust types from JSON Schema
generate_from_schema() {
    local schema_path="$1"
    local module_name="$2"
    local output_file="$OUTPUT_DIR/${module_name}.rs"
    
    if [ ! -f "$schema_path" ]; then
        echo -e "${YELLOW}⚠️  Schema not found: $schema_path${NC}"
        return
    fi
    
    echo -e "${GREEN}📝 Generating ${module_name}.rs from $(basename "$schema_path")...${NC}"
    
    # Use typify to generate Rust code
    typify \
        --input "$schema_path" \
        --output "$output_file" \
        --crate-name "homelab_contracts" \
        --module-name "$module_name" || {
        echo -e "${RED}❌ Failed to generate ${module_name}.rs${NC}"
        return 1
    }
    
    GENERATED_COUNT=$((GENERATED_COUNT + 1))
}

# Generate types from contracts
# Example: mqtt-to-timescale telemetry
if [ -f "$CONTRACTS_DIR/mqtt-to-timescale/data-models/telemetry.schema.json" ]; then
    generate_from_schema \
        "$CONTRACTS_DIR/mqtt-to-timescale/data-models/telemetry.schema.json" \
        "telemetry"
fi

# Generate lib.rs with module declarations
echo -e "${GREEN}📝 Generating lib.rs...${NC}"

cat > "$OUTPUT_DIR/lib.rs" << 'LIB_EOF'
// Auto-generated Rust DTOs from contracts
// DO NOT EDIT MANUALLY - This file is generated from contracts/

pub mod telemetry;

// Re-export commonly used types
pub use telemetry::*;
LIB_EOF

# Generate mod.rs if needed
if [ -f "$OUTPUT_DIR/telemetry.rs" ]; then
    echo "pub mod telemetry;" > "$OUTPUT_DIR/mod.rs" || true
fi

# Build the crate to verify it compiles
if [ -f "$REPO_ROOT/generated/rust-dtos/homelab-contracts/Cargo.toml" ]; then
    echo -e "${GREEN}🔨 Building generated crate...${NC}"
    cd "$REPO_ROOT/generated/rust-dtos/homelab-contracts"
    cargo build --quiet || {
        echo -e "${RED}❌ Generated code failed to compile${NC}"
        exit 1
    }
    echo -e "${GREEN}✅ Generated code compiles successfully${NC}"
fi

echo -e "${GREEN}✅ Generated ${GENERATED_COUNT} Rust module(s)${NC}"
echo -e "${GREEN}✨ Done!${NC}"

