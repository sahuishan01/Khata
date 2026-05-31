#!/data/data/com.termux/files/usr/bin/bash
PGDATA="$HOME/opencode-projects/khata/.pgdata"
PGLOG="$PGDATA/pg.log"
pg_ctl -D "$PGDATA" -l "$PGLOG" start
