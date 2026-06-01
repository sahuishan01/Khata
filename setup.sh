#!/usr/bin/env bash
# Khata — one-shot setup + start
# Run from anywhere: bash /path/to/khata/setup.sh
# Subsequent runs: same command — skips steps already done.

set -e

# ── Resolve project root regardless of where the script is called from ─────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT="$SCRIPT_DIR"
PGDATA="$PROJECT/.pgdata"
PGPORT=5433
PGLOG="$PGDATA/pg.log"
ENV_FILE="$PROJECT/.env"

# ── Colours ─────────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✓${NC} $1"; }
info() { echo -e "${CYAN}→${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
die()  { echo -e "${RED}✗ $1${NC}" >&2; exit 1; }

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Khata — Finance Tracker"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ── 1. Prerequisites ─────────────────────────────────────────────────────────────
info "Checking prerequisites..."

command -v cargo  > /dev/null 2>&1 || die "cargo not found. Install Rust: https://rustup.rs"
command -v psql   > /dev/null 2>&1 || die "psql not found. Install PostgreSQL."
command -v pg_ctl > /dev/null 2>&1 || die "pg_ctl not found. Install PostgreSQL."
command -v node   > /dev/null 2>&1 || die "node not found. Install Node.js >= 18."
command -v npm    > /dev/null 2>&1 || die "npm not found."
command -v claude > /dev/null 2>&1 || warn "claude CLI not found — chat Q&A will not work. Install with: npm i -g @anthropic-ai/claude-code"

ok "Prerequisites OK"

# ── 2. .env ──────────────────────────────────────────────────────────────────────
if [ ! -f "$ENV_FILE" ]; then
    info "Creating .env from .env.example..."
    cp "$PROJECT/.env.example" "$ENV_FILE"
    ok ".env created — review it if you need custom settings"
else
    ok ".env already exists"
fi

# ── 3. sqlx CLI ─────────────────────────────────────────────────────────────────
if ! command -v sqlx > /dev/null 2>&1; then
    info "Installing sqlx-cli (needed for migrations)..."
    cargo install sqlx-cli --no-default-features --features rustls,postgres
    ok "sqlx-cli installed"
else
    ok "sqlx-cli present"
fi

# ── 4. Postgres init (first run only) ────────────────────────────────────────────
if [ ! -d "$PGDATA" ]; then
    info "Initialising Postgres cluster at .pgdata..."
    initdb -D "$PGDATA" --no-locale --encoding=UTF8 -A trust

    # Set port and listen address
    echo "port = $PGPORT"              >> "$PGDATA/postgresql.conf"
    echo "listen_addresses = '127.0.0.1'" >> "$PGDATA/postgresql.conf"

    # Add password-auth entries for app roles (TCP connections from backend)
    cat >> "$PGDATA/pg_hba.conf" <<'EOF'
host    khata    khata      127.0.0.1/32    scram-sha-256
host    khata    khata_ro   127.0.0.1/32    scram-sha-256
EOF

    pg_ctl -D "$PGDATA" -l "$PGLOG" start
    sleep 2

    psql -p "$PGPORT" -d postgres -c "CREATE ROLE khata    LOGIN PASSWORD 'khata';"
    psql -p "$PGPORT" -d postgres -c "CREATE ROLE khata_ro LOGIN PASSWORD 'khata_ro';"
    psql -p "$PGPORT" -d postgres -c "ALTER  ROLE khata    CREATEDB;"
    psql -p "$PGPORT" -d postgres -c "CREATE DATABASE khata OWNER khata;"
    psql -p "$PGPORT" -d khata    -c "CREATE EXTENSION IF NOT EXISTS citext;"

    pg_ctl -D "$PGDATA" stop
    ok "Postgres cluster initialised"
else
    ok "Postgres cluster already exists"
fi

# ── 5. Start Postgres (handle stale PID) ─────────────────────────────────────────
PID_FILE="$PGDATA/postmaster.pid"
if [ -f "$PID_FILE" ]; then
    STORED_PID=$(head -1 "$PID_FILE")
    if ! kill -0 "$STORED_PID" 2>/dev/null; then
        warn "Removing stale postmaster.pid (PID $STORED_PID gone)"
        rm -f "$PID_FILE"
    fi
fi

if pg_ctl -D "$PGDATA" status > /dev/null 2>&1; then
    ok "Postgres already running on :$PGPORT"
else
    info "Starting Postgres..."
    pg_ctl -D "$PGDATA" -l "$PGLOG" start
    sleep 1
    ok "Postgres started on :$PGPORT"
fi

# ── 6. Migrations ────────────────────────────────────────────────────────────────
info "Running database migrations..."
cd "$PROJECT/backend"
# Source .env so DATABASE_URL is available
set -a; [ -f "$ENV_FILE" ] && source "$ENV_FILE"; set +a
export DATABASE_URL="${DATABASE_URL:-postgresql://khata:khata@127.0.0.1:5433/khata}"
sqlx migrate run
ok "Migrations up to date"
cd "$PROJECT"

# ── 7. RLS grants (idempotent) ────────────────────────────────────────────────────
info "Ensuring read-only role grants..."
psql "$DATABASE_URL" -c "
    GRANT USAGE ON SCHEMA public TO khata_ro;
    GRANT SELECT ON transactions TO khata_ro;
" > /dev/null 2>&1 || true
ok "Grants OK"

# ── 8. Frontend dependencies ─────────────────────────────────────────────────────
if [ ! -d "$PROJECT/frontend/node_modules" ]; then
    info "Installing frontend dependencies (npm install)..."
    cd "$PROJECT/frontend" && npm install
    cd "$PROJECT"
    ok "Frontend dependencies installed"
else
    ok "Frontend node_modules present"
fi

# ── 9. Start backend + frontend ──────────────────────────────────────────────────
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Starting Khata"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

set -a; [ -f "$ENV_FILE" ] && source "$ENV_FILE"; set +a
export DATABASE_URL="${DATABASE_URL:-postgresql://khata:khata@127.0.0.1:5433/khata}"
export RO_DATABASE_URL="${RO_DATABASE_URL:-postgresql://khata_ro:khata_ro@127.0.0.1:5433/khata}"
export JWT_SECRET="${JWT_SECRET:-change-me-in-.env}"
export BIND_ADDR="${BIND_ADDR:-127.0.0.1:8090}"
export RUST_LOG="${RUST_LOG:-info}"

info "Starting backend on $BIND_ADDR ..."
cd "$PROJECT/backend" && cargo run &
BACKEND_PID=$!
cd "$PROJECT"

info "Starting frontend dev server..."
cd "$PROJECT/frontend" && npm run dev &
FRONTEND_PID=$!
cd "$PROJECT"

echo ""
ok "Khata is running!"
echo ""
echo "  Frontend: http://localhost:5173"
echo "  Backend:  http://${BIND_ADDR}"
echo ""
echo "  Press Ctrl+C to stop everything."
echo ""

cleanup() {
    echo ""
    info "Shutting down..."
    kill "$BACKEND_PID" "$FRONTEND_PID" 2>/dev/null || true
    wait 2>/dev/null
    echo "Done."
}
trap cleanup INT TERM
wait
