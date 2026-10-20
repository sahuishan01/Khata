# HANDOFF.md — Khata Project Handoff

**SMS HTTP 422 follow-up (2026-10-20):** The `v0.45.3` live receiver wrote
`valueDate` but left `txnDate` empty. `v0.45.6` repaired IDs but still serialized
that empty date. A read-only API contract probe on the VPS reproduced HTTP 422
at `[0].txn_date`; empty-queue push and v2 pull returned 200, with zero transaction
changes. Evidence: `/root/.local/state/khata-ingest-fix/sms-422-probe.json`.
The `v0.45.8` uploader uses the known date for its missing counterpart, omits
absent optional dates, and canonicalizes UUIDs. Validation messages report only
schema field/queue position. Existing queued rows are retained. The existing
release pipeline now runs `testReleaseUnitTest` before packaging; backend changes
only add contract tests. No backend deploy or migration is needed for this fix.
The `v0.45.7` CI run stopped at test discovery because a coroutine test returned
a list instead of Unit; it published no APK. The corrected test also checks that
the acknowledged entry leaves the outbox.
Verified release `v0.45.8` is commit `97ead0a`, Android run `39426528665`:
18 release unit tests passed; APK package `com.khata.app`, version code 191,
and signing certificate match the expected update path from `v0.45.6` (189).
Backend contract tests passed in `39425745878`; security passed in `39426517797`.
APK: `https://github.com/sahuishan01/Khata/releases/download/v0.45.8/khata_0.45.8-release.apk`.
Verification: `/root/.local/state/khata-ingest-fix/ci-sms-422-verification.json`.
Actual phone queue recovery remains pending installation and retry on the device.

**VPS deployment update (2026-10-20):** Khata is hosted at
`https://khaata2.algosculptor.com` through Cloudflare and Caddy. The web UI uses
compiled static assets; the API and database remain on loopback. Admin setup
is complete, and the temporary Caddy Basic Authentication gate has been removed;
Khata's own login protects application data. See `DEPLOYMENT.md` for services,
access details, and recovery paths.

**Ingestion update (2026-10-20):** Email runs now expose phase, message totals,
heartbeat and remaining-batch information; web and Android poll and display live
progress and errors. Capped IMAP scans process the oldest pending UIDs, preserving
older history for later batches. Updating a PDF password or folder resets the
watermark. SMS imports use stable UUIDs, scan the full inbox with progress, retain
failed uploads for retry, and preserve bank metadata. Device downloads use
`cursor_version=2` with a separate change sequence; conflict revisions are unchanged.
Migrations 0031–0033 are required. Android needs a rebuilt APK; see `DEPLOYMENT.md`.
Web progress was checked in a browser with synthetic responses. Android parser
and cache mapping are unit-tested, but phone/emulator UI QA remains outstanding
because no Android device or emulator is available on this VPS.
Verified build `v0.45.5-ingestion-20261020T090430Z` is deployed on this VPS.
Validation: 155 backend tests (including the separately run timing and encrypted
PDF tests), six Android tests, frontend build/lint, and browser progress/error
checks passed. Public and origin checks confirm the new assets and app-level
authentication; Caddy Basic Authentication remains disabled for Khata.

**SMS queue follow-up (2026-10-20):** Production contained zero transactions when
the user reported roughly 150 queued SMS. The previous scanner reported queued
items despite upload failures and retried only when new messages were found.
The Android update repairs persisted legacy IDs, retries existing queues directly
and every 30 seconds while the signed-in app is in the foreground, exposes pending
counts/server/errors, and refreshes transactions and dashboard after successful
sync. Unacknowledged uploads remain queued. An acknowledged legacy entry keeps
its identity when the inbox is rescanned. No device data was cleared.
Android's default server is now `https://khaata2.algosculptor.com`; explicit saved
server overrides are preserved. The user's installed build/server and an actual
phone recovery remain unverified; a release installation needs its original
signing key for an update that retains the queued database. Use the existing
tag-triggered GitHub Actions Android workflow and its configured signing secrets
to produce the installable update. The user explicitly prefers this CI/CD path.

The web transaction list and dashboard refresh every 15 seconds while visible
and on focus. Poll failures retain previous data and show retry status.
Browser checks covered imported rows, filters, failures/recovery, hidden tabs and
mobile layout. Web release `v0.45.5-sms-retry-20261020T110054Z` is deployed and
verified at the public and origin URLs. Backend release/migrations are unchanged.
Follow-up deployment evidence: `/root/.local/state/khata-ingest-fix/sms-deployment.json`.

**Verified CI release (2026-10-20):** `v0.45.6` tags commit `6049fe1`.
The existing Android workflow passed in run `39419480906` and published
`https://github.com/sahuishan01/Khata/releases/download/v0.45.6/khata_0.45.6-release.apk`.
APK verification confirmed package `com.khata.app`, version code 189 (previously
186), a matching release signing certificate, and the SMS retry UI/default VPS URL.
Backend tests passed in run `39419237583`; security passed in `39419419324`.
The existing SDK setup needed an explicit `platform-tools` input because the
action otherwise requested the removed `tools` package.

Phone queue recovery remains unverified until the user installs this signed APK
as an update and syncs the persisted queue. Never uninstall or clear storage to
work around installation issues. Verify an existing server override points to
the VPS. Eleven local Android tests passed before the CI release build.
Evidence: `/root/.local/state/khata-ingest-fix/ci-release-verification.json`.

Separate follow-ups: CodeQL's existing Java/Kotlin autobuild still fails and needs
explicit build configuration. The dependency security fixes in the repository
have passed CI but have not been redeployed to the VPS backend/static frontend;
the deployed ingestion functionality is still the releases documented above.

**Current Version:** v0.45.8

**Last Updated:** 2026-10-20

**Status:** SMS/email ingestion recovery; signed APKs use the existing GitHub Actions release workflow

---

## What Khata Is

Khata is a personal finance tracker with multi-user support. It ingests bank statements (CSV/XLS/XLSX), categorizes transactions, provides analytics/charts, budgeting, portfolio/net-worth tracking, and an LLM-powered "Ask Claude" chat that generates SQL queries from natural language.

**Data sensitivity:** Financial PII — bank/UPI transaction history, payee names, partial account numbers, IMPS references.

---

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  Frontend    │     │  Android    │     │  Self-hosted │
│  (React)     │     │  (Compose)  │     │  instances   │
└──────┬───────┘     └──────┬──────┘     └──────┬───────┘
       │                    │                    │
       └────────────┬───────┴────────────────────┘
                    │
            ┌───────▼────────┐
            │  Caddy Reverse  │  ← khata.algosculptor.com
            │  Proxy (:80/443)│
            └───────┬─────────┘
                    │
            ┌───────▼────────┐
            │  Vite Dev Srv   │  ← :5174 (proxies /api → :8090)
            │  (or static)    │
            └───────┬─────────┘
                    │
            ┌───────▼────────┐
            │  Axum Backend   │  ← :8090
            │  (Rust)         │
            └───────┬─────────┘
                    │
            ┌───────▼────────┐
            │  PostgreSQL     │  ← :5433
            │  (RLS enabled)  │
            └─────────────────┘
```

---

## Directory Structure

```
Khata/
├── backend/              # Rust/Axum API server
│   ├── src/
│   │   ├── auth/         # JWT auth, middleware, handlers
│   │   ├── txns/         # Transaction CRUD + analytics
│   │   ├── ingest/       # CSV/XLSX statement parser
│   │   ├── chat/         # LLM SQL generation + validator
│   │   ├── accounts/     # Bank accounts
│   │   ├── budgets/      # Budget tracking
│   │   ├── categories/   # Category management
│   │   ├── portfolio/    # Net worth / assets / liabilities
│   │   ├── rules/        # Auto-categorization rules
│   │   ├── config.rs     # Env-based config
│   │   ├── db.rs         # Pool + RLS helper
│   │   ├── error.rs      # Unified error types
│   │   └── main.rs       # Router, CORS, middleware
│   ├── migrations/       # SQLx migrations (0001–0025)
│   └── Cargo.toml
├── frontend/             # React + Vite SPA
│   ├── src/
│   │   ├── api/client.ts # Axios instance, auth interceptor
│   │   ├── store/auth.ts # Zustand auth state
│   │   ├── pages/        # Route pages
│   │   └── components/   # Shared components
│   └── package.json
├── android/              # Jetpack Compose Android app
│   └── app/src/main/java/com/khata/app/
│       ├── api/          # Retrofit, TokenManager, NetworkModule
│       ├── data/         # Repository, Room DB, SyncEngine
│       ├── ui/           # Compose screens
│       ├── viewmodel/    # Hilt ViewModels
│       └── util/         # CrashLogWriter, formatters
├── DESIGN.md             # Visual design contract (tokens, spacing, rules)
├── SECURITY.md           # Security policy & remediation tracking
├── AGENTS.md             # Dev guidelines for AI agents
└── HANDOFF.md            # This file
```

---

## Key Technical Decisions

### Backend (Rust/Axum)
- **Framework:** Axum with Tower middleware
- **Database:** PostgreSQL via sqlx (compile-time checked queries)
- **Auth:** JWT (HS256, 30-day expiry) in `HttpOnly; SameSite=Strict` cookies + Bearer header. Token versioning for revocation.
- **Password hashing:** Argon2id (default params)
- **RLS:** Row-Level Security on all user-scoped tables via `SET LOCAL app.current_user_id`. Helper: `db::set_current_user()`.
- **LLM:** Claude CLI invoked as subprocess. SQL validated by `sql_validator.rs` (allowlist: `transactions`, `statements`, `chat_messages`, `user_accounts`). Read-only DB role + statement timeout + row limit.
- **Ownership:** `verify_ownership()` uses `OwnedTable` enum (no string interpolation).

### Frontend (React/Vite)
- **State:** Zustand for auth, React Router for navigation
- **Auth:** Cookie-based (`withCredentials: true`). 401 interceptor clears state without page reload.
- **API base:** Relative `/api` paths (works when served by same backend). Configurable via localStorage for self-hosting.
- **Styling:** Custom CSS with design tokens (no Tailwind). Dark mode by default.

### Android (Jetpack Compose)
- **DI:** Hilt
- **Networking:** Retrofit + OkHttp. Dynamic base URL via `@Named("server")` interceptor reading from `TokenManager`.
- **Auth storage:** `EncryptedSharedPreferences` with auto-recovery on keystore corruption.
- **Server URL:** Stored in regular SharedPreferences. Configurable on Login/Setup/Profile screens.
- **Crash handling:** `CrashLogWriter` writes to SharedPreferences (accessible without root). Next launch shows crash report dialog with copy button.

---

## Environment Variables (Backend)

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `RO_DATABASE_URL` | Yes | Read-only DB role for LLM queries |
| `JWT_SECRET` | Yes | Min 32 chars, rejected if contains "change-me" |
| `BIND_ADDR` | No | Default `127.0.0.1:8090` |
| `CORS_ORIGINS` | No | Comma-separated origins. Default `http://localhost:5173` |
| `CLAUDE_BIN` | No | Path to Claude CLI. Default `claude` |

---

## CI/CD

**Workflow:** `.github/workflows/build-android.yml`
- **Trigger:** Push tags matching `v*`
- **Runner:** `ubuntu-latest`
- **Steps:** JDK 17 → Android SDK → Gradle `clean assembleDebug` + `assembleRelease` → GitHub Release with APKs
- **Signing:** `keystore.properties` + `upload.keystore` restored from GitHub secrets
- **Version:** Derived from git tag (`GITHUB_REF_NAME`)

**Workflow:** `.github/workflows/security-scan.yml`
- **Trigger:** Push to `master`
- **Steps:** `npm audit` + Trivy filesystem scan. Blocks on high-severity.

**Workflow:** `.github/workflows/backend-test.yml`
- **Trigger:** Push/PR to `master` touching `backend/**`
- **Steps:** Rust stable + clippy → `Postgres 16` service → fetch pinned `libpdfium`
  (`backend/scripts/fetch-pdfium.sh`, sets `PDFIUM_LIB_PATH`) → `cargo test` → `cargo clippy --all-targets`
- **Note:** clippy `-D warnings` deferred until ~40 pre-existing warnings are cleared.

---

## PDF Statement Parsing

The PDF ingest path uses **`pdfium-render`** to pull positioned words (x/y coords) from
each page, then reconstructs table columns from per-bank `BankProfile.pdf_columns`
bands (`backend/src/ingest/pdf/table.rs` → `rows.rs` → `reconcile.rs` confidence scoring),
orchestrated by `backend/src/ingest/pdf/mod.rs`.

- **`PDFIUM_LIB_PATH`** must point at the dir containing `libpdfium.so` (or the file).
  Local setup: `cd backend && ./scripts/fetch-pdfium.sh linux x64` (arch `arm64` on ARM),
  then `export PDFIUM_LIB_PATH="$PWD/.pdfium/lib"`.
- **Fallback:** when pdfium is unavailable or confidence is very low, the parser falls
  back to the legacy `pdf_extract` line-heuristic (`backend/src/ingest/pdf/legacy.rs`).
- **Low confidence** surfaces as `UploadResponse.warnings` (web + Android) and a
  non-fatal `parse-confidence` entry in email sync runs.
- **Credit-card statements** are detected in the shared `src/ingest/detect.rs`, which
  selects the `generic_cc` profile; `pdf/mod.rs` reads its `statement_kind`.
- Integration tests (`backend/tests/pdf_parser.rs`) self-skip unless `PDFIUM_LIB_PATH` is set.
- Spec: `docs/superpowers/specs/2026-09-09-pdf-table-parser-design.md`
- **Deploy:** the repo-root `Dockerfile` downloads pinned `libpdfium` (`chromium/8044`)
  in the builder and sets `ENV PDFIUM_LIB_PATH=/usr/local/lib/libpdfium.so`. No DB migration.

**Releasing:** Tag `vX.Y.Z`, push. Build creates release with `khata_X.Y.Z-debug.apk` and `khata_X.Y.Z-release.apk`.

---

## Self-Hosting

1. Deploy backend + PostgreSQL. Set env vars in `.env`.
2. Set `CORS_ORIGINS=https://your-domain.com` in `.env`.
3. Deploy frontend (Vite dev server or `vite build` + static serve).
4. Point reverse proxy to frontend port. Frontend proxies `/api` to backend.
5. Android: expand "Server Settings" on login screen → enter `https://your-domain.com`.

---

## Known Issues & TODOs

- **LLM SQL generation** relies on blocklist validator. Should migrate to table/column allowlist with constrained query objects (see SECURITY.md §A1).
- **RLS** is `ENABLE` (not `FORCE`) on tables added in migration 0025. Application-level `WHERE user_id = $1` is the primary guard.
- **JWT 30-day expiry** is long for financial data. Consider shorter-lived access tokens + refresh tokens.
- **No email verification** on account creation.
- **In-memory rate limiting** doesn't survive server restart (acceptable for single-instance).
- **Android `@Named` interceptors** — if a third interceptor is added, ensure Hilt qualifiers stay consistent.
- **Frontend CSS** has some inline styles in page components that should be extracted to CSS classes.

---

## Build & Run

### Backend
```bash
cd backend
cargo run                          # Dev (reads ../.env)
cargo test                         # Run tests (needs test DB)
cargo build --release              # Production build
```

### Frontend
```bash
cd frontend
npm install
npm run dev                        # Dev server on :5174
npm run build                      # Production build
```

### Android
```bash
cd android
./gradlew assembleDebug            # Debug APK
./gradlew assembleRelease          # Release APK
```

---

## Page Mapping (Web ↔ Android)

| Web Route | Android Screen | Purpose |
|-----------|---------------|---------|
| `/` | `DashboardScreen.kt` | Financial overview |
| `/transactions` | `TransactionsScreen.kt` | Transaction list with filters |
| `/upload` | `CombinedUploadScreen.kt` | Statement upload |
| `/chat` | `ChatScreen.kt` | Ask Claude (LLM) |
| `/analytics` | `AnalyticsScreen.kt` | Spending analytics |
| `/accounts` | `AccountsScreen.kt` | Bank accounts |
| `/rules` | `RulesScreen.kt` | Auto-categorization rules |
| `/budgets` | `BudgetsScreen.kt` | Budget tracking |
| `/portfolio` | `PortfolioScreen.kt` | Net worth |
| `/categories` | `CategoriesScreen.kt` | Category management |
| `/profile` | `ProfileScreen.kt` | Settings |
| `/admin/users` | `AdminUsersScreen.kt` | User management |
| `/more` | `MoreScreen.kt` | Navigation hub |
| `/login` | `LoginScreen` (AuthScreens.kt) | Login |
| `/setup` | `SetupScreen` (AuthScreens.kt) | Initial admin setup |
| N/A | `DebugScreen.kt` | Crash logs + connectivity test |

---

## Security References

- See `SECURITY.md` for the full security policy, remediation tracking, and release gate checklist.
- See `DESIGN.md` for design tokens, spacing rules, and the "purple for actions, green/red for money" color rule.
- See `AGENTS.md` for AI agent development guidelines.
