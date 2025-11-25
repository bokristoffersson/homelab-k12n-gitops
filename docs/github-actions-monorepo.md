# GitHub Actions in Monorepo

## Structure

All GitHub Actions workflows are stored in `.github/workflows/` at the repository root.

## Workflow Naming Convention

- Use descriptive names that include the application/service name
- Format: `{application-name}-ci-cd.yml`
- Example: `mqtt-to-timescale-ci-cd.yml`

## Path Filtering

All workflows use path filters to only trigger when relevant files change:

```yaml
on:
  push:
    branches: [main]
    paths:
      - 'applications/{app-name}/**'
      - '.github/workflows/{workflow-name}.yml'
```

This ensures workflows only run when their specific application changes, saving CI/CD resources and reducing build times.

## Adding a New Application Workflow

1. Create a new file in `.github/workflows/`
2. Name it `{application-name}-ci-cd.yml`
3. Add path filters for the application directory
4. Update working directories and build contexts to point to the application path

## Current Workflows

- `mqtt-to-timescale-ci-cd.yml` - Builds and tests the mqtt-to-timescale Rust application

