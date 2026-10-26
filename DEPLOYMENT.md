# VPS hosting

Khata is served at https://khaata2.algosculptor.com through Cloudflare's proxied
DNS record and Caddy. Cloudflare SSL/TLS is Full (strict). Caddy manages the
origin's Let's Encrypt certificate and renews through HTTP-01.

## Routing and services

- `/etc/caddy/Caddyfile` imports `/etc/caddy/sites/*.caddy`.
- `/etc/caddy/sites/khaata2.caddy` serves the compiled SPA from
  `/srv/khata/current` and proxies `/api/*` to `127.0.0.1:8090`.
- The web build is copied into a versioned `/srv/khata/releases/` directory;
  `/srv/khata/current` selects the active build. SPA routes fall back to
  `index.html`; API routes retain their original paths.
- `khata-backend.service` runs the release binary at
  `/usr/local/lib/khata/current/khata` with `~/projects/khata/.env`.
- PostgreSQL stays on `127.0.0.1:5433`. Caddy administration stays on
  `127.0.0.1:2019`.
- The Vite development service is disabled. No public application port is needed.
- The firewall permits only TCP 22 (SSH), 80 (redirects and ACME), and 443 (HTTPS).
  HTTP/3 is disabled, so UDP 443 is unnecessary.

## Access and secrets

Khata's own login protects application data. The temporary Caddy Basic
Authentication gate was removed on 2026-10-20 after the live setup-status API
confirmed that an admin account exists (`setup_required: false`). No separate
Caddy username or password is needed for this site.

The former `/etc/caddy/secrets/khaata2-auth.caddy` include is inactive. Its
restricted hash file and host-key-encrypted credential at
`~/.config/khata/web-access.cred` remain available for recovery; do not commit
them or copy their contents into instructions.

The backend keeps `BIND_ADDR=127.0.0.1:8090`, `COOKIE_SECURE=true`, and
`CORS_ORIGINS=https://khaata2.algosculptor.com` in its protected environment file.

## Changes and recovery

Validate configuration as the service user before reloading:

```sh
sudo -u caddy caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
sudo systemctl reload caddy
```

Build frontend and backend updates through the existing `Backend Tests` GitHub
Actions workflow. For frontend updates, download the successful run's
`khata-web-<commit>` artifact into a new release directory, verify it, and switch
`/srv/khata/current`. For backend updates, the same workflow
runs tests and Clippy, builds the release binary, and uploads `khata-backend-<commit>`
with PDFium, the source revision, and checksums. Download the artifact for the
verified commit with `gh run download`, verify `SHA256SUMS`, install the
binary into a new `/usr/local/lib/khata/releases/` directory, switch
`/usr/local/lib/khata/current`, and restart `khata-backend.service` after checking
the build.

The Caddy migration preserves config-sync at `https://82.41.67.76`, including
its existing pinned certificate and `/install.sh`, `/file.sh`, `/config-sync`,
and `/server.crt` downloads. Nginx is stopped and disabled. Pre-migration backups
are under `/root/.local/state/khata-hosting/`; `latest-backup` identifies the
snapshot. Backups include secrets and must remain restricted to root.

## Email and SMS ingestion update

Backend migrations 0031–0033 add email progress fields, the device transaction
identity index, and a separate transaction change cursor. The updated Android
client requests `cursor_version=2`, resets its old download cursor once, and
reconstructs its server cache without deleting unsent transactions. Older API
clients retain the previous cursor protocol.

Email scans remain bounded by `EMAIL_SYNC_MAX_MESSAGES` (default 200 per run).
When more messages remain, the progress panel explains that the next manual or
scheduled sync continues the remaining history. Each run reports its own total.
The default background polling interval is 15 minutes. No Gmail connection was
configured on this VPS when the update was investigated; connect it in Add Data
before expecting live mailbox imports.

Android now defaults to `https://khaata2.algosculptor.com`.
`KHATA_API_BASE_URL` can override this during Gradle execution. An explicitly
selected server on an existing device is preserved; verify the address shown in
the device sync panel. Keep the existing version tag in `GITHUB_REF_NAME`.
Use the original release signing key for an update to a release installation;
a debug APK is a test artifact and may not update an existing signed app.
Use `.github/workflows/build-android.yml` for the installable signed APK: push
the source changes, then use the existing `v*` tag trigger after verifying the
previous build with GitHub CLI as required by `AGENTS.md`. Signing secrets are already configured
in GitHub; inspect their names with `gh secret list`, without printing values.
Do not uninstall or clear app storage to work around an update failure: queued
SMS transactions are stored in that installation's database. The updated app
retries the existing queue without requiring another inbox scan or SMS permission.

The web transaction list and dashboard poll every 15 seconds while visible and
refresh on focus so phone uploads appear without a page reload. This follow-up
web deployment is recorded in
`/root/.local/state/khata-ingest-fix/sms-deployment.json`; the earlier manifest
still records the backend release and its encrypted database backup.

The ingestion rollout's encrypted database backup and prior release paths are in
`/root/.local/state/khata-ingest-fix/backup/`. The database archive is encrypted
with this host's `systemd-creds` key; keep that directory restricted to root.

## SMS dates and own-account card repayments

Android `v0.45.9` selects the transaction date from the SMS body, then the SMS
message timestamp, then the persisted queue timestamp. Bank dates use
`Asia/Kolkata`; due dates, statement dates and invalid dates are ignored.
Live capture and inbox scans share this policy. Existing dates remain stable
when an offline queue is uploaded later.

Credit-card bill payments and received repayment acknowledgements are transfers.
They remain visible in history but do not count towards spending, income,
savings or summary transaction counts. Card purchases, merchant refunds,
cashback and card fees retain their normal treatment. The backend applies the
same repayment policy to new statement/email/device imports. Migration 0034
backfills identifiable existing repayments and advances their device sync cursor.
Later manual transfer overrides are respected.

Install the signed APK as an update, then run **Scan SMS inbox** once to repair
earlier SMS imports using the original messages. Their existing identities and
other edits are preserved. The server accepts date corrections through sync.
The original SMS bodies were not uploaded, so older rows with only an extracted
payee require this device rescan. Keep app storage and its queue intact.

CI artifact verification, the backend deployment and encrypted backup are
recorded in `/root/.local/state/khata-ingest-fix/ci-card-repayments-verification.json`
and `card-repayments-deployment.json` in that same protected directory.

The backend was deployed from successful GitHub Actions run `39443068032`
(commit `bd31998`); 167 backend tests passed and one was ignored. The signed APK
was built by run `39441969710` with 26 tests passing and the previous signing
certificate. Live Caddy API checks passed after migration 0034. All 3,177 existing
SMS rows were preserved; their stored payee-only descriptions yielded no confident
repayment backfill matches, so use the inbox rescan for historical corrections.

## Rescanning after Clear All Data

The Android rescan refreshes its reproducible server cache before checking SMS
identities. This lets SMS be imported again after server history was cleared,
including when it was cleared from the web. Unsent local work is preserved, and
an incomplete server-history fetch leaves the existing cache untouched. Offline
scanning remains available with an explanation when the server check fails.

Clearing data from Android also clears that device's saved/queued transactions
and resets its sync cursor after the server confirms deletion. The confirmation
dialog explains this scope. A scan displays separate new, updated and already
saved counts, followed by its upload result. No database migration is needed.

Released as Android `v0.45.10`, signed by the existing key (versionCode 193).
GitHub Actions run `39450723305` passed 30 release unit tests, including the
2,961-message clear-and-reimport regression, queued-work preservation and failed
snapshot handling. Security Scan run `39450713088` passed. Install as an update
and scan the inbox once. Verification is recorded in
`/root/.local/state/khata-ingest-fix/ci-sms-clear-verification.json`.

## Future SMS dates and repeat alerts

Backend and web artifacts from successful run `39552892838` (commit `8c4493e`)
are deployed with migration 0036. The run passed 181 backend tests, one ignored,
Clippy and the web build. Production checks verified authentication, forced RLS,
summary totals, public asset hashes, sync exclusions and the absence of future
SMS dates. Backend and database ports remain bound to loopback behind Caddy.

The encrypted backup and backend deployment record are in
`/root/.local/state/khata-ingest-fix/sms-provenance-deployment.json` and its
referenced `backup/*.cred` file. Root-only verification evidence is under
`/root/.local/state/khata-duplicate-audit/`.

A valid date in an SMS must not be after the original message day. Untouched
legacy SMS imports with impossible future dates now retain their original dates
in `sms_source` and use their import day until the original SMS is rescanned.
Do not guess a day/month swap or restore previously cleared transaction history.

Rescans attach receipt time and hashes without uploading SMS text. Recognized
UTR/UPI references are saved on existing records without replacing their client
IDs. No-reference repeat alerts need matching anchored text and receipt within
two minutes; amount/payee/day alone remain review suggestions. Keep current data
when updating and rescanning: Clear all data also removes saved exclusions.

The signed Android update is
[v0.45.14](https://github.com/sahuishan01/Khata/releases/tag/v0.45.14), built by
run `39574262637`: 40 release tests passed and all required metadata fields
survived minification. The downloaded APK has versionCode 198, a valid ZIP and
signature, and the same signing certificate as v0.45.12. Install over the
existing app, then run **Scan SMS inbox** once. v0.45.13 was withdrawn after
artifact verification found its SMS metadata removed by R8; it was not sent
as a verified release notification. Invalid older metadata now falls back
without blocking queued transactions. Physical-device UI validation was unavailable.


## Automatic history loading and card confirmations

The current signed Android release is
[v0.45.15](https://github.com/sahuishan01/Khata/releases/tag/v0.45.15), built by
run `39581391908`: 46 release unit tests passed. The downloaded APK has
versionCode 199, a valid signature and the same signing certificate as
v0.45.14. Its required SMS metadata fields survived minification. Install over
the existing app, keep app data, and run **Scan SMS inbox** once to attach card
confirmation evidence to messages still present on the phone.

Backend and web artifacts from run `39582103790` (commit `76d658a`) are deployed
with migration 0037. All 187 backend tests passed, one was ignored, and Clippy
and the web build passed. The backend-only migration correction followed the
Android tag; Android and web sources are identical to the tested release.

Android and web load further transaction pages automatically near the bottom.
Page errors retain loaded records and allow retry. Filter changes reject stale
requests, refresh retains the loaded range, and tied API sort values use a
unique-ID tie breaker. Browser checks at phone and desktop widths cover these
flows, duplicate review, failed exclusion and Undo. Deployed web asset hashes
match the browser-tested CI artifact. Physical-phone installation and gestures
are not verified by these automated checks.

Unique, closely timed CRED debit/card receipt pairs are transfers. Later card
confirmations need explicit receipt/confirmation roles, the same card
fingerprint and unambiguous funding before automatic exclusion. Ambiguous
records remain review candidates. Source records and sync exclusions are
retained; spending and income exclude repayment legs and duplicate confirmations.
Migration 0037 drains deferred backfill events before restoring forced RLS.

The encrypted backup and backend record are in
`/root/.local/state/khata-ingest-fix/card-confirmations-deployment.json`.
Root-only evidence under `/root/.local/state/khata-duplicate-audit/` includes
`card-confirmations-ci-verification.json`, `apk-0.45.15-verification.json`,
`history-browser-verification.json`, `card-overlap-correction.json`,
`card-confirmations-live-verification.json` and `web-deployment.json`.
Live verification paged through the complete visible history without repeated
IDs, checked the reported repayment corrections and sync tombstones, compared
summary totals against the database, checked that no future SMS dates remain,
and verified public assets and authentication. Backend/database listeners stay
on loopback with Caddy as the public entry point.


## General repayment reconciliation follow-up

Backend artifact from successful run `39587141836` (commit `f4149f0`) is
deployed with migration 0038: 194 backend tests passed, one was ignored, and
Clippy and the web build passed. Both jobs ran on the intended `heaven` runner.
The deployed web assets remain those verified from successful run `39582103790`;
web source and the Android APK are unchanged. Both clients consume the shared
server reconciliation results.

The migration processes every tenant's existing repayment history. The deferred
triggers handle future imports, including a later generic confirmation in a
separate upload. CRED and explicit bill payments can supply funding evidence;
older SMS metadata can use a unique complete chain, while explicit card
fingerprints/roles also support missing funding history. Adjacent-day matches
are available in duplicate review. Ambiguous candidates are preserved for
review. Explicit restores remain authoritative during future rescans.

The encrypted backup is referenced by
`/root/.local/state/khata-confirmation-filter/backend-deployment.json`.
Additional root-only evidence in that directory includes `ci-verification.json`,
`history-at-deployment.json` and `live-verification.json`. Verification checks
every original financial field and source ID, newly excluded confirmations,
transfer legs, all history pages, summary totals, sync tombstones, public assets,
authentication and forced RLS. See `docs/transaction-identity.md` for rules.


## Upload retries reporting 200 conflicts

The app sends queued transactions in batches of 200. A repeated CREATE with
`base_rev=0` previously conflicted even when the server had already saved the
same transaction. Automatic repayment reconciliation also changed some revisions
after upload acknowledgements were assembled. The server now acknowledges exact
replays, re-enqueues canonical rows for download without changing their financial
fields, and flushes deferred reconciliation before returning final revisions.
Manual transfer overrides and genuine stale edits remain protected.

Backend run [39597159197](https://github.com/sahuishan01/Khata/actions/runs/39597159197) passed 202 tests (1 ignored),
Clippy and the web build on heaven. Its verified backend artifact is deployed;
there is no new migration or APK. Existing Android v0.45.15 can recover by tapping
**Retry queued uploads**. Keep the current app data and queue; another clear or
inbox scan is unnecessary for this recovery.

After the user's Clear all data and reimport, production replay verification
changed a 228-record sample from all conflicts to all acknowledgements.
Revisions matched committed rows, and canonical rows appeared after the prior
pull cursor. Every stored financial field, timestamp, exclusion and source ID
was preserved. Full checks covered all 2959 source rows, 2940 visible rows,
19 exclusions, history paging, sync tombstones, totals, authentication,
forced RLS, public web files and loopback-only service ports. Physical-phone
completion still needs the user to retry their existing queue.

The encrypted backup, artifact/deployment manifest and private verification
reports are under `~/.local/state/khata-sync-conflicts/` (the manifest references
the encrypted archive in `~/.local/state/khata-ingest-fix/backup/`). Previously
cleared history was not restored. The frontend assets remain unchanged.


## Widgets and installed-web shortcuts (2026-10-24)

Android `v0.45.16` is the verified signed update from run `40068680696` (51 release
unit tests; versionCode 202; same signing certificate as `v0.45.15`). It adds two
widgets, four launcher shortcuts and four Quick Settings tiles. Install as an
update and open More → Widgets & shortcuts; see `docs/android-shortcuts.md`.

The web build from successful run `40068081552` (revision `93bb1a9`) is active at
`/srv/khata/releases/v0.45.16-shortcuts-20261024T061240Z`. Its frontend source is
identical to the `v0.45.16` tag. It adds the authenticated `/add` route, return-to
navigation through sign-in/password reset, and four installed-web-app shortcuts.
Origin files and public JavaScript/CSS, manifest and revision match the CI
artifact. Cloudflare injects analytics into public HTML, whose asset references
were checked separately. The browser-tested production output matches the CI
artifact. Backend binaries and database migrations are unchanged.

Release/deployment evidence is under `~/.local/state/khata-shortcuts/`.

## SMS loan promotion correction (2026-10-24)

Android `v0.45.17` was built and signed by successful CI run `40085238240` on
`heaven` (61 release unit tests). The downloaded APK is versionCode 203 and its
signing certificate matches v0.45.16, supporting an in-place update. Its SHA-256
is `2519d73fbd4446cbd1f9bea5d7bc3996472071c715e146003057296351048ad9`, matching
the GitHub release asset digest. Archive integrity, all nine SMS metadata fields,
HTTPS-only configuration and retained shortcut components were verified. Local
release lint and Security Scan `40085238241` passed. GitHub default CodeQL's
existing unsigned Java autobuild issue remains separate from signed release CI.

Install [the APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.17/khata_0.45.17-release.apk)
as an update, then use **Add Data → Scan SMS inbox** with the original SMS present
and let sync finish. The parser ignores instructional loan offers, and the rescan
can exclude a proven unchanged false import without replacing its financial
fields. Edited or unverifiable entries remain for manual review. No live
financial data was modified during development. The existing backend email
extractor already rejects the example; no backend or web deployment is needed.
See `docs/sms-promotion-filtering.md`. Evidence:
`~/.local/state/khata-sms-promotions/`.

## Editable transaction titles (2026-10-24)

Android v0.45.18 is the verified signed update from run `40098934956`, attempt 2:
62 release unit tests passed, versionCode 204, and its signing certificate matches
v0.45.17. APK archive integrity, the GitHub asset digest, all nine SMS metadata
fields, the title update DTO/API, HTTPS-only policy and retained shortcut
components were verified. Local release lint passed.

Backend/web run `40098383249`, attempt 2, succeeded on `heaven`: 205 backend tests,
1 ignored, Clippy and web build. Both artifacts use tag commit
`0e4da62ab2dc08e2b34657348d7a5f01de335a2a`. The browser-tested web files match the
CI artifact. The backend executable and PDFium match verified artifact checksums;
health and the new endpoint's authentication check passed. Caddy origin/public
revision, public asset bytes and installed file checksums were verified.

Backend release: `/usr/local/lib/khata/releases/v0.45.18-titles-20261024T150021Z`.
Web release: `/srv/khata/releases/v0.45.18-titles-20261024T150021Z`.
APK SHA-256: `16edf00e976f092c50a57c337dd8e6f405f3ed3cec0ea9b434492e0e6e3c14a6`.

Open Transactions, expand a row, then choose **Edit title**. Android also exposes
it under transaction actions. Install the signed APK as an update. Saved titles
sync and survive later SMS rescans. No database migration or direct edit to live
user transactions was needed. API 35 emulator and desktop/mobile browser checks
covered editing, cancellation, validation, failed-save drafts, retry and reopen.
Security Scan `40098383337` passed. The existing default CodeQL Java autobuild
signing configuration issue remains separate from signed release CI.

The first attempt failed to upload its otherwise successful backend build because
the VPS clock lagged NTP/GitHub by about 42 minutes. Time synchronization was
refreshed, the Khata runner registration restored, and encrypted runtime credential
handling verified before successful CI retries. Resource caps were retained.
Evidence and screenshot comparison: `~/.local/state/khata-transaction-titles/`.

## Pending SMS request filtering (2026-10-24)

Android `v0.45.19` is the verified signed update from run `40108105231` on `heaven`:
69 release tests passed, versionCode 205, matching v0.45.18's signing certificate.
The downloaded APK retains request/future-posting guards, SMS metadata and the
existing title/shortcut components. ZIP integrity and the GitHub asset digest
were verified. Local release lint and Security Scan `40108105133` passed.

APK SHA-256: `ad45a018edaa991b8767eb0cb2eb82354981060d72134c8f35ecc74bb1a01346`.

Install as an update, then run **Add Data → Scan SMS inbox** with the original
message present. Rescans exclude only proven unchanged false imports and use
normal sync; edited or unverifiable entries stay available for review. Backend
email filtering already rejects the example (15 parity tests passed). No backend,
web or database migration is needed; the v0.45.18 deployment remains current.

The exact reported HP Telecom text was already ignored by v0.45.18. Its stored
entry/import path and phone version are unconfirmed, and no matching record was
found by read-only provenance/company/amount lookup on this server. No live user
transaction was modified. Do not claim the particular saved entry is removed.
Usage and scope: `docs/sms-request-filtering.md`. Evidence:
`~/.local/state/khata-sms-requests/`.

## IPO application receipt filtering (2026-10-24)

Android `v0.45.21` is the verified signed update from run `40114558787` on `heaven`:
77 release tests passed, versionCode 207, matching v0.45.19's signing certificate.
The downloaded APK retains application/fund-block guards, required SMS metadata,
title editing and all existing widgets/tiles/launcher shortcuts. ZIP integrity,
GitHub asset digest and HTTPS-only configuration were verified. Local release
lint, 16 email-parser parity tests and Security Scan `40114558309` passed.

APK SHA-256: `fae5aeddc76aba616785554104ef2a81fd27eb9ea467d01625ba2e76852996ea`.

[Download APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.21/khata_0.45.21-release.apk) · [Release page](https://github.com/sahuishan01/Khata/releases/tag/v0.45.21).

Install as an update and run **Add Data → Scan SMS inbox** with the original SMS
present. The existing guarded correction excludes only proven unchanged false
imports through normal sync, preserving edited/unverifiable entries. The exact
reported SMS was rejected by a direct JVM check, and neutral-identifier fixtures
cover equivalent wording and variants. No live user transactions were modified
in development. Backend/web source is unchanged; no deployment or migration is
needed. Usage: `docs/sms-request-filtering.md`. Evidence:
`~/.local/state/khata-sms-ipo/`.


## Exact SMS and complete transaction editing (2026-10-25)

Android [v0.45.23](https://github.com/sahuishan01/Khata/releases/tag/v0.45.23) is the verified signed update from
run `40155432187` (81 release tests, versionCode 209, signing continuity with
v0.45.21). [Download APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.23/khata_0.45.23-release.apk).
Install as an update, expand a transaction and choose **Edit transaction**.
**Add Data → Scan SMS inbox** backfills proven automatic descriptions while the
original SMS remains present; saved custom descriptions and full edits survive.

Backend/web CI `40161566580` at `860395d4d7d42acbeb388d39d0345d1fab5adc44` passed 214 backend tests
(1 ignored), Clippy and the web build. Both artifacts are deployed:

- Backend: `/usr/local/lib/khata/releases/v0.45.23-transactions-20261025T043328Z`.
- Web: `/srv/khata/releases/v0.45.23-transactions-20261025T043328Z`.
- Database migration: 0039, function/trigger updates with no financial backfill.

The Android tree matches the signed tag. Backend/web include the server-side
legacy-client, inline-edit and source-matching safeguards. Local release lint,
Security Scan `40161566635`, browser flows and API 35 emulator flows passed.
Verified CI checksums, public assets/revision, service health, authentication,
forced RLS and loopback bindings match the deployment. All original financial
and source fields remained unchanged; sync delivery cursors may advance normally.
The encrypted database backup and verification manifests are referenced under
`~/.local/state/khata-transaction-edit/`. See `docs/transaction-editing.md`.


## SMS title and notes swap (2026-10-25)

Android [v0.45.24](https://github.com/sahuishan01/Khata/releases/tag/v0.45.24) is the verified signed update from
run `40176026574` (82 release tests, versionCode 210, signature continuity
with v0.45.23). [Download APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.24/khata_0.45.24-release.apk). The title is
**Auto-captured via SMS (bank)** and Notes holds the exact original SMS.
Use **Add Data → Scan SMS inbox** to swap verified automatic entries with the
original message present. Custom and unverifiable title/notes pairs stay; full edits remain protected.

Backend/web CI `40176342020` at `7fdcb20f6e2ad44ab01e55fc623d8236ff0fc870` passed 216 tests
(1 ignored), Clippy and web build. Deployed artifacts:

- Backend: `/usr/local/lib/khata/releases/v0.45.24-sms-notes-20261025T083753Z`.
- Web: `/srv/khata/releases/v0.45.24-sms-notes-20261025T083753Z`.
- Migration 0040 updates SMS evidence functions without rewriting stored rows.

Public revision/assets match verified CI output and browser-tested files.
Stored financial/source records, encrypted backup roundtrip, authentication,
forced RLS and loopback bindings were verified. Release lint, Security Scan
`40176342018`, desktop/mobile browser checks and Android API 35 emulator checks
passed. The Android tree matches the signed tag. Evidence and backup manifest:
`~/.local/state/khata-sms-title-notes/`. See `docs/transaction-editing.md`.


## Generic SMS postings (2026-10-25)

Android [v0.45.26](https://github.com/sahuishan01/Khata/releases/tag/v0.45.26) is the verified signed update from
run `40194289828` (88 release tests, versionCode 212, signing continuity
with v0.45.24). [Download APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.26/khata_0.45.26-release.apk). A shared grammar recognizes
account Dr/Cr postings across issuers, with generic payee delimiters, UPI
reference whitespace and bank-name suffix normalization. Unknown issuers do not
prevent a posting from being recognized. See `docs/sms-posting-grammar.md`.

The exact reported debit/receipt and Android-to-server repayment/retry replay
passed in isolation. Release lint, Security Scan `40194289598`, desktop/mobile
browser checks and API 35 real SMS receive/upload/display checks passed.
Install as an update, then **Add Data → Scan SMS inbox** with originals present.
The receipt keeps its credit direction and transfer flag. Custom edits and
verified prior identities remain protected.

Backend/web remain deployed from run `40176342020` at `7fdcb20`, with migration
0040. Their source is unchanged; no server build, deployment or data backfill is
needed. Production health/revision were rechecked. Exact fixtures/replay records
remain private under `~/.local/state/khata-generic-sms/`.


## v0.45.27 background SMS decisions (2026-10-26)

Signed [v0.45.27](https://github.com/sahuishan01/Khata/releases/tag/v0.45.27) passed Android CI `40262809332`
(104 release unit tests), release lint, APK digest/ZIP/signature checks and
certificate continuity with v0.45.26; versionCode 213 supports updates.
[Download APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.27/khata_0.45.27-release.apk). SMS decision/source DTOs, worker class,
editing metadata, production API URL, HTTPS-only networking and shortcut/widget
components are retained. Signed startup and the production sign-in screen were
checked on an API 35 emulator. Physical-phone verification remains unavailable.

Backend/web artifacts from successful run `40209851091` (`69b15ca`) are deployed:

- Backend: `/usr/local/lib/khata/releases/v0.45.27-laya-20261025T162200Z`.
- Web: `/srv/khata/releases/v0.45.27-laya-20261025T162200Z`.

Their backend/frontend source trees match the signed tag. Migration remains 0040.
Encrypted backup round-trip, unchanged stored financial/source records, forced
RLS, authenticated SMS endpoints, public asset equality and loopback bindings
were verified. The build passed 222 backend tests (1 ignored), Clippy and web
build. The tag revision's backend/web CI `40212200830` also passed.

Before v0.45.30, the resident model ran as `khata-laya.service` under a dedicated unprivileged
user, CPU-only on 127.0.0.1:8091. It loads the pinned English Laya checkpoint
`7b928d828b7b0e022f929d9bd2e44165aa270148` offline. CPU threads/quota, memory
limits, an 80% CPU-idle gate, filesystem protections and loopback-only egress
bound its work. The authenticated Rust gateway owns the public API; no model
port is exposed. That service is now stopped and disabled; see `docs/sms-local-import.md`.

Android and web browser checks cover exact Notes, full edits and payment-method
correction. Native checks cover real SMS receipt, OTP exclusion before queueing
or network calls, offline force-stop/retry, explicit manual review, Room 2 to 3
migration, separate identical events and repeated inbox scans without new
financial rows or overwriting edits. Private exact-message/model fixtures stay
outside the repository. Runtime dependencies were patched after the initial
scan; Security Scan `40212200937` and policy CI `40212200885` passed. Default
CodeQL Java autobuild still has its existing unsigned-release configuration
failure; it is not included in the passing-check claim.

Evidence, signed APK manifest, encrypted backup, deployment manifest and browser/
emulator results: `~/.local/state/khata-laya/`.

## v0.45.30 local SMS parsing

Laya has been removed from SMS imports at the user's request.
`khata-laya.service` is stopped and disabled; the private model files are retained
for recovery only. Do not start it unless explicitly requested. The Android app
uses its local posting parser and the ordinary bulk-sync API. Existing queued
messages are rechecked locally after updating; saved transactions and edits remain.
The retired decision route returns a terminal error and never forwards message
bodies to an inference service. Backend run `40326120243` is deployed from source `4a509d9`; migration remains
0040. Its release artifact, encrypted backup round-trip, unchanged financial/source
records, forced RLS, authenticated retired-route responses, and loopback-only
backend/database bindings were verified. The web release and assets are unchanged.
Backend path: `/usr/local/lib/khata/releases/v0.45.30-local-sms-20261026T130932Z`.
See `docs/sms-local-import.md`; evidence is private under
`~/.local/state/khata-sms-local-restore/`.

Signed Android [v0.45.30 APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.30/khata_0.45.30-release.apk)
and [release page](https://github.com/sahuishan01/Khata/releases/tag/v0.45.30):
CI `40326121068` passed 108 release tests. APK ZIP/digest/signature, versionCode
216, signing continuity with v0.45.29, no model-decision endpoint, production
URL, HTTPS-only networking and API 35 startup were verified. Release lint,
three private exact-SMS regressions, true debug-app upgrade with 12,000 local
queue exclusions and two upload batches, offline/reconnect, OTP exclusion and
repeated scans passed. Backend CI passed 219 tests (1 ignored), Clippy and web
build; Security Scan `40326120916` passed. Physical-phone UI checks remain
unavailable. Current Android/backend source trees match the signed tag.

## SMS sync URL compatibility (v0.45.30/v0.45.31)

v0.45.30 incorrectly used `/api/sync/*` instead of `/api/txns/sync/*`. Caddy
now rewrites the exact legacy pull/push/resolve paths to the canonical handlers
before proxying. Queries, methods, bodies and application authentication are
preserved. Keep this compatibility mapping for installed v0.45.30 clients.
The app is being corrected to use the normal transaction sync route constants.

```caddyfile
@legacySmsSync path /api/sync/pull /api/sync/push /api/sync/resolve
uri @legacySmsSync path_regexp ^/api/sync/ /api/txns/sync/
```

The mapping lives inside `handle /api/*` in
`/etc/caddy/sites/khaata2.caddy`. Caddy validation/reload and live authenticated
GET/empty-POST probes passed for both route families; unauthenticated calls stay
401. No production test transactions were created. Evidence:
`~/.local/state/khata-sms-upload-route-fix/`.
