#!/bin/sh
set -e

echo "Starting database migrations..."

# Database connection settings
export PGHOST="${PGHOST:-postgres}"
export PGPORT="${PGPORT:-5432}"
export PGUSER="${PGUSER:-postgres}"
export PGPASSWORD="${PGPASSWORD:-postgres}"
export PGDATABASE="${PGDATABASE:-postgres}"

# Wait for PostgreSQL to be ready
until psql -c '\q' 2>/dev/null; do
  echo "Waiting for PostgreSQL to be ready..."
  sleep 2
done

echo "PostgreSQL is ready."

# Ensure the gasa database exists (CREATE DATABASE cannot run inside a
# transaction or conditionally in plain SQL, so it is handled here)
if ! psql -tAc "SELECT 1 FROM pg_database WHERE datname = 'gasa'" | grep -q 1; then
  echo "Creating database gasa..."
  psql -c "CREATE DATABASE gasa"
fi

echo "Running migrations..."

# Run each migration file in order against the gasa database.
# ON_ERROR_STOP makes a failing statement fail the Job instead of
# being silently skipped.
for migration_file in /migrations/*.sql; do
  if [ -f "$migration_file" ]; then
    migration_name=$(basename "$migration_file")
    echo "Applying migration: $migration_name"
    PGDATABASE=gasa psql -v ON_ERROR_STOP=1 -f "$migration_file"
    echo "Migration $migration_name applied successfully"
  fi
done

echo "All migrations completed successfully!"
