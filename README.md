# Khata — Personal Finance Tracker

A self-hosted finance tracker that runs entirely on **Termux (Android, no root)**. Import Indian bank statements (CSV/Excel), get automatic transaction categorisation, an interactive dashboard, and ask Claude natural-language questions about your money.

---

## Features

- **Multi-user** with email/password login (JWT)
- **Bank statement import** — CSV and Excel (.xls / .xlsx)
  - Supported banks: HDFC, ICICI, SBI, Axis Bank, + generic fallback
  - Duplicate detection across re-uploads (SHA-256 file hash + transaction fingerprint)
- **Auto-categorisation** — 14+ categories (Food & Dining, Transport, Utilities, …) with Miscellaneous fallback
- **Category editing** — change one transaction's category or bulk-apply to all similar ones; create new categories
- **Dashboard** — spend vs earn chart, category donut, savings rate, month-over-month comparison, top spending
- **Transactions list** — paginated, with inline category editor
- **Claude Q&A** — ask plain-English questions answered by SQL generated from the native `claude` CLI (no API key — uses OAuth)
- **Row-Level Security** — Postgres RLS ensures one user can never see another's data, even in Claude-generated SQL

---

## Prerequisites

Install via Termux (`pkg install <name>`):

| Tool | Version | Install |
|------|---------|---------|
| Rust + cargo | ≥ 1.95 | `pkg install rust` |
| Node.js | ≥ 18 | `pkg install nodejs` |
| PostgreSQL | ≥ 16 | `pkg install postgresql` |
| Claude CLI | any | pre-installed / `npm i -g @anthropic-ai/claude-code` |

Verify:
```bash
rustc --version     # rustc 1.95.x
node --version      # v26.x
psql --version      # PostgreSQL 18.x
claude --version
```

---

## Setup (first time)

### 1. Clone / copy the project

```bash
cd ~/opencode-projects
# project lives at ~/opencode-projects/khata/
```

### 2. Initialise Postgres

```bash
cd ~/opencode-projects/khata
scripts/pg_init.sh
```

This runs `initdb`, creates roles `khata` / `khata_ro`, creates database `khata`, enables `citext`, and stops Postgres. Only needed once.

### 3. Configure environment

```bash
cp .env.example .env
# Edit .env — at minimum change JWT_SECRET to a long random string
```

`.env` variables:

| Variable | Default | Notes |
|----------|---------|-------|
| `DATABASE_URL` | `postgresql://khata:khata@127.0.0.1:5433/khata` | Main app pool |
| `RO_DATABASE_URL` | `postgresql://khata_ro:khata_ro@127.0.0.1:5433/khata` | Read-only pool for Claude queries |
| `JWT_SECRET` | *(change this)* | Random 32+ char string |
| `CLAUDE_BIN` | `claude` | Path to `claude` CLI if not in PATH |
| `BIND_ADDR` | `127.0.0.1:8090` | Backend listen address (8080 may be taken) |
| `CORS_ORIGINS` | `http://localhost:5173` | Comma-separated allowed origins |
| `RUST_LOG` | `info` | Log level (`warn` for quieter output) |

### 4. Install frontend dependencies

```bash
cd ~/opencode-projects/khata/frontend
npm install
```

### 5. Log in to Claude CLI (for Q&A feature)

```bash
claude   # follow OAuth login prompt
```

---

## Running

### Development (two terminals)

**Terminal 1 — Backend**
```bash
cd ~/opencode-projects/khata
scripts/pg_start.sh      # start Postgres (skip if already running)

cd backend
cargo run                # auto-loads ../.env, runs migrations on start
```

**Terminal 2 — Frontend**
```bash
cd ~/opencode-projects/khata/frontend
npm run dev
```

Open **http://localhost:5173** in your browser.

### Single command (bash)

```bash
cd ~/opencode-projects/khata
scripts/dev.sh
```

> **Fish shell note:** `scripts/dev.sh` has a bash shebang and works from fish. The backend also auto-loads `../.env` via `dotenvy`, so no manual `source` is needed.

---

## Cloudflare Tunnel (remote access)

```bash
pkg install cloudflared
# In .env, change:
#   BIND_ADDR=0.0.0.0:8090
#   CORS_ORIGINS=https://your-tunnel.trycloudflare.com

scripts/tunnel.sh   # starts cloudflared tunnel on port 8090
```

---

## Architecture

```
khata/
  backend/          Rust (Axum 0.7 + sqlx 0.8 + tokio)
    src/
      auth/         register, login, JWT middleware
      ingest/       file upload → parse → normalise → categorise → dedup → store
        profiles/   per-bank column-name mappings (HDFC, ICICI, SBI, Axis, Generic)
        categorize  keyword-based auto-categorisation engine
      txns/         list, dashboard, analysis, category editor endpoints
      chat/         text-to-SQL via claude CLI, SQL validator, RLS-safe execution
    migrations/     sqlx SQL migrations (run automatically on startup)
  frontend/         React 18 + Vite + TypeScript
    src/
      pages/        Dashboard, Transactions, Chat, Login, Register
      components/   FileUpload, CategoryEditor, SpendEarnChart, CategoryChart
      store/        Zustand auth store
      api/          Axios client with JWT interceptor
  scripts/
    pg_init.sh      One-time Postgres setup
    pg_start.sh     Start Postgres
    pg_stop.sh      Stop Postgres
    dev.sh          Start everything (bash)
    tunnel.sh       Cloudflare Tunnel
```

### Security model

- **RLS + FORCE** on `transactions`, `statements`, `chat_messages` — even the table owner can't read other users' rows without `app.current_user_id` set
- **Two DB roles**: `khata` (app, writes), `khata_ro` (Claude queries — SELECT only)
- **SQL validator**: Claude-generated SQL is parsed with `sqlparser`, must be a single SELECT, keyword-blocklist rejects DML/DDL
- **Statement-timeout**: 5 s per Claude query

### Dedup fingerprint

- If bank reference number present → `sha256(user_id + account_label + bank_ref + amount)`
- Else → `sha256(user_id + account_label + value_date + amount + direction + balance + normalised_narration)`
- Stored as `UNIQUE(user_id, fingerprint)` — `ON CONFLICT DO NOTHING`

---

## Supported Bank Formats

| Bank | Detection | Date formats |
|------|-----------|-------------|
| HDFC | "hdfc" in file header | dd/mm/yy, dd/mm/yyyy, dd-mm-yyyy, dd-mon-yyyy |
| ICICI | "icici" in file header | dd/mm/yyyy, dd-mm-yyyy, dd mon yyyy |
| SBI | "sbi" / "state bank" in header | dd mon yyyy, dd/mm/yyyy |
| Axis Bank | "axis" in header | dd-mm-yyyy, dd/mm/yyyy, dd-mon-yyyy |
| Generic | fallback | All of the above |

To add a new bank: create `backend/src/ingest/profiles/<bank>.rs` following the existing profiles, and add it to `registry()` in `profiles/mod.rs`.

---

## Database

- **Port**: 5433 (avoids conflicts with any system Postgres on 5432)
- **Data dir**: `~/opencode-projects/khata/.pgdata`
- **Roles**: `khata` (owner, CREATEDB for tests), `khata_ro` (SELECT only)
- **Migrations**: managed by sqlx, run automatically at startup

Manual psql access:
```bash
psql -p 5433 -h 127.0.0.1 -U khata -d khata
```

---

## Tests

```bash
cd ~/opencode-projects/khata/backend
DATABASE_URL=postgresql://khata:khata@127.0.0.1:5433/khata cargo test
# 30 tests: auth, dedup, fingerprint, normalise, RLS isolation, SQL validator, claude_cli
```

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `Address already in use` on 8090 | Another process is on that port; change `BIND_ADDR` in `.env` |
| `404` on API calls | Backend not running, or Vite proxy points to wrong port (check `vite.config.ts`) |
| `0 normalized` after upload | Date/amount columns not recognised — use `/api/ingest/debug-headers` endpoint |
| Dashboard shows ₹0 | Re-check that upload showed `normalized > 0`; if statement was already imported, delete its record in psql and re-upload |
| Chat returns plain text | `claude` CLI not logged in — run `claude` interactively to authenticate |
