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
