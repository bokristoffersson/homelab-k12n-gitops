# Rust DTO Generation from Contracts

This guide explains how to generate Rust Data Transfer Objects (DTOs) from contracts and use them across services.

## Overview

Generate Rust types from:
- **JSON Schema** → Rust structs/enums
- **OpenAPI** → Rust client/server types
- **AsyncAPI** → Rust message types

## Architecture

```
contracts/                          # Source of truth
├── mqtt-to-timescale/
│   └── data-models/
│       └── telemetry.schema.json
│
generated/                          # Generated code (git-ignored or committed)
└── rust-dtos/
    └── homelab-contracts/          # Shared crate
        ├── Cargo.toml
        ├── src/
        │   ├── lib.rs
        │   ├── telemetry.rs        # Generated from JSON Schema
        │   └── mod.rs
        └── build.rs                # Generation script

applications/
└── mqtt-to-timescale/
    └── Cargo.toml                  # Depends on homelab-contracts
```

## Option 1: Shared Crate (Recommended)

Create a shared Rust crate that all services can import.

### Structure

```
generated/
└── rust-dtos/
    └── homelab-contracts/
        ├── Cargo.toml
        ├── build.rs
        ├── src/
        │   ├── lib.rs
        │   ├── telemetry.rs
        │   └── mod.rs
        └── schemas/                 # Symlink or copy from contracts/
            └── ...
```

### Cargo.toml for Shared Crate

```toml
[package]
name = "homelab-contracts"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }

[build-dependencies]
typify = "0.1"
typify-cli = "0.1"
```

### Generation Script (build.rs)

```rust
// generated/rust-dtos/homelab-contracts/build.rs
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../contracts");
    
    let contracts_dir = Path::new("../../contracts");
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Generate Rust types from JSON Schema
    generate_from_json_schema(
        contracts_dir.join("mqtt-to-timescale/data-models/telemetry.schema.json"),
        Path::new(&out_dir).join("telemetry.rs"),
    );
}

fn generate_from_json_schema(schema_path: impl AsRef<Path>, output_path: impl AsRef<Path>) {
    if !schema_path.as_ref().exists() {
        return;
    }
    
    let schema_content = fs::read_to_string(schema_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    // Use typify to generate Rust code
    let rust_code = typify::generate_types(&schema).unwrap();
    
    fs::write(output_path, rust_code).unwrap();
}
```

### Using in Services

```toml
# applications/mqtt-to-timescale/Cargo.toml
[dependencies]
homelab-contracts = { path = "../../generated/rust-dtos/homelab-contracts" }
```

```rust
// applications/mqtt-to-timescale/src/main.rs
use homelab_contracts::TelemetryRow;

fn process_message(payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let row: TelemetryRow = serde_json::from_slice(payload)?;
    // Use row...
    Ok(())
}
```

## Option 2: Using typify CLI (Simpler)

Generate types at build time using `typify` CLI tool.

### Installation

```bash
cargo install typify-cli
```

### Generation Script

```bash
#!/bin/bash
# scripts/generate-rust-dtos.sh

set -e

CONTRACTS_DIR="contracts"
OUTPUT_DIR="generated/rust-dtos/homelab-contracts/src"

mkdir -p "$OUTPUT_DIR"

# Generate from JSON Schema
typify \
  --input "$CONTRACTS_DIR/mqtt-to-timescale/data-models/telemetry.schema.json" \
  --output "$OUTPUT_DIR/telemetry.rs" \
  --crate-name "homelab_contracts" \
  --module-name "telemetry"
```

### CI Integration

Run generation in CI before building services.

## Option 3: Using openapi-generator (For OpenAPI)

If you have OpenAPI specs:

```bash
# Install
brew install openapi-generator  # or download from GitHub

# Generate Rust client
openapi-generator generate \
  -i contracts/grafana/openapi/api.yaml \
  -g rust \
  -o generated/rust-dtos/grafana-client
```

## CI Pipeline Setup

### GitHub Actions Example

```yaml
# .github/workflows/generate-dtos.yml
name: Generate Rust DTOs

on:
  push:
    paths:
      - 'contracts/**'
    branches: [main]
  pull_request:
    paths:
      - 'contracts/**'
  workflow_dispatch:

jobs:
  generate-dtos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install typify
        run: cargo install typify-cli
      
      - name: Generate Rust DTOs
        run: |
          mkdir -p generated/rust-dtos/homelab-contracts/src
          ./scripts/generate-rust-dtos.sh
      
      - name: Build generated crate
        run: |
          cd generated/rust-dtos/homelab-contracts
          cargo build
          cargo test
      
      - name: Check if DTOs changed
        id: check-dtos
        run: |
          if [ -n "$(git status --porcelain generated/)" ]; then
            echo "changed=true" >> $GITHUB_OUTPUT
          else
            echo "changed=false" >> $GITHUB_OUTPUT
          fi
      
      - name: Create PR with generated code
        if: steps.check-dtos.outputs.changed == 'true' && github.event_name == 'push'
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: "chore: regenerate Rust DTOs from contracts"
          title: "Regenerate Rust DTOs"
          body: |
            Auto-generated PR: Rust DTOs updated from contract changes.
            
            **Do not merge manually** - this will be auto-merged or updated on next contract change.
          branch: auto/regenerate-dtos
          delete-branch: true
```

### Alternative: Commit Generated Code

If you prefer committing generated code:

```yaml
# .github/workflows/generate-dtos.yml
name: Generate Rust DTOs

on:
  push:
    paths:
      - 'contracts/**'
    branches: [main]

jobs:
  generate-and-commit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install typify
        run: cargo install typify-cli
      
      - name: Generate Rust DTOs
        run: ./scripts/generate-rust-dtos.sh
      
      - name: Commit generated code
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add generated/
          git diff --staged --quiet || git commit -m "chore: regenerate Rust DTOs from contracts"
          git push
```

## Complete Example: Shared Crate Setup

### 1. Create Shared Crate Structure

```bash
mkdir -p generated/rust-dtos/homelab-contracts/src
```

### 2. Cargo.toml

```toml
[package]
name = "homelab-contracts"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
pretty_assertions = "1.4"
```

### 3. Generation Script

```bash
#!/bin/bash
# scripts/generate-rust-dtos.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$REPO_ROOT/contracts"
OUTPUT_DIR="$REPO_ROOT/generated/rust-dtos/homelab-contracts/src"

# Install typify if not available
if ! command -v typify &> /dev/null; then
    echo "Installing typify..."
    cargo install typify-cli
fi

mkdir -p "$OUTPUT_DIR"

# Generate telemetry types
if [ -f "$CONTRACTS_DIR/mqtt-to-timescale/data-models/telemetry.schema.json" ]; then
    echo "Generating telemetry.rs..."
    typify \
        --input "$CONTRACTS_DIR/mqtt-to-timescale/data-models/telemetry.schema.json" \
        --output "$OUTPUT_DIR/telemetry.rs" \
        --crate-name "homelab_contracts" \
        --module-name "telemetry"
fi

# Generate lib.rs
cat > "$OUTPUT_DIR/lib.rs" << 'EOF'
pub mod telemetry;

pub use telemetry::*;
EOF

echo "✅ Rust DTOs generated successfully!"
```

### 4. Use in Service

```toml
# applications/mqtt-to-timescale/Cargo.toml
[dependencies]
homelab-contracts = { path = "../../generated/rust-dtos/homelab-contracts" }
```

```rust
// applications/mqtt-to-timescale/src/mapping.rs
use homelab_contracts::TelemetryRow;

pub fn parse_message(payload: &[u8]) -> Result<TelemetryRow, serde_json::Error> {
    serde_json::from_slice(payload)
}
```

## Advanced: Custom Code Generation

For more control, use a custom Rust script:

```rust
// scripts/generate-dtos.rs
use std::fs;
use std::path::Path;

fn main() {
    let schema = fs::read_to_string("contracts/mqtt-to-timescale/data-models/telemetry.schema.json")
        .unwrap();
    
    // Parse JSON Schema
    let schema_value: serde_json::Value = serde_json::from_str(&schema).unwrap();
    
    // Generate Rust code
    let rust_code = generate_rust_code(&schema_value);
    
    fs::write("generated/rust-dtos/homelab-contracts/src/telemetry.rs", rust_code)
        .unwrap();
}

fn generate_rust_code(schema: &serde_json::Value) -> String {
    // Custom generation logic
    // Use a library like json_typegen or write custom logic
    format!(
        r#"
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRow {{
    pub ts: chrono::DateTime<chrono::Utc>,
    // ... fields from schema
}}
"#
    )
}
```

## Best Practices

1. **Version Contracts**: Tag contract versions, generate DTOs per version
2. **Validate**: Run `cargo build` and `cargo test` on generated code
3. **Documentation**: Add doc comments to generated types
4. **CI Checks**: Fail CI if contracts change but DTOs aren't regenerated
5. **Git Strategy**: 
   - Option A: Commit generated code (simpler, larger diffs)
   - Option B: Generate in CI, don't commit (cleaner, requires CI)

## Tools Comparison

| Tool | Input | Output | Pros | Cons |
|------|-------|--------|------|------|
| `typify` | JSON Schema | Rust types | Simple, good defaults | Less customization |
| `openapi-generator` | OpenAPI | Rust client/server | Full-featured | Complex config |
| `json_typegen` | JSON Schema | Rust types | Customizable | More setup |
| Custom script | Any | Rust types | Full control | More maintenance |

## Next Steps

1. Set up the shared crate structure
2. Create generation script
3. Add CI pipeline
4. Update services to use generated DTOs
5. Add validation tests

