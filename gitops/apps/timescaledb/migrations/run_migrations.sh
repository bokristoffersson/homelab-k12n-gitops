#!/bin/bash
set -e

echo "Starting database migrations..."

# Database connection settings
export PGHOST="${PGHOST:-timescaledb}"
export PGPORT="${PGPORT:-5432}"
export PGUSER="${PGUSER:-postgres}"
export PGPASSWORD="${PGPASSWORD:-postgres}"
export PGDATABASE="${PGDATABASE:-postgres}"

# Wait for PostgreSQL to be ready
until psql -c '\q' 2>/dev/null; do
  echo "Waiting for PostgreSQL to be ready..."
  sleep 2
done

echo "PostgreSQL is ready. Running migrations..."

# Run each migration file in order
for migration_file in /migrations/*.sql; do
  if [ -f "$migration_file" ]; then
    migration_name=$(basename "$migration_file")
    echo "Applying migration: $migration_name"
    psql -f "$migration_file"
    echo "Migration $migration_name applied successfully"
  fi
done

echo "All migrations completed successfully!"
