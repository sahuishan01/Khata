# Khata Development Guidelines

## Deployment

- Use Caddy for public web UI exposure; keep backend and database ports on loopback.
- See `DEPLOYMENT.md` for this VPS's Cloudflare hostname, service paths, and access setup.
- Build and distribute Android updates through the existing GitHub Actions workflow in `.github/workflows/build-android.yml`, using its tag-triggered signed release pipeline.
- Release notifications must include both the direct APK link and the GitHub release-page link. The shared `ntfy-agent-notify` helper adds the page link/button and opens the release page when tapped; keep direct links on `github.com`, not its expiring CDN redirect.

## Feature Parity

Changes made to the web frontend should also be applied to the Android app, and vice versa. Before considering a feature complete, verify it works on both platforms:
- `/profile` (web) ↔ `ProfileScreen.kt` (Android)
- `/upload` (web) ↔ `CombinedUploadScreen.kt` (Android)  
- `/transactions` (web) ↔ `TransactionsScreen.kt` (Android)
- `/accounts` (web) ↔ `AccountsScreen.kt` (Android)
- `/rules` (web) ↔ `RulesScreen.kt` (Android)
- `/budgets` (web) ↔ `BudgetsScreen.kt` (Android)
- `/portfolio` (web) ↔ `PortfolioScreen.kt` (Android)
- `/analytics` (web) ↔ `AnalyticsScreen.kt` (Android)
- `/chat` (web) ↔ `ChatScreen.kt` (Android)
- `/admin/users` (web) ↔ `AdminUsersScreen.kt` (Android)
- `/categories` (web) ↔ `CategoriesScreen.kt` (Android)
- `/more` (web) ↔ `MoreScreen.kt` (Android)

Keep the app version below 1.0 (pre-release, e.g. 0.x.y) until the user explicitly says it's production-ready.

Before changing the version number, verify the previous build through GitHub CLI. Use the last successful build tag as the reference point. Ask the user only if GitHub cannot establish the build status.

After verifying the previous build succeeded, auto-update the version number stored here and create the next release tag. The current version is defined by the latest git tag matching `v*`.

Current release version and last verified signed APK: `v0.45.10`
(GitHub Actions run `39450723305`, 30 release unit tests passed). Its signing
certificate matches `v0.45.9`. The release workflow runs regression tests before
packaging the signed APK.



## Design
- Read DESIGN.md for design related instructions

## Security
- Read SECURITY.md for security related instructions

## PDF Statement Parsing
- The PDF ingest path uses `pdfium-render` for positioned (coordinate-aware) text
  extraction, then reconstructs table columns from per-bank `BankProfile.pdf_columns`
  bands. Code: `backend/src/ingest/pdf/` (`table.rs`, `rows.rs`, `reconcile.rs`, `mod.rs`).
- Requires `PDFIUM_LIB_PATH` (dir containing `libpdfium.so`, or the file itself).
  Local setup: `cd backend && ./scripts/fetch-pdfium.sh linux x64` then
  `export PDFIUM_LIB_PATH="$PWD/.pdfium/lib"`.
- Fallback: when pdfium is unavailable or confidence is very low, it falls back to the
  legacy `pdf_extract` line-heuristic (`backend/src/ingest/pdf/legacy.rs`).
- Low confidence → `UploadResponse.warnings` (web + Android) and a non-fatal
  `parse-confidence` entry in email sync runs.
- Credit-card statements are detected and use the `generic_cc` profile.
- Integration tests (`backend/tests/pdf_parser.rs`) self-skip unless `PDFIUM_LIB_PATH` is set.
- Deploy: the root `Dockerfile` pulls pinned `libpdfium` (`chromium/8044`); no DB migration.
- Spec: `docs/superpowers/specs/2026-09-09-pdf-table-parser-design.md`
- See `backend/README.md` for how to add a bank's `pdf_columns` and fixtures.
