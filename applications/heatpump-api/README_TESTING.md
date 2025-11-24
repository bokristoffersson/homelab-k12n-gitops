# Testing Guide

## Running Tests Locally with Docker

### Quick Start

1. Start PostgreSQL in Docker:
```bash
# Use 'docker compose' (newer) or 'docker-compose' (older)
docker compose -f docker-compose.test.yml up -d
# OR
docker-compose -f docker-compose.test.yml up -d
```

2. Wait for PostgreSQL to be ready, then run tests:
```bash
export DATABASE_URL="postgresql://testuser:testpass@localhost:5433/testdb"
cargo test --test integration_test -- --test-threads=1 --nocapture
```

3. Stop PostgreSQL:
```bash
docker compose -f docker-compose.test.yml down
# OR
docker-compose -f docker-compose.test.yml down
```

### Using the Test Script

Alternatively, use the provided script:
```bash
./run-tests.sh
```

This script will:
- Start PostgreSQL in Docker
- Wait for it to be ready
- Run the integration tests sequentially
- Stop PostgreSQL when done

## Test Isolation

Tests run sequentially (`--test-threads=1`) to avoid interference. Each test:
1. Sets up the database schema
2. Cleans up any existing data
3. Inserts its own test data
4. Runs the test
5. Data is cleaned up for the next test

## Debugging Failed Tests

If a test fails, check:
1. Is PostgreSQL running and accessible?
2. Are there leftover records from previous test runs?
3. Check the error messages - they now include data samples

To debug a specific test:
```bash
cargo test --test integration_test test_service_list_with_filters -- --test-threads=1 --nocapture
```

## Database Connection

Default connection (if DATABASE_URL is not set):
- Host: localhost
- Port: 5432 (or 5433 for Docker)
- User: testuser
- Password: testpass
- Database: testdb

To use a different database:
```bash
export DATABASE_URL="postgresql://user:password@host:port/database"
```

