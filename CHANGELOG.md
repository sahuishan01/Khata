# Changelog

All notable changes to Khata are documented here.
Format: `## [date] — Summary` → bullet list of changes.

---

## [2026-05-31] — Auto-categorisation + Dashboard upgrade + Category editor

### Added
- **Auto-categorisation engine** (`ingest/categorize.rs`): keyword-based rules assign one of 14 categories on every import — Food & Dining, Groceries, Shopping, Transport, Travel, Fuel, Entertainment, Utilities, Health, Education, Insurance, Investment, Subscriptions, Transfer, Salary, Refund, ATM/Cash, EMI/Loan; fallback is **Miscellaneous**
- **`category` column** on `transactions` (migration `0007_add_category.sql`), indexed; all existing rows backfilled via SQL CASE
- **`GET /api/txns/analysis`** endpoint: category breakdown with %, savings rate, avg daily spend, month-over-month comparison, largest single expense
- **`GET /api/txns/categories`**: distinct category list for the logged-in user
- **`PUT /api/txns/:id/category`**: update a transaction's category with three scope options:
  - `single` — this transaction only
  - `same_description` — all transactions with identical description
  - `contains` — all descriptions containing a keyword (case-insensitive LIKE)
  - Also supports creating a new category by typing a new name
- **CategoryEditor component**: inline popover in Transactions page with scope radio buttons, keyword input, and optimistic UI update
- **Dashboard upgrades**: savings-rate card, avg-daily-spend card, transaction-count card, month-comparison banner (▲/▼ % vs last month), spending-by-category donut chart (Recharts PieChart), largest-expense card
- **Transactions page**: category badge is now an editable button, Category column replaces Bank column

### Changed
- `UploadResponse` now includes `normalized` count (rows that passed date+amount parsing) in addition to `rows_parsed`
- Upload warning shown in UI when `normalized = 0` with `rows_parsed > 0` (date/column parse failure)

---

## [2026-05-31] — Bank detection fix (preamble scanning) + Axis Bank profile

### Fixed
- **Bank detection now searches the full file** (preamble rows + column headers), not just the column-header row. HDFC statements were detected as GENERIC because "HDFC BANK Ltd." appears in account-info rows above the column headers, not in the header row itself.
- HDFC `ref_aliases` updated to include `"chq./ref.no."` (exact format in real statements)
- HDFC `date_formats` updated to include `"%d %b %Y"`

### Added
- **Axis Bank profile** (`profiles/axis.rs`): covers both old (`DR`/`CR`/`BAL`) and new (`Withdrawal (Dr)`/`Deposit (Cr)`) column formats, plus `"axis"` detect keyword
- **`GET /api/ingest/debug-headers`**: diagnostic endpoint — upload a file, get back detected bank, header row, column mappings, and `sample_normalized` count without actually importing data

---

## [2026-05-31] — Chat SQL generation fix + upload diagnostics

### Fixed
- **Chat Q&A**: `--output-format json` + `--json-schema` combination was unreliable — Claude sometimes returned plain text instead of JSON. Replaced with a prompt-embedded JSON requirement and a 3-fallback parser: bare JSON → fenced code block → first `{…}` in text.
- **`parse_file` signature** changed to return `(Vec<RawRow>, Vec<String>, String)` — the third value is full file text used for bank detection.

### Added
- Claude CLI response: 4 new unit tests for JSON extraction logic

---

## [2026-05-31] — Port conflict fix (8080 → 8090) + fish shell compatibility

### Fixed
- Backend moved from port **8080 → 8090** (port 8080 was occupied by another service)
- Updated `vite.config.ts` proxy and `scripts/dev.sh` default to 8090
- **Fish shell**: `source ../.env` doesn't work in fish. Backend now calls `dotenvy::from_path("../.env")` before `dotenvy::dotenv()`, so `cargo run` from `backend/` auto-loads the project `.env` without any shell-specific sourcing

---

## [2026-05-31] — Registration/login error messages

### Fixed
- Registration returned generic "Registration failed" for all errors. Now:
  - Duplicate email → `"An account with that email already exists"` (detected via Postgres error code 23505)
  - Password < 8 chars → `"Password must be at least 8 characters"` (400, not 409)
  - Backend not running → `"Cannot reach server — make sure the backend is running"`
  - Login: network errors distinguished from invalid-credentials 401

---

## [2026-05-31] — Initial release (v0.1)

### Added
- **Multi-user auth**: register/login with argon2 password hashing, 30-day JWT tokens
- **Statement ingestion**: multipart upload, calamine (Excel) + csv crate parser, HDFC/ICICI/SBI profiles + generic fallback
- **Dedup**: SHA-256 file hash (skip identical re-upload) + per-transaction fingerprint `UNIQUE(user_id, fingerprint)` with `ON CONFLICT DO NOTHING`; reports inserted vs skipped
- **Postgres RLS**: `FORCE ROW LEVEL SECURITY` on all user-data tables; two DB roles (`khata` owner, `khata_ro` read-only)
- **`GET /api/txns/`**: paginated transaction list
- **`GET /api/txns/dashboard`**: totals, monthly spend/earn buckets, top merchants
- **`POST /api/chat/ask`**: text-to-SQL via native `claude` CLI (OAuth, no API key), SQL validator (sqlparser), RLS-safe read-only execution with 200-row cap
- **React/Vite frontend**: login, register, upload, dashboard (bar chart), transactions, chat with "show SQL" toggle
- **`scripts/pg_init.sh`**: one-command Postgres setup (initdb, roles, database)
- **`scripts/dev.sh`**: start everything with one command
- **30 backend tests**: auth integration, dedup, fingerprint, normalise, RLS isolation, SQL validator, JSON extraction
