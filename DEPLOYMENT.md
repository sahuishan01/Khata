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
  `/usr/local/lib/khata/current/khata` with `/root/Khata/.env`.
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

For frontend updates, run `npm run build` in `frontend/`, copy only `dist/` to a
new release directory, and switch `/srv/khata/current` after verification. For
backend updates, use the existing `Backend Tests` GitHub Actions workflow. It
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
