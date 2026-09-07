# Email Ingestion Worker — Design

Date: 2026-09-07
Status: Approved for planning
Component: `backend/src/ingest/email/`, Android Gmail Sync tab

## Problem

`user_email_configs` stores per-user Gmail credentials (app password + optional
statement-PDF password, both AES-256-GCM encrypted at rest). The `POST
/api/ingest/email/sync` handler is a stub: it only stamps `last_synced_at` and
returns a message. No IMAP client, no attachment fetching, no import. Users who
connect Gmail see nothing happen.

## Goal

A background worker that, per sync-enabled user, connects to their mailbox over
IMAP, finds emails carrying bank-statement attachments, and runs those
attachments through the existing parse → normalize → store pipeline. Every run
is recorded so the app can show real progress and history.

Non-goals: OAuth (app passwords only), non-IMAP providers, real-time push/IDLE,
categorisation changes, any change to how transactions are parsed or deduped.

## Existing pipeline (reused unchanged)

From `src/ingest/`:

- `detect::detect_file_kind(filename) -> FileKind`
- `detect::detect_bank(&profiles, hint) -> &BankProfile`
- `parse::parse_file(bytes, kind, profile) -> (Vec<RawRow>, Vec<String>, String)`
  — pure function; called twice (generic profile for the bank hint, then the
  detected profile)
- `normalize::normalize(raw_rows, user_id, stmt_id, profile, account_label) -> Vec<NormalizedTxn>`
- `store::store_transactions(pool, user_id, &txns) -> (inserted, skipped)`
  — dedupes on `ON CONFLICT (user_id, fingerprint) DO NOTHING`
- `statements` has `UNIQUE (user_id, file_sha256)` — natural dedupe for a
  re-seen attachment
- `crypto::decrypt_credential(enc, secret, user_id_str) -> String`

The `upload_handler` in `ingest/handlers.rs` is the reference implementation of
the full flow (validate → parse twice → insert statement → normalize → store).

## Architecture

Single in-process `tokio` task, spawned from `main()` after migrations, holding
`AppState` (db pool + `Arc<Config>`). The backend runs as one systemd binary
(`khata-backend.service`), so there is no separate process or cron.

```
main() ──spawn──> worker::poll_loop
                     │  every EMAIL_SYNC_POLL_SECS
                     ▼
                  pick sync_enabled users idle past the interval
                     │  (one at a time)
                     ▼
                  run::sync_user(state, user_id, Trigger::Poll)

POST /api/ingest/email/sync ──> run::sync_user(state, caller, Trigger::Manual)
                                  spawned detached; returns { run_id }
```

### Module layout: promote `src/ingest/email.rs` → `src/ingest/email/`

| File | Responsibility | Depends on |
|---|---|---|
| `mod.rs` | re-exports; route wiring helpers; `spawn_worker(state)` | all below |
| `config.rs` | `get` / `save` / `delete` config handlers (moved verbatim; `save` gains optional `sender_allowlist` / `subject_patterns`) | `crypto`, db |
| `imap.rs` | `trait Mailbox` (connect/search/fetch) + `RustlsImap` impl over `async-imap` + `tokio-rustls` | `async-imap` |
| `mime.rs` | `parse_attachments(raw_message: &[u8]) -> Vec<Attachment>` where `Attachment { filename: String, bytes: Vec<u8> }` | `mail-parser` |
| `pdf_decrypt.rs` | `decrypt_pdf(bytes: &[u8], password: &str) -> anyhow::Result<Vec<u8>>` — returns a decrypted PDF byte stream; `is_encrypted(bytes) -> bool` | `lopdf` |
| `run.rs` | `sync_user(state, user_id, trigger) -> ()` — one full run; the unit of testing | `imap` (trait), `mime`, `pdf_decrypt`, `detect`, `parse`, `normalize`, `store` |
| `handlers_runs.rs` | `GET /runs`, `GET /runs/latest`, `POST /sync` | db |
| `worker.rs` | `poll_loop(state)` | `run` |

`imap.rs` exposes a trait so `run.rs` is testable against a fake mailbox:

```rust
pub struct FetchedMessage { pub uid: u32, pub raw: Vec<u8> }

#[async_trait]
pub trait Mailbox: Send {
    /// UIDVALIDITY of the selected folder.
    async fn uid_validity(&mut self) -> anyhow::Result<u64>;
    /// UIDs matching the server-side criteria, ascending.
    async fn search(&mut self, since_uid: Option<u32>, criteria: &SearchCriteria)
        -> anyhow::Result<Vec<u32>>;
    /// Full RFC822 bytes for the given UIDs (batched internally).
    async fn fetch(&mut self, uids: &[u32]) -> anyhow::Result<Vec<FetchedMessage>>;
}
```

`SearchCriteria` carries the sender/subject terms (see Filtering).

## Schema changes

### `migrations/0028_email_sync_runs.sql`

```sql
CREATE TABLE email_sync_runs (
    id                 uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id            uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at         timestamptz NOT NULL DEFAULT now(),
    finished_at        timestamptz,
    status             text NOT NULL DEFAULT 'running',   -- running | ok | error
    trigger            text NOT NULL,                     -- poll | manual
    full_scan          boolean NOT NULL DEFAULT false,
    messages_scanned   integer NOT NULL DEFAULT 0,
    attachments_seen   integer NOT NULL DEFAULT 0,
    attachments_parsed integer NOT NULL DEFAULT 0,
    txns_imported      integer NOT NULL DEFAULT 0,
    txns_skipped       integer NOT NULL DEFAULT 0,
    errors             jsonb NOT NULL DEFAULT '[]'::jsonb, -- [{stage, item, detail}]
    error              text                                -- fatal summary, if status=error
);
CREATE INDEX idx_email_runs_user ON email_sync_runs (user_id, started_at DESC);

ALTER TABLE email_sync_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE email_sync_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY email_runs_user_iso ON email_sync_runs
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);

ALTER TABLE user_email_configs
    ADD COLUMN sender_allowlist   text[],
    ADD COLUMN subject_patterns   text[],
    ADD COLUMN imap_uid_validity  bigint,
    ADD COLUMN last_uid           bigint;
```

The worker runs outside a request, so it sets the RLS GUC itself via
`db::set_current_user(&mut *tx, user_id)` inside every transaction, exactly as
the request handlers do.

## Data flow: one run (`run::sync_user`)

1. **Guard** — in one txn: if an `email_sync_runs` row for this user has
   `status='running'` and `started_at > now() - interval '30 min'`, abort
   (`Manual` → return 409 to the caller; `Poll` → skip). Otherwise mark any
   stale `running` rows `error` ("superseded") and `INSERT` a fresh run row
   (`status=running`, `trigger`). Keep `run_id`.
2. **Load config** — `email_address`, `encrypted_app_password`,
   `encrypted_pdf_password`, `imap_server`, `sender_allowlist`,
   `subject_patterns`, `imap_uid_validity`, `last_uid`. If not
   `sync_enabled` → finish run `error` "sync disabled".
3. **Decrypt creds** — `crypto::decrypt_credential`. Hold in a struct that
   zeroizes on drop. `full_scan = last_uid IS NULL`.
4. **Connect** — `RustlsImap::connect(imap_server, email, app_password)`,
   `SELECT INBOX`.
5. **Watermark** — `uid_validity = mailbox.uid_validity()`. If
   `imap_uid_validity` is set and differs, or `full_scan` → `since_uid = None`;
   else `since_uid = Some(last_uid)`.
6. **Search** — `mailbox.search(since_uid, &criteria)`. The server-side term is
   `UID <since_uid+1>:*` AND (`OR FROM ...` from `sender_allowlist`) AND
   (`OR SUBJECT ...` from `subject_patterns`); each list contributes a term only
   if non-empty, so both-empty ⇒ "every UID since the watermark". Attachment
   presence is **never** a server term (IMAP has no portable one) — it is always
   the client-side filter in step 8. Cap the UID list at
   `EMAIL_SYNC_MAX_MESSAGES` (oldest first); a truncated result leaves the
   watermark at the last processed UID so the next run resumes.
7. **Fetch** in batches; `messages_scanned += n`.
8. **Per message** → `mime::parse_attachments`:
   - filter to extensions `{pdf, csv, xls, xlsx}`; `attachments_seen += 1` each
   - `> EMAIL_SYNC_MAX_ATTACH_BYTES` → record `errors[]` `{stage:"attachment", detail:"too large"}`, continue
   - `sha256(bytes)` → if `statements (user_id, file_sha256)` exists → skip silently
   - if PDF and `pdf_decrypt::is_encrypted` → `decrypt_pdf(bytes, pdf_password)`;
     on error record `errors[]` `{stage:"decrypt", item: filename}`, continue
   - `detect_file_kind` + two-pass `parse_file` + `detect_bank`; 0 rows or
     unknown bank → `errors[]` `{stage:"parse", item: filename, detail:"unrecognised"}`, continue
   - `INSERT statements (...) ON CONFLICT (user_id, file_sha256) DO NOTHING
     RETURNING id`; if no id (race) → skip
   - `normalize` + `store_transactions` → `txns_imported += inserted`,
     `txns_skipped += skipped`; `attachments_parsed += 1`
   - advance in-memory `max_uid`
9. **Finalize** — one txn: `UPDATE user_email_configs SET last_uid=$max_uid,
   imap_uid_validity=$uid_validity, last_synced_at=now(), last_error=NULL`;
   `UPDATE email_sync_runs SET status='ok', finished_at=now(), <counters>,
   errors=$errors`.
10. **Fatal error** (connect/login/select failed, or an unexpected error):
    `UPDATE email_sync_runs SET status='error', error=$msg, finished_at=now()`;
    `UPDATE user_email_configs SET last_error=$msg`. Watermark unchanged.

Counters are flushed to the run row periodically (every N messages) so a
long run shows live progress, not just a final total.

## Filtering

`SearchCriteria` from config:
- `sender_allowlist` (e.g. `["alerts@hdfcbank.net", "estatement@icicibank.com"]`)
  → `OR FROM "a" OR FROM "b" ...`
- `subject_patterns` (e.g. `["statement", "e-statement"]`) → `OR SUBJECT ...`
- Always AND with a "has attachment" heuristic. IMAP lacks a portable
  "has attachment" search, so: no server term for it — instead the client
  filters on the presence of a file attachment after `mime::parse_attachments`.
- Both lists empty → the run scans every message since the watermark and relies
  on attachment-presence + parser rejection. This is the permissive default; the
  app surfaces the two lists so the user can tighten them.

## Config (env, `config.rs`)

| Var | Default | Meaning |
|---|---|---|
| `EMAIL_SYNC_POLL_SECS` | `900` | poll loop interval; `0` disables the background loop (manual only) |
| `EMAIL_SYNC_MAX_MESSAGES` | `200` | max messages processed per run |
| `EMAIL_SYNC_MAX_ATTACH_BYTES` | `15728640` | per-attachment size cap (15 MB) |

Added to `Config` struct + `from_env` with `env_flag`-style parsing helpers for
integers.

## Endpoints (`ingest/mod.rs`)

| Method | Path | Change |
|---|---|---|
| GET | `/api/ingest/email/config` | unchanged |
| PUT | `/api/ingest/email/config` | body gains optional `sender_allowlist: string[]`, `subject_patterns: string[]` |
| DELETE | `/api/ingest/email/config` | unchanged |
| POST | `/api/ingest/email/sync` | **now** guards + spawns `run::sync_user(Manual)`, returns `{ run_id, full_scan }` or `409` |
| GET | `/api/ingest/email/runs?limit=10` | **new** — recent runs, newest first |
| GET | `/api/ingest/email/runs/latest` | **new** — single most recent run (or `null`) |

## Concurrency & safety

- One run per user: the `status='running'` guard row (step 1), stale after 30
  min.
- The poll loop processes users sequentially; a slow mailbox delays others but
  never runs two at once for one user.
- IMAP credentials: decrypted only inside `sync_user`, wrapped in a
  zeroize-on-drop struct, never logged. Connection uses TLS
  (`tokio-rustls` + webpki roots).
- Caps: `EMAIL_SYNC_MAX_MESSAGES` per run, `EMAIL_SYNC_MAX_ATTACH_BYTES` per
  attachment. Existing `parse.rs` guards (row cap, zip-bomb, formula injection)
  still apply.
- Per-item errors are captured in `errors[]` and never abort the run; only
  connection/protocol failures are fatal.
- `pdf_decrypt` shares code with `handlers.rs`: the manual upload handler is
  updated to call `is_encrypted` / `decrypt_pdf` when a `pdf_password` form
  field (or the user's stored key) is supplied — closes the existing gap where
  encrypted uploads fail.

## Android / Web progress UI (follow-up release, v0.43.3)

Backend-first. Once `/runs/latest` exists:

- **Gmail tab** shows the latest run: status chip (`running` spinner / `ok` /
  `error`), a one-line summary (`12 imported · 3 skipped · 1 error`), and while
  `running`, polls `/runs/latest` every 3 s and animates the counters.
- Expandable "Details" → the `errors[]` list.
- "Recent syncs" → `/runs?limit=10`, compact rows.
- "Sync now" button → `POST /sync` → begin polling.
- Web `UploadPage.tsx` gets the equivalent panel.

`KhataApi` / `KhataRepository` / `MainViewModel` gain `listEmailRuns` /
`latestEmailRun`; `CombinedUploadScreen` gets a `RunStatus` composable.

## Testing

| Unit | Test |
|---|---|
| `pdf_decrypt` | fixtures: an RC4-encrypted and an AES-encrypted statement PDF with known passwords → `decrypt_pdf` yields text-extractable bytes; wrong password → `Err`; `is_encrypted` true/false cases |
| `mime` | a multipart/mixed sample `.eml` with two attachments + inline image → `parse_attachments` returns exactly the two files with correct names/bytes |
| `run::sync_user` | `#[sqlx::test]` + a `FakeMailbox` seeded with fixture messages: asserts run counters, `statements` sha-dedupe (same attachment twice → one statement), `transactions` fingerprint-dedupe, watermark (`last_uid`) advance, UIDVALIDITY-change → full rescan, per-item error capture (corrupt PDF → `errors[]`, run still `ok`), `EMAIL_SYNC_MAX_MESSAGES` truncation leaves a resumable watermark |
| filter | `SearchCriteria` builder: allowlist/patterns → expected IMAP search tokens; empty → attachment-only |
| endpoints | `/sync` returns 409 while a run is live; `/runs/latest` RLS-scoped to the caller |

No live-network test in CI; `RustlsImap` is exercised only by a manual
`--ignored` integration test against a real Gmail test account.

## Files

**New**
- `backend/migrations/0028_email_sync_runs.sql`
- `backend/src/ingest/email/{mod,config,imap,mime,pdf_decrypt,run,worker,handlers_runs}.rs`
- test fixtures under `backend/tests/fixtures/email/`

**Modified**
- `backend/Cargo.toml` — `async-imap`, `tokio-rustls`, `mail-parser`, `lopdf`, `async-trait`
- `backend/src/main.rs` — `ingest::email::spawn_worker(state.clone())`
- `backend/src/ingest/mod.rs` — `email` is now a dir module; new routes
- `backend/src/ingest/handlers.rs` — call shared `pdf_decrypt`
- `backend/src/config.rs` — three env vars
- `backend/src/ingest/email.rs` — **deleted** (contents split into the dir module)
- `CHANGELOG.md`

**Follow-up (separate commit / v0.43.3)**
- Android: `KhataApi.kt`, `KhataRepository.kt`, `MainViewModel.kt`,
  `CombinedUploadScreen.kt`, `Models.kt`
- `frontend/src/pages/UploadPage.tsx`

## Rollout

1. Backend module + migration + tests → PR → merge → deploy (`deploy-latest.sh`).
   `EMAIL_SYNC_POLL_SECS` starts at `900`; can be set to `0` to ship dormant and
   enable after a manual `/sync` smoke test.
2. Manual `/sync` against the real connected account; inspect the run row.
3. Android + web progress UI → v0.43.3.
