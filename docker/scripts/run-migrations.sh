#!/bin/sh
set -eu

echo "============================================================"
echo " Macro Database Migrator (db-migrator)"
echo "============================================================"

# Default connection parameters
PGHOST="${PGHOST:-postgres}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-macro}"
PGDATABASE="${PGDATABASE:-macrodb}"
PGPASSWORD="${PGPASSWORD:-macro123456}"
export PGHOST PGPORT PGUSER PGDATABASE PGPASSWORD

echo "Target: ${PGUSER}@${PGHOST}:${PGPORT}/${PGDATABASE}"

# 1. Wait for PostgreSQL readiness
echo "Waiting for PostgreSQL to be ready..."
retries=30
while [ $retries -gt 0 ]; do
  if pg_isready -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" >/dev/null 2>&1; then
    echo "PostgreSQL is ready!"
    break
  fi
  retries=$((retries - 1))
  echo "Database not ready yet, waiting 2s... ($retries left)"
  sleep 2
done

if [ $retries -eq 0 ]; then
  echo "Error: Timed out waiting for PostgreSQL to become ready." >&2
  exit 1
fi

# 2. Acquire Advisory Lock to prevent concurrent migrations
LOCK_ID="718281828"
echo "Acquiring PostgreSQL advisory lock ($LOCK_ID)..."
psql -v ON_ERROR_STOP=1 -c "SELECT pg_advisory_lock($LOCK_ID);" >/dev/null
echo "Advisory lock acquired."

# Ensure lock is released on exit
cleanup() {
  echo "Releasing PostgreSQL advisory lock ($LOCK_ID)..."
  psql -c "SELECT pg_advisory_unlock($LOCK_ID);" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

# 3. Ensure SQLx migrations tracking table exists
psql -v ON_ERROR_STOP=1 <<'SCHEMA_EOF' >/dev/null
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
);
SCHEMA_EOF

# 4. Execute migrations
MIGRATIONS_DIR="${MIGRATIONS_DIR:-/migrations}"
echo "Running migrations from ${MIGRATIONS_DIR}..."

applied_count=0
skipped_count=0

for migration_file in $(ls "${MIGRATIONS_DIR}"/*.sql | grep -v '\.down\.sql$' | sort); do
  filename=$(basename "$migration_file")
  
  # Extract version: digits before the first underscore
  version=$(echo "$filename" | sed -E 's/^([0-9]+)_.*/\1/')
  description=$(echo "$filename" | sed -E 's/^[0-9]+_(.*)\.sql$/\1/' | tr '_' ' ')

  # Check if migration already succeeded
  is_applied=$(psql -t -A -c "SELECT 1 FROM _sqlx_migrations WHERE version = $version AND success = true;" 2>/dev/null || echo "")

  if [ "$is_applied" = "1" ]; then
    skipped_count=$((skipped_count + 1))
    continue
  fi

  echo "Applying migration [${version}] ${filename}..."
  start_time=$(date +%s%N 2>/dev/null || date +%s)

  # Calculate SHA384 checksum (used by sqlx)
  checksum_hex=$(sha384sum "$migration_file" | awk '{print $1}')

  # Run migration
  psql -v ON_ERROR_STOP=1 -f "$migration_file"

  end_time=$(date +%s%N 2>/dev/null || date +%s)
  duration=$(( (end_time - start_time) / 1000000 )) 2>/dev/null || duration=1

  # Record in _sqlx_migrations
  psql -v ON_ERROR_STOP=1 -c "
    INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
    VALUES ($version, '$description', now(), true, decode('$checksum_hex', 'hex'), $duration)
    ON CONFLICT (version) DO UPDATE SET success = true, installed_on = now();
  " >/dev/null

  applied_count=$((applied_count + 1))
done

echo "Migrations completed: ${applied_count} applied, ${skipped_count} skipped."

# 5. Bootstrap Super Administrator in macro_user when explicitly configured.
ADMIN_EMAIL="${ADMIN_EMAIL:-${MACRO_ADMIN_EMAIL:-}}"
if [ -z "${ADMIN_EMAIL}" ]; then
  echo "Skipping Super Administrator bootstrap (ADMIN_EMAIL is not configured)."
else
  echo "Ensuring Super Administrator exists (${ADMIN_EMAIL})..."

  psql -v ON_ERROR_STOP=1 -v admin_email="${ADMIN_EMAIL}" <<ADMIN_EOF >/dev/null
INSERT INTO "macro_user" ("id", "username", "email", "stripe_customer_id")
VALUES ('00000000-0000-0000-0000-000000000001'::uuid, 'admin', :'admin_email', 'cus_local_admin')
ON CONFLICT ("id") DO UPDATE
SET "username" = 'admin',
    "email" = EXCLUDED."email",
    "stripe_customer_id" = EXCLUDED."stripe_customer_id";
ADMIN_EOF
fi

echo "============================================================"
echo " Database initialization finished successfully!"
echo "============================================================"
exit 0
