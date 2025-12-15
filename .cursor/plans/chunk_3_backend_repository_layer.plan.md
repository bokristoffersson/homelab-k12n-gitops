````javascript
---
name: Chunk 3 Backend Repository Layer
overview: Implement data access layer for querying energy and heatpump data from database
todos:
    - id: create-energy-repo
    content: Implement energy.rs with get_latest, get_hourly_total, get_hourly_history
    status: pending
    - id: create-heatpump-repo
    content: Implement heatpump.rs with get_latest function
    status: pending
    - id: create-repo-mod
    content: Create mod.rs to export repository modules
    status: pending
    - id: integrate-repos
    content: Update main.rs/lib.rs to include repositories module
    status: pending
    - id: test-repos
    content: Verify repository functions compile and can be called
    status: pending
---

# Chunk 3: Backend Repository Layer

## Overview

Create the repository layer that provides database access methods for energy and heatpump data. This layer queries the continuous aggregates created in Chunk 1.

## Files to Create

### 1. Energy Repository

**File**: `applications/redpanda-sink/src/repositories/energy.rs`

- `get_latest()` - Query latest energy reading from timeseries table
- `get_hourly_total()` - Query continuous aggregate for current hour total
- `get_hourly_history()` - Query continuous aggregate for date range
- All methods return appropriate types (Option or Vec)

### 2. Heatpump Repository

**File**: `applications/redpanda-sink/src/repositories/heatpump.rs`

- `get_latest()` - Query latest heatpump status (with optional device_id filter)
- Returns latest status record

### 3. Repository Module

**File**: `applications/redpanda-sink/src/repositories/mod.rs`

- Export energy and heatpump modules
- Common types if needed

## Implementation Details

### Energy Repository Functions

- Use SQL queries to TimescaleDB continuous aggregate `energy_hourly`
- Handle timezone-aware queries
- Return structured data models

### Heatpump Repository Functions

- Query from heatpump status table
- Support filtering by device_id (optional parameter)
- Return most recent record

## Implementation Steps

1. Create repositories/energy.rs with energy query functions
2. Create repositories/heatpump.rs with heatpump query functions
3. Create repositories/mod.rs to export modules
4. Update lib.rs or main.rs to include repositories module
5. Write tests for repository functions (optional but recommended)
6. Test queries manually or with integration tests

## Verification

```bash
cd applications/redpanda-sink
cargo test repositories
cargo check
```

## Dependencies

- Chunk 1: Database migrations (continuous aggregates must exist)
- Chunk 2: Module structure and config setup

## Next Chunk

Chunk 4: Backend Authentication


````