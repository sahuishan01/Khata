#!/data/data/com.termux/files/usr/bin/bash
PGDATA="$HOME/opencode-projects/khata/.pgdata"
pg_ctl -D "$PGDATA" stop
