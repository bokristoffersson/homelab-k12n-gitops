````javascript
---
name: Chunk 1 Database Migrations
overview: Create database migration for continuous aggregates that will power the API's energy queries
todos:
    - id: create-migration-sql
    content: Create 005_add_energy_continuous_aggregates.sql with continuous aggregate definition
    status: pending
    - id: update-kustomization
    content: Add migration 005 to kustomization.yaml configMapGenerator
    status: pending
    - id: verify-migration-job
    content: Ensure migration-job.yaml has correct version label
    status: pending
    - id: verify-migration
    content: Deploy and verify migration runs successfully
    status: pending
---

# Chunk 1: Database Migrations

## Overview

Set up TimescaleDB continuous aggregates for efficient hourly energy queries. This is the foundation for the API endpoints.

## Files to Create/Modify

### 1. Create Migration SQL File

**File**: `gitops/apps/base/redpanda-sink/migrations/005_add_energy_continuous_aggregates.sql`

- Create continuous aggregate `energy_hourly` using TimescaleDB
- Use origin '2000-01-01 00:00:00+00' for consistent bucket alignment (no gaps)
- Calculate energy consumption as (last - first) to ensure continuity
- Include hourly energy consumption in kWh for all phases
- Add refresh policy (every 5 minutes)

### 2. Update Kustomization

**File**: `gitops/apps/base/redpanda-sink/kustomization.yaml`

- Add migration 005 to configMapGenerator files list

### 3. Verify Migration Job

**File**: `gitops/apps/base/redpanda-sink/migration-job.yaml`

- Ensure migration-version label is set to "005"

## Implementation Steps

1. Create the SQL migration file with continuous aggregate definition
2. Update kustomization.yaml to include the new migration
3. Commit and push changes
4. Verify migration runs successfully in cluster
5. Verify continuous aggregate exists and refreshes correctly

## Verification

```bash
# Check migration job
kubectl get jobs -n redpanda-sink

# Verify aggregate exists
kubectl exec -it -n heatpump-mqtt <postgres-pod> -- psql -U postgres -d heatpump
# Then: SELECT * FROM timescaledb_information.continuous_aggregates;
```

## Dependencies

- None (first chunk)

## Next Chunk

Chunk 2: Backend Foundation (dependencies, config, module structure)


````