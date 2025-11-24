# Testing Guide

## Running Tests

### Unit Tests
Unit tests test the business logic without requiring a database:

```bash
cargo test --lib
```

### Integration Tests
Integration tests require a PostgreSQL database. They use generated test data and automatically set up the database schema.

#### Local Testing
Set the `DATABASE_URL` environment variable and run:

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/testdb"
cargo test --test integration_test
```

#### CI/CD Testing
The GitHub Actions workflow automatically:
1. Sets up a PostgreSQL service container
2. Creates the test database schema
3. Runs all tests with generated data

## Test Data Generation

The integration tests use the `test_helpers` module which provides:

- `create_test_pool()` - Creates a database connection pool
- `setup_test_schema()` - Sets up the heatpump table schema
- `cleanup_test_data()` - Cleans up test data between tests
- `insert_test_reading()` - Inserts a single test reading with random data
- `insert_test_readings()` - Inserts multiple test readings

All test data is randomly generated with realistic ranges:
- Temperatures: -10°C to 60°C
- Speeds: 0-100
- Runtime values: 0-10000
- Boolean flags: Random true/false with appropriate probabilities

## Test Coverage

The integration tests cover:

1. **List Operations**
   - Default parameters
   - Filtering by device_id
   - Filtering by room
   - Time range filtering
   - Pagination

2. **Get Operations**
   - Get latest reading
   - Get by timestamp and device_id

3. **Validation**
   - Limit too large (> 1000)
   - Negative offset
   - Invalid time range (start > end)
   - Not found errors

## Database Schema

The tests automatically create the following schema:

```sql
CREATE TABLE heatpump (
    ts TIMESTAMPTZ NOT NULL,
    device_id TEXT,
    room TEXT,
    outdoor_temp DOUBLE PRECISION,
    supplyline_temp DOUBLE PRECISION,
    returnline_temp DOUBLE PRECISION,
    hotwater_temp BIGINT,
    brine_out_temp BIGINT,
    brine_in_temp BIGINT,
    integral BIGINT,
    flowlinepump_speed BIGINT,
    brinepump_speed BIGINT,
    runtime_compressor BIGINT,
    runtime_hotwater BIGINT,
    runtime_3kw BIGINT,
    runtime_6kw BIGINT,
    brinepump_on BOOLEAN,
    compressor_on BOOLEAN,
    flowlinepump_on BOOLEAN,
    hotwater_production BOOLEAN,
    circulation_pump BOOLEAN,
    aux_heater_3kw_on BOOLEAN,
    aux_heater_6kw_on BOOLEAN
);
```

If TimescaleDB is available, the table is automatically converted to a hypertable.

