#!/data/data/com.termux/files/usr/bin/bash
set -e
PGDATA="$HOME/opencode-projects/khata/.pgdata"
PGPORT=5433
PGLOG="$PGDATA/pg.log"

if [ -d "$PGDATA" ]; then
  echo "Already initialized at $PGDATA. Delete it to reinit."
  exit 1
fi

# peer for local (socket) connections, scram-sha-256 for TCP — never trust.
initdb -D "$PGDATA" --no-locale --encoding=UTF8 --auth-local=peer --auth-host=scram-sha-256

echo "port = $PGPORT" >> "$PGDATA/postgresql.conf"
echo "listen_addresses = '127.0.0.1'" >> "$PGDATA/postgresql.conf"
# Hash role passwords with SCRAM (must be set before the roles are created).
echo "password_encryption = 'scram-sha-256'" >> "$PGDATA/postgresql.conf"

# Replace pg_hba.conf with an explicit least-privilege ruleset.
# pg_hba is first-match: an appended tightening rule is shadowed by initdb's
# default trust lines, so the whole file is rewritten (cat >, not cat >>).
cat > "$PGDATA/pg_hba.conf" <<'EOF'
# TYPE  DATABASE  USER      ADDRESS         METHOD
local   all       all                       peer
host    all       khata     127.0.0.1/32    scram-sha-256
host    all       khata_ro  127.0.0.1/32    scram-sha-256
host    all       khata     ::1/128         scram-sha-256
host    all       khata_ro  ::1/128         scram-sha-256
host    all       all       0.0.0.0/0       reject
host    all       all       ::0/0           reject
EOF

pg_ctl -D "$PGDATA" -l "$PGLOG" start
sleep 2

psql -p "$PGPORT" -d postgres -c "CREATE ROLE khata LOGIN PASSWORD 'khata';"
psql -p "$PGPORT" -d postgres -c "CREATE ROLE khata_ro LOGIN PASSWORD 'khata_ro';"
psql -p "$PGPORT" -d postgres -c "CREATE DATABASE khata OWNER khata;"
psql -p "$PGPORT" -d khata -c "CREATE EXTENSION IF NOT EXISTS citext;"

pg_ctl -D "$PGDATA" stop
echo "Done. Run scripts/pg_start.sh to start Postgres."
