# HANDOFF.md — Khata Project Handoff

**Current Version:** v0.41.0  
**Last Updated:** 2026-08-26  
**Status:** Pre-release (0.x.y)

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
