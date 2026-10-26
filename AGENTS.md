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

Current release version: `v0.45.30` (verification in progress).

`v0.45.30` removes Laya from SMS ingestion at the user's request. Local generic
posting grammar recognizes transactions before queueing; only parsed records
use normal batched sync. Existing pending/review jobs are converted locally,
without deleting saved transactions or edits. Progress, OTP protection, exact
Notes, payment methods, timestamps and owner-scoped identity checks remain.
No database migration or financial backfill. The Laya service is stopped and
disabled; do not re-enable it without an explicit user request. See
`docs/sms-local-import.md`. Release lint, 108 unit tests and all three exact SMS
regressions passed. A real debug-app update from v0.45.29 kept Room 3, locally
filtered 12,000 old jobs and handled 304 recognized SMS with 303 uploads in two
batches (199/104), zero model calls, preserved edits and owner isolation.
Backend unit checks passed; signed CI and backend rollout are in progress.
Evidence: `~/.local/state/khata-sms-local-restore/`.


`v0.45.29` adds persistent overall queued-SMS progress to Upload and Transactions:
finished/total, percentage and a progress bar. Pending and review messages keep
work unfinished; accepted, ignored and locally filtered messages finish work.
The total grows during inbox scanning, which cannot report completion early.
Retries and restarts retain progress for the current account/server. No migration
or backend/model/web deployment. Release lint and 114 unit tests passed.
Android API 35 checks verified 50% progress survives restart, 83% remains
unfinished with a review item, and resolving it reaches 100%. New incoming work
grows the total immediately (6/7, 85%) and returns to 100% after processing.
Account isolation, saved edits and both screens passed. Signed CI `40312581787`
passed with 114 release tests; Security Scan `40312582034` passed. APK ZIP/digest,
signing continuity with v0.45.28, versionCode 215, retained metadata/progress UI,
production API URL and HTTPS-only networking were verified. Android source
matches the signed tag; physical-phone verification is unavailable.
Evidence: `~/.local/state/khata-sms-overall-progress/`.

`v0.45.28` narrows phone SMS uploads to messages with a currency amount,
rechecks the old pending/review backlog locally, and separates SMS classification
from saved transaction sync. Exact queue counts remain visible while processing;
accepted transactions refresh before the queue drains. Manual retry resets old
backoff without losing completed decisions. Saved transactions, edits, import
identities and other owners remain protected. No backend/model deployment or
database migration. See `docs/sms-laya.md`. Local release lint and 110 unit tests
passed. Android API 35 checks passed for filtering 12,000 old queued texts,
partial-result refresh, transient retry, review completion, live OTP exclusion,
owner isolation and preservation of edited records. Signed CI `40289726794`
passed with 110 release tests; Security Scan `40289726974` passed. All three
exact reported SMS remain eligible in private regressions. APK ZIP/digest,
signing continuity with v0.45.27, versionCode 214, retained metadata, production
API URL, HTTPS-only networking and shortcut components were verified. The
Android tree matches the signed tag. Physical-phone verification is unavailable.
Evidence: `~/.local/state/khata-sms-candidate-fix/`.

Last verified signed APK: `v0.45.29` (run `40312581787`).

Earlier signed APK: `v0.45.27`
(GitHub Actions run `40262809332`, 104 release unit tests passed). Its signing
certificate matches `v0.45.26`; versionCode 213 supports updating the existing
app. ZIP/digest/signature, retained decision/editing metadata and worker class,
production API URL, HTTPS-only networking and shortcut components were verified.

`v0.45.27` adds persistent background Laya SMS decisions, OTP exclusion before
queueing/networking, owner-scoped imports and uncertain-result review. The local
model first decides transaction status, then debit direction for positive results.
Amount/completion/direction checks still protect against model mistakes. Payment
methods are visible and editable on Android and web. Inbox/live capture share the
SMS-centre timestamp; verified older arrival-based identities and custom edits
survive rescans. Android Room migrates 2 to 3. No backend migration or data backfill.
See `docs/sms-laya.md`. Release lint, 222 backend tests (1 ignored), Clippy, web
build, eight policy tests, desktop/mobile browser and Android API 35 receive,
offline restart/retry, migration, review, editing and repeated-scan checks passed.
All three exact reported SMS passed private live-model and extractor regressions.
Security Scan `40212200937` and SMS Decision Policy `40212200885` passed. The
Android tree matches the signed tag. Physical-phone UI verification is unavailable.
Backend/web run `40209851091` at `69b15ca` is deployed with migration 0040;
its backend/frontend source trees match the signed tag. Stored source/financial
records, authentication, forced RLS and loopback bindings were verified unchanged.
Before v0.45.30, `khata-laya.service` ran the pinned model offline as an unprivileged user on
127.0.0.1:8091, with CPU/memory limits and an 80% CPU-idle gate.
Evidence: `~/.local/state/khata-laya/`.

`v0.45.26` adds shared account posting grammar for Dr/Cr alerts across banks,
with generic payee field boundaries, spaced UPI references and issuer suffixes.
Issuer names do not control transaction recognition. Requests, offers, balances
and future debits retain their guards; verified legacy identities and custom
edits remain protected. See `docs/sms-posting-grammar.md`. Signed CI `40194289828`,
release lint and Security Scan `40194289598` passed. Exact reported SMS regression,
isolated Android-to-server repayment/retry replay, desktop/mobile browser checks
and Android API 35 receive/upload/display checks passed. Private exact-message
fixtures remain outside the repository. Backend/web are unchanged from deployed
run `40176342020`, with migration 0040; no server deployment is required.
Evidence: `~/.local/state/khata-generic-sms/`.

`v0.45.25` was cancelled before APK publication after device checks found
an unsupported regex flag accepted by JVM tests. The flag is removed in
v0.45.26; verify SMS receive/scan on Android, not only host JVM tests.

`v0.45.24` swaps SMS-generated title and notes: the title is
**Auto-captured via SMS (bank)** and Notes retains the exact original message.
Inbox rescans swap only verified automatic fields, preserving custom notes/titles
and full edits. Migration 0040 preserves repayment matching from verified Notes
and legacy descriptions, future-date safeguards and retry acknowledgements.
Notes accept 16000 characters on both clients and the API. No data backfill.
See `docs/transaction-editing.md`. Release lint, 216 backend tests (1 ignored),
Clippy, web build, browser/emulator checks and Security Scan `40176342018` passed.
Backend/web run `40176342020` (revision `7fdcb20`) is deployed with migration 0040.
The Android tree matches the signed tag. Stored financial/source records,
authentication, forced RLS and loopback bindings were verified. Evidence:
`~/.local/state/khata-sms-title-notes/`.

`v0.45.23` uses exact SMS text as the description and adds **Edit transaction**
to Android and web. All user-facing fields can be edited atomically. Saved edits
survive rescans, including older scanners; unchanged upload retries still work.
Legacy inline changes advance revisions, preventing stale whole-form saves.
Original message hashes preserve card matching without replacing prior custom
title behavior. Existing automatic titles can be backfilled through **Scan SMS
inbox** with the original message present; customized/unverified descriptions stay.
See `docs/transaction-editing.md`. Signed CI `40155432187`, release lint, 214
backend tests (1 ignored), Clippy, web build and Security Scan `40161566635`
passed. Browser and Android API 35 emulator flows passed. Backend/web run `40161566580`
(revision `860395d`) is deployed with migration 0039. The Android source tree is
identical to the signed tag. Financial/source records, forced RLS and loopback
bindings were verified. Evidence: `~/.local/state/khata-transaction-edit/`.

`v0.45.22` was cancelled before publication while transfer-override coverage was
completed. It has no release APK. The Android workflow compares against the last
successful signed release with an APK, so cancelled tags cannot mask app changes.

`v0.45.16` adds two home-screen widgets, four Quick Settings tiles and four
launcher shortcuts. Open **More → Widgets & shortcuts** for placement controls.
Shortcut destinations survive sign-in and required password reset. The matching
web shortcuts and `/add` route are deployed from successful run `40068081552`.
Android release lint, native widget/navigation checks, seven browser flows and
Security Scan `40068392810` passed. Evidence: `~/.local/state/khata-shortcuts/`.
Repository-wide web lint and GitHub default CodeQL autobuild still have existing
configuration/code issues; do not describe all repository checks as passing.

`v0.45.17` rejects instructional SMS loan offers while preserving completed loan
credits and genuine alerts with promotional footers. Explicit **Scan SMS inbox**
can exclude unchanged false imports only when the original SMS hash, timestamp,
identity and financial fields match. Preserve edited or unverifiable records.
See `docs/sms-promotion-filtering.md` for safeguards and regression checks.
Release lint and Security Scan `40085238241` passed. Signed APK integrity,
signature continuity, metadata and retained shortcut components were verified.
Evidence: `~/.local/state/khata-sms-promotions/`.

`v0.45.18` adds **Edit title** to expanded transactions on Android and web, and
the Android transaction actions menu. Titles use the existing description field;
keep import identities/source text stable, and preserve custom titles on SMS
repairs. See `docs/transaction-titles.md` for the API and validation checks.
Backend/web CI `40098383249` attempt 2 passed (205 backend tests, 1 ignored,
Clippy and web build); both artifacts are deployed. Browser and API 35 emulator
editing flows passed. APK signature continuity, metadata and title DTO/API were
verified. Evidence: `~/.local/state/khata-transaction-titles/`.

`v0.45.19` explicitly rejects pending UPI/mandate requests, request receipts and
future debits by clause. Keep completed mandate payments/refunds and real posting
clauses in the same message. Rescans exclude only proven unchanged false imports;
preserve edited or unverifiable records. The reported HP Telecom wording was
already rejected in v0.45.18; its stored-entry origin remains unconfirmed.
See `docs/sms-request-filtering.md`. Signed CI `40108105231`, local release
lint and Security Scan `40108105133` passed. APK integrity, signing continuity,
request/future-posting guards and required metadata were verified. Evidence:
`~/.local/state/khata-sms-requests/`.

`v0.45.21` rejects IPO/application acknowledgements and fund-block notices that
say an amount was received, even when the collect-request instruction follows in
another sentence. Require money evidence in the receipt clause; preserve real
IPO debits/refunds and separate completed postings. Rescan exclusions retain the
existing identity/provenance/edit safeguards. See `docs/sms-request-filtering.md`.
Signed CI `40114558787`, local release lint, 16 email-parser parity tests and
Security Scan `40114558309` passed. APK integrity, signing continuity, application/
fund-block guards and required metadata were verified. Evidence:
`~/.local/state/khata-sms-ipo/`.

`v0.45.13` was withdrawn after artifact checks found SMS metadata stripped by
R8. Use the latest verified signed release; the Gson DTO is retained explicitly
in the ProGuard rules.

Backend artifact from run `40209851091` (commit `69b15ca`) is deployed
with migration 0040 (222 backend tests passed, 1 ignored; Clippy and the
workflow's web build passed). Identical upload retries now acknowledge saved rows
and refresh the device's canonical cache. Automatic transfer/duplicate flags are
preserved; genuine stale edits still require review. Push and resolution return
revisions after deferred reconciliation. This recovery works in Android v0.45.15.

The current web assets are from run `40209851091`. Earlier desktop and phone
browser checks cover scrolling, retry, stale responses, refresh, duplicate review and Undo.
Whole-history reconciliation handles CRED and direct bill payments, older
confirmation metadata, separate upload batches and date differences. Explicit
restores survive later rescans. Live replay checks acknowledged 228 saved records
without conflicts or financial-field changes; all 2959 source records
remain available through paged sync, including exclusions. Authentication, totals,
forced RLS and loopback bindings were verified. Physical-phone UI verification
remains unavailable. See `DEPLOYMENT.md` and `docs/transaction-identity.md`.


## Design
- Read DESIGN.md for design related instructions

## Security
- Read SECURITY.md for security related instructions

## PDF Statement Parsing
- Before statement comparison, read `docs/statement-reconciliation.md` and the
  protected baseline reference at `~/.local/state/khata-baselines/comparison-session.json`.
  Preserve that frozen snapshot through uploads, rescans and parser repairs.
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
