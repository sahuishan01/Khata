# 📒 Khata: Personal Finance Tracker

![Rust](https://img.shields.io/badge/backend-Rust%20%2B%20Axum-DEA584?logo=rust&logoColor=white)
![React](https://img.shields.io/badge/web-React%2019%20%2B%20Vite-61DAFB?logo=react&logoColor=white)
![Android](https://img.shields.io/badge/Android-Kotlin%20%2B%20Compose-3DDC84?logo=android&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-RLS-4169E1?logo=postgresql&logoColor=white)
![Self-hosted](https://img.shields.io/badge/self--hosted-yes-2ea44f)

A self-hosted personal finance tracker for Indian bank accounts and cards. Import bank statements (CSV, Excel and PDF), pull them in from email, auto-capture transactions from bank SMS on your phone, and see where your money goes on the web or in the Android app. You can also ask Claude plain-English questions about your spending. Your data stays on your own server, and each user's rows are isolated by PostgreSQL row-level security.

---

## ✨ Features

| | Feature |
|---|---|
| 📥 | **Statement import.** CSV, Excel (`.xls`, `.xlsx`) and PDF statements. Profiles for HDFC, ICICI, SBI and Axis, a generic bank fallback and a generic credit-card profile. Duplicates are caught across re-uploads. |
| 📧 | **Email ingestion.** A per-user IMAP sync picks up statement attachments, including password-protected PDFs, and transaction emails. |
| 💬 | **SMS auto-capture on Android.** Bank SMS are parsed on the device, and only recognized transactions are uploaded. OTPs, offers, payment requests and future debits are ignored. The original text is kept in Notes. |
| 🏷️ | **Auto-categorization.** Built-in categories with a Miscellaneous fallback, editable per transaction or in bulk, custom categories, and rules that apply to matching transactions. |
| 📊 | **Dashboard and analytics.** Spend vs earn, category breakdown, savings rate, month-over-month comparison, top spending, configurable analytics layout and reports. |
| 💳 | **Cards and transfers.** Card repayments and transfers between your own accounts are reconciled, so they don't inflate your totals. |
| 🎯 | **Budgets, goals and subscriptions,** plus an investment portfolio view. |
| ✏️ | **Edit anything.** Titles, notes, categories and manual entries, with an audit log. |
| 🤖 | **Ask Claude.** Questions are turned into SQL by the `claude` CLI (OAuth, no API key), checked by a validator and run read-only. |
| 👥 | **Multi-user.** Email and password login, one admin who can manage users, forced password reset, and cookie-based sessions that can be revoked. |
| 📱 | **Android extras.** Offline-first local database with sync, home-screen widgets, quick-settings tiles and app shortcuts for Add transaction, Transaction history, Chat and Add data. |
| 🌗 | **"Bahi by lamplight" design** with a warm parchment palette, shared by web and Android. |

---

## 🗺️ How it fits together

```mermaid
flowchart LR
    subgraph Inputs
        F["📄 CSV / Excel / PDF<br/>statements"]
        E["📧 Email (IMAP)"]
        S["📱 Bank SMS<br/>parsed on-device"]
        M["✍️ Manual entry"]
    end
    F & E & M --> API
    S -- "recognized transactions only" --> AND["🤖 Android app<br/>Room DB + sync"]
    AND <--> API
    W["🌐 Web app<br/>React"] <--> API["🦀 Khata backend<br/>Axum, JWT cookies"]
    API --> DB[("🐘 PostgreSQL<br/>row-level security")]
    API -- "read-only role<br/>validated SELECT" --> CL["🧠 claude CLI<br/>text to SQL"]
    CL --> DB
```

### 🛡️ Security model

- **Row-level security with FORCE** on user tables, so even the table owner can't read another user's rows without the user context set.
- **Two database roles:** `khata` (the app, writes) and `khata_ro` (Claude queries, SELECT only).
- **SQL validator** for Claude-generated queries: a single `SELECT` only, with a blocklist for DML, DDL and session functions, and a 5 second statement timeout.
- **Auth hardening:** login rate limits, short-lived revocable tokens, HttpOnly cookies on the web, encrypted token storage on Android, and secure cookies by default.
- **First-run admin setup** is allowed only on a loopback bind unless you explicitly allow it.

See [SECURITY.md](SECURITY.md) for the full policy and fixes.

### 🔁 Duplicate detection

- With a bank reference number: `sha256(user_id + account + bank_ref + amount)`.
- Without one: a hash of user, account, date, amount, direction, balance and the normalized narration.
- Stored as `UNIQUE(user_id, fingerprint)` and inserted with `ON CONFLICT DO NOTHING`.

---

## 🚀 Getting started (development)

The backend, frontend and database all run on a normal Linux box or on **Termux (Android, no root)**.

| Tool | Version |
|---|---|
| Rust and cargo | 1.95 or newer |
| Node.js | 18 or newer |
| PostgreSQL | 16 or newer |
| Claude CLI | any (only for the chat feature) |

```bash
# 1. one-time database setup (initdb, roles khata and khata_ro, citext)
scripts/pg_init.sh

# 2. configure
cp .env.example .env        # at minimum set JWT_SECRET to a long random string

# 3. frontend dependencies
cd frontend && npm install && cd ..

# 4. log in to Claude for the chat feature
claude

# 5. run everything
scripts/dev.sh              # Postgres, backend and frontend together
```

Open **http://localhost:5173**. The first visit creates the admin account. Migrations run automatically at startup. To run the pieces separately: `scripts/pg_start.sh`, then `cd backend && cargo run`, then `cd frontend && npm run dev`.

### ⚙️ Configuration

| Variable | Default | Notes |
|---|---|---|
| `DATABASE_URL` | local Postgres on port 5433 | Main app pool |
| `RO_DATABASE_URL` | same server, `khata_ro` role | Read-only pool for Claude queries |
| `JWT_SECRET` | none | A long random string. Placeholder or short values are rejected at startup. |
| `CLAUDE_BIN` | `claude` | Path to the CLI if it isn't on `PATH` |
| `BIND_ADDR` | `127.0.0.1:8090` | Backend listen address |
| `CORS_ORIGINS` | `http://localhost:5173` | Comma-separated allowed origins. Empty means none allowed. |
| `COOKIE_SECURE` | `true` | Set to `false` only for plain-http local development |
| `EMAIL_SYNC_POLL_SECS` | `900` | Email poll interval. `0` turns the poll off. |

More options, such as the email sync limits and remote first-run setup, are documented in [.env.example](.env.example). Password-protected statement PDFs need `qpdf` on the host.

For remote access from Termux, `scripts/tunnel.sh` starts a Cloudflare Tunnel.

---

## 🧪 Tests

```bash
cd backend
DATABASE_URL=postgresql://khata:khata@127.0.0.1:5433/khata cargo test
```

The backend suite covers auth, deduplication and fingerprints, normalization, row-level-security isolation, the SQL validator, PDF parsing, SMS decisions, card repayments and transaction edits. The Android app has its own unit and instrumented tests. CI ([workflows](.github/workflows)) runs backend tests and Clippy, builds the signed Android release, and runs security scans. See [docs/self-hosted-ci.md](docs/self-hosted-ci.md).

---

## 📦 Deployment

- **Docker:** the [Dockerfile](Dockerfile) builds a small non-root image (user 1000) with a pinned pdfium for PDF statements. It listens on port 8090.
- **Behind a proxy:** serve the built web app as static files and proxy `/api/*` to the backend, with TLS terminated at the proxy. [DEPLOYMENT.md](DEPLOYMENT.md) describes one working VPS setup, and [POSTGRES.md](POSTGRES.md) covers the database.
- **Android:** CI builds a signed APK on each tagged release. Install updates over the existing app.

---

## 📱 Android app

The app is Kotlin with Jetpack Compose. It keeps a local Room database and syncs with the backend, and it needs SMS permissions only for the auto-capture feature. Go to **Add Data → Scan SMS inbox** to import existing messages. Details:

- [docs/sms-local-import.md](docs/sms-local-import.md), [docs/sms-posting-grammar.md](docs/sms-posting-grammar.md) and the request and promotion filtering notes for how messages are recognized
- [docs/android-shortcuts.md](docs/android-shortcuts.md) for widgets, tiles and shortcuts
- [docs/transaction-editing.md](docs/transaction-editing.md), [docs/transaction-titles.md](docs/transaction-titles.md) and [docs/transaction-identity.md](docs/transaction-identity.md)

---

## 🗂️ Project layout

```
backend/    Rust (Axum, sqlx): auth, ingest (profiles, PDF, email), txns, budgets, goals,
            rules, reports, portfolio, chat (text to SQL), SMS, audit; sqlx migrations
frontend/   React + Vite + TypeScript web app (pages, charts, Zustand store)
android/    Kotlin + Compose app (data and sync, SMS, UI, shortcuts)
scripts/    Postgres setup, dev launcher, Cloudflare tunnel
docs/       Feature and design notes
```

To add a bank, create `backend/src/ingest/profiles/<bank>.rs` following the existing ones and register it in `profiles/mod.rs`.

---

## 🩹 Troubleshooting

| Problem | Fix |
|---|---|
| `Address already in use` on 8090 | Change `BIND_ADDR` in `.env` |
| `404` on API calls | The backend isn't running, or the Vite proxy points at the wrong port (check `vite.config.ts`) |
| Upload shows `0 normalized` | The date or amount columns weren't recognized. Check the statement's header row against the bank profile in `backend/src/ingest/profiles/` |
| Dashboard shows ₹0 | Check the upload reported `normalized > 0` |
| Chat returns plain text | The `claude` CLI isn't logged in. Run `claude` interactively once |

More history is in [CHANGELOG.md](CHANGELOG.md) and [HANDOFF.md](HANDOFF.md).
