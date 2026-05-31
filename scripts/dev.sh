#!/data/data/com.termux/files/usr/bin/bash
set -e
cd "$(dirname "$0")/.."
PROJECT="$(pwd)"

# Start Postgres if not already running
pg_ctl -D "$PROJECT/.pgdata" status > /dev/null 2>&1 || scripts/pg_start.sh
sleep 1

# Load env
set -a; [ -f .env ] && source .env; set +a

# Default env fallbacks
export DATABASE_URL="${DATABASE_URL:-postgresql://khata:khata@127.0.0.1:5433/khata}"
export RO_DATABASE_URL="${RO_DATABASE_URL:-postgresql://khata_ro:khata_ro@127.0.0.1:5433/khata}"
export JWT_SECRET="${JWT_SECRET:-change-me-in-.env}"
export BIND_ADDR="${BIND_ADDR:-127.0.0.1:8090}"
export RUST_LOG="${RUST_LOG:-info}"

echo "Starting backend on $BIND_ADDR ..."
cd backend && cargo run &
BACKEND_PID=$!
cd "$PROJECT"

echo "Starting frontend dev server ..."
cd frontend && npm run dev &
FRONTEND_PID=$!
cd "$PROJECT"

echo ""
echo "Khata running:"
echo "  Backend:  http://$BIND_ADDR"
echo "  Frontend: http://localhost:5173"
echo ""
echo "Press Ctrl+C to stop."

cleanup() {
  echo "Stopping..."
  kill "$BACKEND_PID" "$FRONTEND_PID" 2>/dev/null || true
  wait 2>/dev/null
}
trap cleanup INT TERM
wait
