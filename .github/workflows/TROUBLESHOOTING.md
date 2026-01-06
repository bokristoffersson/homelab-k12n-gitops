# Troubleshooting GitHub Actions Runner Issues

## Common Issues

### 1. "sudo: a password is required" or Permission Denied

**Problem**: The runner user doesn't have passwordless sudo access.

**Solution**: Ensure the runner container has sudo configured. The `actions-runner:latest` image should have this by default, but if not, you may need to:

1. Check if sudo is installed: `which sudo`
2. Check sudoers: The runner user should be in the sudoers file
3. Alternative: Use a custom runner image with build tools pre-installed

### 2. "linker `cc` not found" or "gcc: command not found"

**Problem**: Build tools aren't installed in the runner.

**Solution**: Make sure your workflow includes the build tools setup step **before** running cargo:

```yaml
- name: Install build dependencies
  uses: ./.github/actions/setup-rust-build-tools
```

### 3. Workflow runs but cargo fails with linker errors

**Problem**: Build tools are installed but not in PATH, or wrong architecture.

**Solution**: 
- Verify the installation step completed successfully
- Check the runner architecture matches your project (ARM64 vs AMD64)
- Ensure the step runs before any cargo commands

### 4. Runner pod fails to start

**Problem**: Kubernetes resource issues (CPU, memory, storage).

**Solution**:
- Check pod events: `kubectl describe pod <pod-name> -n actions-runners`
- Verify storage class is available
- Check resource limits in runner config

## Debugging Steps

### Check Runner Logs

```bash
# Find the runner pod
kubectl get pods -n actions-runners

# View logs
kubectl logs <pod-name> -n actions-runners --tail=100
```

### Test Sudo Access

Add this step to your workflow to test:

```yaml
- name: Test sudo access
  run: |
    sudo -v
    echo "Sudo access confirmed"
```

### Verify Build Tools Installation

Add this step after the install step:

```yaml
- name: Verify build tools
  run: |
    gcc --version
    g++ --version
    cmake --version
    pkg-config --version
```

## Alternative: Pre-installed Build Tools

If sudo access is problematic, consider creating a custom Docker image with build tools pre-installed:

```dockerfile
FROM ghcr.io/actions/actions-runner:latest

USER root
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      build-essential \
      gcc \
      g++ \
      cmake \
      pkg-config \
      libssl-dev \
      ca-certificates && \
    rm -rf /var/lib/apt/lists/*

USER runner
```

Then update your runner config to use this custom image instead of `ghcr.io/actions/actions-runner:latest`.

