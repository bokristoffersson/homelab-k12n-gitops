# Rust Build Tools Setup for GitHub Actions

## Why Both Actions Are Needed

The `actions-rust-lang/setup-rust-toolchain@v1` action **only installs the Rust toolchain** (rustc, cargo, clippy, etc.). It does **NOT** install system-level build dependencies like:
- `gcc` / `g++` (C/C++ compilers)
- `cmake` (build system)
- `libssl-dev` (SSL libraries)
- `pkg-config` (library discovery)

These are OS packages that must be installed separately via the system package manager.

**GitHub-hosted runners** have these pre-installed, but **self-hosted runners** (like `homelab-runners`) typically don't, so you need to install them in your workflow.

When using self-hosted runners (like `homelab-runners`), you need to install build dependencies in your workflow since the runner image doesn't include them by default.

## Quick Setup Step

Add this step to your workflow **before** running `cargo` commands:

```yaml
- name: Install build dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends \
      build-essential \
      gcc \
      g++ \
      cmake \
      pkg-config \
      libssl-dev \
      ca-certificates
    sudo rm -rf /var/lib/apt/lists/*
```

## Complete Example

See `.github/workflows/rust-ci.example.yml` for a complete example workflow.

## Why This Works

- The runner runs as the `runner` user, which has `sudo` access
- Installing packages in the workflow step is cleaner than modifying the runner image
- Packages are installed fresh for each job, ensuring consistency
- No need to modify runner configuration

## Alternative: Use a Composite Action

You can create a reusable composite action in `.github/actions/setup-rust-build-tools/action.yml`:

```yaml
name: 'Setup Rust Build Tools'
description: 'Install build dependencies for Rust projects'
runs:
  using: 'composite'
  steps:
    - name: Install build dependencies
      shell: bash
      run: |
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends \
          build-essential \
          gcc \
          g++ \
          cmake \
          pkg-config \
          libssl-dev \
          ca-certificates
        sudo rm -rf /var/lib/apt/lists/*
```

Then use it in your workflows:

```yaml
- uses: ./.github/actions/setup-rust-build-tools
```

