# Postgres Setup — khata

Everything about running the Postgres instance that backs khata, specifically on Termux (Android, no root).

---

## Instance details

| Setting | Value |
|---------|-------|
| Data directory | `~/opencode-projects/khata/.pgdata` |
| Port | **5433** (not the default 5432 — avoids system conflicts) |
| Listen address | `127.0.0.1` only |
| Log file | `~/.pgdata/pg.log` |
| Database | `khata` |
| App role | `khata` / password `khata` |
| Read-only role | `khata_ro` / password `khata_ro` |

The DATABASE_URL in `.env` must use **5433**:
```
DATABASE_URL=postgresql://khata:khata@127.0.0.1:5433/khata
RO_DATABASE_URL=postgresql://khata_ro:khata_ro@127.0.0.1:5433/khata
```

---

## First-time setup

Run once after cloning:

```bash
bash scripts/pg_init.sh
```

This does:
1. `initdb` into `.pgdata` with UTF-8, no locale
2. Sets `port = 5433` and `listen_addresses = '127.0.0.1'` in `postgresql.conf`
3. Appends `scram-sha-256` auth entries for both roles to `pg_hba.conf`
4. Creates roles `khata` and `khata_ro`, database `khata`, extension `citext`

Then run migrations:
```bash
cd backend
sqlx migrate run
```

---

## Daily use

### Start
```bash
bash scripts/pg_start.sh
```
The script auto-removes a stale `postmaster.pid` if the process died (common on Termux when the screen locks or Android kills background processes).

### Stop
```bash
bash scripts/pg_stop.sh
```

### Status
```bash
pg_ctl -D ~/opencode-projects/khata/.pgdata status
```

### Connect (quick check)
```bash
psql -h 127.0.0.1 -p 5433 -U khata -d khata
```

---

## Why Postgres keeps dying on Termux

Android's memory manager (LMKD) kills background processes when:
- The screen locks
- Memory pressure is high
- Battery saver mode activates

**Symptoms**: `postmaster.pid` exists but PID is gone; `pg_ctl status` says "no server running"; connection refused on 5433.

**Fix**:
```bash
bash scripts/pg_start.sh   # handles stale pid automatically
```

**Prevention** (optional): Acquire a Termux wake lock before a long session:
```
Termux app → long-press the notification → "Acquire wakelock"
```
Or in shell: `termux-wake-lock` (requires Termux:API app).

---

## Row-Level Security

The `transactions` table has `FORCE ROW LEVEL SECURITY`. Every query must be inside a transaction with:
```sql
SET LOCAL app.current_user_id = '<uuid>';
```
Without this, **all queries return 0 rows** — even as the table owner.

Policy:
```sql
USING (user_id = current_setting('app.current_user_id', true)::uuid)
```

This is why the backend wraps every DB operation in a transaction that sets `app.current_user_id` first.

**To query as superuser (bypasses RLS)** — useful for debugging:
```bash
psql -h 127.0.0.1 -p 5433 -U u0_a467 -d khata -c "SELECT COUNT(*) FROM transactions;"
# Replace u0_a467 with your actual Unix username (run: whoami)
```

---

## Roles and permissions

```sql
-- khata: table owner, full CRUD, migrations
GRANT ALL ON ALL TABLES IN SCHEMA public TO khata;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO khata;
ALTER ROLE khata CREATEDB;  -- needed for sqlx::test

-- khata_ro: SELECT only, used for Claude text-to-SQL queries
GRANT SELECT ON transactions TO khata_ro;
-- RLS still applies to khata_ro — it can only see rows for the set current_user_id
```

---

## Migrations

Location: `backend/migrations/`

Run with:
```bash
cd backend && sqlx migrate run
```

Revert last:
```bash
cd backend && sqlx migrate revert
```

Check status:
```bash
cd backend && sqlx migrate info
```

All migrations are recorded in `_sqlx_migrations` table.

---

## Backup

Quick dump:
```bash
pg_dump -h 127.0.0.1 -p 5433 -U khata khata > khata_backup_$(date +%Y%m%d).sql
```

Restore:
```bash
psql -h 127.0.0.1 -p 5433 -U khata -d khata < khata_backup_YYYYMMDD.sql
```

---

## Troubleshooting

### "connection refused" on port 5433
```bash
bash scripts/pg_start.sh
```

### "could not connect to server: No such file or directory"
The socket path is wrong — use TCP (`-h 127.0.0.1 -p 5433`) not the Unix socket.

### "FATAL: role does not exist"
The `khata` or `khata_ro` role wasn't created. Re-run the relevant parts of `pg_init.sh` manually:
```bash
psql -h 127.0.0.1 -p 5433 -d postgres -c "CREATE ROLE khata LOGIN PASSWORD 'khata';"
psql -h 127.0.0.1 -p 5433 -d postgres -c "CREATE ROLE khata_ro LOGIN PASSWORD 'khata_ro';"
```

### SELECT returns 0 rows when data exists
RLS is active without `app.current_user_id` being set. Expected — the backend handles this. To bypass for debugging, connect as your Unix user (see "RLS" section above).

### sqlx::test panics — "must be owner of database"
The `khata` role needs CREATEDB:
```bash
psql -h 127.0.0.1 -p 5433 -d postgres -c "ALTER ROLE khata CREATEDB;"
```

### Checking what's in the DB (ignoring RLS)
```bash
# as your Unix superuser role
psql -h 127.0.0.1 -p 5433 -U $(whoami) -d khata
\dt              -- list tables
SELECT COUNT(*) FROM transactions;
SELECT bank, COUNT(*) FROM transactions GROUP BY bank;
```
