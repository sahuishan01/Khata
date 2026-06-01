#!/data/data/com.termux/files/usr/bin/bash
set -e
PGDATA="$HOME/opencode-projects/khata/.pgdata"
PGLOG="$PGDATA/pg.log"
PID_FILE="$PGDATA/postmaster.pid"

# Remove stale pid file if the process is gone (Termux kills bg processes on lock)
if [ -f "$PID_FILE" ]; then
    PID=$(head -1 "$PID_FILE")
    if ! kill -0 "$PID" 2>/dev/null; then
        echo "Removing stale postmaster.pid (PID $PID no longer running)"
        rm -f "$PID_FILE"
    fi
fi

if pg_ctl -D "$PGDATA" status > /dev/null 2>&1; then
    echo "Postgres already running."
    exit 0
fi

pg_ctl -D "$PGDATA" -l "$PGLOG" start
echo "Postgres started on port 5433."
