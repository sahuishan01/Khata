#!/data/data/com.termux/files/usr/bin/bash
set -e
PGDATA="$HOME/opencode-projects/khata/.pgdata"
PGPORT=5433
PGLOG="$PGDATA/pg.log"

if [ -d "$PGDATA" ]; then
  echo "Already initialized at $PGDATA. Delete it to reinit."
  exit 1
fi

initdb -D "$PGDATA" --no-locale --encoding=UTF8

echo "port = $PGPORT" >> "$PGDATA/postgresql.conf"
echo "listen_addresses = '127.0.0.1'" >> "$PGDATA/postgresql.conf"

# Append scram-sha-256 auth for app roles
cat >> "$PGDATA/pg_hba.conf" <<'EOF'
host    khata    khata      127.0.0.1/32    scram-sha-256
host    khata    khata_ro   127.0.0.1/32    scram-sha-256
EOF

pg_ctl -D "$PGDATA" -l "$PGLOG" start
sleep 2

psql -p "$PGPORT" -d postgres -c "CREATE ROLE khata LOGIN PASSWORD 'khata';"
psql -p "$PGPORT" -d postgres -c "CREATE ROLE khata_ro LOGIN PASSWORD 'khata_ro';"
psql -p "$PGPORT" -d postgres -c "CREATE DATABASE khata OWNER khata;"
psql -p "$PGPORT" -d khata -c "CREATE EXTENSION IF NOT EXISTS citext;"

pg_ctl -D "$PGDATA" stop
echo "Done. Run scripts/pg_start.sh to start Postgres."
