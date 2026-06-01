# Changelog

All notable changes to Khata are documented here.
Format: `## [date] — Summary` → bullet list of changes.

---

## [2026-06-01] — Add setup.sh / setup.bat (one-shot setup + start)

### Added
- `setup.sh` — single script for Termux/Linux/macOS that handles everything: checks prerequisites (cargo, psql, node, claude), copies `.env.example` if no `.env` exists, installs `sqlx-cli` if missing, initialises the Postgres cluster on first run, handles stale PID file, runs migrations, installs npm deps, then starts backend + frontend. Safe to re-run — every step is idempotent.
- `setup.bat` — Windows equivalent: same flow, opens backend and frontend in separate cmd windows, loads `.env` variables into the session.

---

## [2026-06-01] — Fix: Postgres connection (wrong port in .env) + POSTGRES.md docs

### Fixed
- `.env` had `DATABASE_URL` pointing to port 5432, but the Postgres instance runs on port **5433** (set in `postgresql.conf` during `pg_init.sh`). All backend connections were failing to connect at startup.
- `scripts/pg_start.sh` now auto-removes a stale `postmaster.pid` when the underlying process no longer exists (common on Termux — Android's memory manager kills background processes on screen lock).

### Added
- `POSTGRES.md` — comprehensive reference covering: instance details, first-time setup, daily start/stop, why Postgres dies on Termux and how to fix it, RLS mechanics, role permissions, migration commands, backup/restore, and a troubleshooting section.

---

## [2026-05-31] — Fix: year-over-year same-month comparison query

### Fixed
- Question "How my spending compared to previous financial years for the same month" was matching the generic `SpendPeriod` pattern (because it contains "spending") and returning the all-time total instead of a year-over-year breakdown.

### Added
- `SameMonthYears` query kind: groups spending by each year's instance of the current month, ordered newest first, with % change vs previous year. SQL uses `EXTRACT(month FROM value_date) = EXTRACT(month FROM CURRENT_DATE)` so it always compares the current calendar month across all years in the data.
- Pattern triggers: "year over year", "yoy", "year on year", "compare/compared/versus + year/financial year/fy", "same month + last year/previous year/financial year", "previous financial year + month/spend".
- The YoY check fires **before** the generic SpendPeriod catch-all to prevent false positives.

---

## [2026-05-31] — Ask Claude: predefined query engine (zero Claude calls for common questions)

### Changed
- **`chat/predefined.rs`** (new): pattern matcher for ~12 common question types — SpendPeriod, EarnPeriod, SavingsRate, NetBalance, TopExpenses, LargestExpense, TransactionCount, MonthlyBreakdown, CategorySpend. Matches by keyword + time qualifier (this month / last month / this year / this week / today). Category questions use word-level matching so "food spending" correctly resolves to "Food & Dining".
- **Ask Claude handler** now tries predefined first; only calls Claude CLI when no pattern matches. For predefined queries: 0 Claude calls, answer formatted in Rust (< 100 ms). For unrecognised questions: 1 Claude call (down from 2 — removed the `phrase_answer` round-trip; result formatted in Rust).
- **Claude SQL prompt** updated: added `category TEXT` column (was missing!), now injects the user's actual category list as context so Claude uses real names.
- **Suggestion chips** updated to 8 questions that all match predefined patterns (guaranteed instant responses).
- 36 backend tests pass including 5 new unit tests for the predefined matcher and INR formatter.

---

## [2026-05-31] — Date range filter on transactions page

### Added
- **Date preset buttons** on the Transactions page: All time · This week · This month · This quarter · This year · Custom
- **Custom date range**: selecting "Custom" reveals two `<input type="date">` fields (from / to) for arbitrary ranges
- **Backend wired up**: `from` and `to` query params on `GET /api/txns` were already in `ListParams` but ignored — now applied as `value_date >= $3 AND value_date <= $4` (nullable, so omitting either end leaves it open). Refactored the COUNT + data queries into a single parameterised form using `IS NULL` guards, eliminating the prior 4-branch duplication.

---

## [2026-05-31] — Fix pie chart click passing index instead of category name

### Fixed
- Pie `onClick` was reading `entry.name` (a Recharts internal field that defaults to the data index `0,1,2…`) instead of `entry.category`. Fixed by casting the entry to `CategoryBucket` and reading `.category` directly; also added `nameKey="category"` so Recharts tooltips use the category name.

---

## [2026-05-31] — Category drill-down + CategoryEditor new-category fix

### Added
- **Category drill-down from dashboard**: clicking any pie slice or legend row in the Spending by Category chart navigates to `/transactions?category=<name>` — shows only that category's transactions with all sort options available
- **Active filter chip** on transactions page: when a `?category=` param is present, a purple badge shows the active filter with an ✕ clear button; removed from URL on click
- **Backend category filter**: `GET /api/txns` now applies `WHERE category = $4` when `?category=` is passed (was parsed but ignored before)

### Fixed
- **CategoryEditor "Create new category" toggle bug**: clicking "← Choose existing" after typing a new name was resetting `category` to `''`, leaving the select with no valid value and the Save button permanently disabled. Now resets to `current` (the original category) instead.

---

## [2026-05-31] — CategoryEditor fix + sortable transactions

### Fixed
- **CategoryEditor popup** was silently clipped by `overflow: hidden` / `overflow-x: auto` on parent panels — nothing happened on click. Fixed by switching popup from `position: absolute` to `position: fixed` with `getBoundingClientRect()` positioning. Popup is also clamped to the right edge of the viewport and auto-closes on scroll or resize.

### Added
- **Sortable transactions** (backend + frontend):
  - Backend: `sort_by` (`date` | `amount` | `description` | `category`) and `sort_dir` (`asc` | `desc`) query params added to `GET /api/txns`; `ORDER BY` clause built from a whitelist-validated match (safe against injection)
  - Desktop: clickable column headers with `↑` / `↓` / `↕` indicators; active column highlighted in accent colour
  - Mobile: sort dropdown (`#mobile-sort`) shown only on `≤640px`, hidden on desktop

---

## [2026-05-31] — Transactions mobile card layout fix

### Fixed
- **Mobile card descriptions** no longer overflow on narrow screens: switched from single-line `truncate` to 2-line `-webkit-line-clamp` with `word-break: break-all`, handling slash-heavy bank strings (e.g. `UPI/123456789/PAYMENT-TO-AMAZON/AXIS`)
- **Amount** moved from a competing third flex column to the meta row (date · category · amount) with `margin-left: auto`, giving the description the full card width
- **Card container** now has `overflow: hidden` to prevent any child element escaping panel bounds

---

## [2026-05-31] — Frontend UI redesign (modern + mobile-friendly)

### Changed
- **`index.css`** fully rewritten as a design system: CSS variables for light/dark mode (`prefers-color-scheme`), design tokens (colors, radii, shadows), utility classes (`.btn`, `.form-input`, `.card`, `.panel`, `.stat-card`, `.badge`, `.data-table`, `.upload-zone`, `.chat-bubble`, grid helpers, etc.)
- **`App.css`** cleared (all styles consolidated into `index.css`)
- **`App.tsx`** updated: protected routes now wrap children in `Layout`; auth pages bypass the layout

### Added
- **`Layout.tsx`** component: fixed 224px sidebar with logo + nav links (desktop); sticky top bar + fixed bottom tab nav with icons (mobile ≤768px); logout button in both views; uses `lucide-react` icons (`LayoutDashboard`, `Receipt`, `MessageSquare`, `LogOut`)

### Improved
- **Login / Register pages**: centered card with logo, labeled inputs, loading state on submit button
- **Dashboard**: stat cards with colored icon badges and hover-lift animation; responsive `auto-fit` grid collapses to 2 columns on mobile; removed inline page nav (now in Layout)
- **Transactions page**: desktop shows styled `<table>`; mobile (≤640px) switches to card list with directional icons; `ChevronLeft/Right` pagination buttons
- **Chat page**: suggestion chips when empty; animated typing-dots loading state; SQL toggle with icons; auto-focus input after send
- **FileUpload**: drag-and-drop zone with visual feedback (`drag-over` state); icon-based success/warning/error messages
- **CategoryEditor**: `ChevronDown` trigger badge; styled popup using design-system classes; `accentColor` radio buttons; icon-labeled action buttons

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
