#!/bin/bash

# Try docker compose (newer) first, fall back to docker-compose (older)
if command -v docker &> /dev/null && docker compose version &> /dev/null 2>&1; then
    DOCKER_COMPOSE="docker compose"
elif command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
else
    echo "Error: Neither 'docker compose' nor 'docker-compose' found"
    exit 1
fi

# Start PostgreSQL in Docker
echo "Starting PostgreSQL in Docker..."
$DOCKER_COMPOSE -f docker-compose.test.yml up -d

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
until $DOCKER_COMPOSE -f docker-compose.test.yml exec -T postgres pg_isready -U testuser -d testdb > /dev/null 2>&1; do
  echo "Waiting for PostgreSQL..."
  sleep 1
done

echo "PostgreSQL is ready!"

# Set database URL
export DATABASE_URL="postgresql://testuser:testpass@localhost:5433/testdb"

# Run tests sequentially to avoid interference
echo "Running integration tests..."
cargo test --test integration_test -- --test-threads=1 --nocapture

# Capture exit code
TEST_EXIT_CODE=$?

# Stop PostgreSQL
echo "Stopping PostgreSQL..."
$DOCKER_COMPOSE -f docker-compose.test.yml down

exit $TEST_EXIT_CODE
