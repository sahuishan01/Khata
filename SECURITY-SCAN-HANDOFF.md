# Khata — Security Scan Handoff

**Status: ✅ COMPLETE.** The full `claude-security` scan finished on 2026-08-27 after several usage-limit-interrupted resumes.

- **Revision scanned:** `ff32334` (branch `master`, tree clean)
- **Scope / effort:** whole repository, `medium`
- **Pipeline:** 8-component inventory → threat model → 30 researchers → breadth sweep → 132-vote 3-lens verification panel
- **Result:** 52 raw candidates → 44 deduplicated → **20 verified** (4 HIGH, 12 MEDIUM, 4 LOW), 24 rejected by the panel
- **Verification status:** `verified`

## Where the results are

`CLAUDE-SECURITY-20260826-084441/` (gitignored by its own `.gitignore` — delete that file if you want it in history):

| File | Purpose |
|------|---------|
| `CLAUDE-SECURITY-RESULTS.md` | **The report — read this.** Full write-up per finding: impact, location, exploit, preconditions, fix, vote tally. |
| `CLAUDE-SECURITY-RESULTS.jsonl` | Machine-readable, one finding per line — for CI gates. |
| `CLAUDE-SECURITY-RESULTS.sarif` | SARIF for code-scanning dashboards / IDE viewers. |
| `CLAUDE-SECURITY-REVISION-ff3233450dcf.json` | Revision stamp: what was scanned, at what effort, how verified. |

## The 20 verified findings

| ID | Sev | Votes | Location | Issue |
|----|-----|-------|----------|-------|
| F1 | HIGH | 3/3 | `setup.sh:62` | Postgres `initdb -A trust`; scram rules are dead config → passwordless DB superuser from any local process |
| F2 | HIGH | 3/3 | `android/app/build.gradle.kts:29` | Release APK signed with committed `debug.keystore` + hard-coded password `khata2024` |
| F3 | HIGH | 3/3 | `android/app/build.gradle.kts:28` | (same keystore, distribution-integrity angle) |
| F4 | HIGH | 3/3 | `android/app/debug.keystore:1` | (same keystore, build non-repudiation angle) |
| F5 | MED | 3/3 | `android/.../sms/SmsReceiver.kt:62` | Any incoming SMS parsed → persisted + server-synced bank transaction (no origin check, no dedup) |
| F6 | MED | 3/3 | `backend/src/chat/claude_cli.rs:85` | `sanitize_question` panic — byte offsets from lowercased copy applied to original string |
| F7 | MED | 3/3 | `android/.../viewmodel/MainViewModel.kt:58` | Mandatory password reset bypassed by restarting the app; not enforced server-side |
| F8 | MED | 3/3 | `android/.../api/NetworkModule.kt:71` | OkHttp `Level.BODY` logging in release → bearer token + passwords in logcat |
| F9 | MED | 3/3 | `scripts/pg_init.sh:12` | Second init path also defaults to `trust` — same passwordless-superuser exposure as F1 |
| F10 | MED | 3/3 | `backend/src/auth/handlers.rs:57` | Login user-enumeration via Argon2 timing (skip on unknown email) |
| F11 | MED | 2/3 | `backend/src/auth/handlers.rs:206` | Session cookie omits `Secure` under default/documented 127.0.0.1 bind |
| F12 | MED | 3/3 | `backend/src/chat/claude_cli.rs:80` | O(n²) `sanitize_question` on 2 MB input → single-request CPU exhaustion / backend DoS |
| F13 | MED | 3/3 | `backend/src/chat/sql_validator.rs:75` | Text-to-SQL allowlist not applied to subqueries → `pg_catalog` / cross-tenant aggregate reads |
| F14 | MED | 2/3 | `backend/src/auth/handlers.rs:123` | Unauthenticated `/api/auth/setup` → first caller seizes the sole admin account |
| F15 | MED | 3/3 | `backend/src/auth/handlers.rs:166` | Per-email login lockout → attacker locks any user (incl. admin) out indefinitely |
| F16 | MED | 2/3 | `android/.../data/KhataDatabase.kt:23` | Room DB unencrypted + `allowBackup=true` → financial history in cloud/adb backup |
| F17 | LOW | 2/3 | `backend/src/chat/claude_cli.rs:85` | (same panic as F6, multi-byte-char input angle) |
| F18 | LOW | 2/3 | `backend/src/chat/predefined.rs:310` | `&desc[..55]` non-char-boundary slice panic on top-expenses query |
| F19 | LOW | 2/3 | `backend/src/auth/handlers.rs:209` | `must_reset_password` advisory only server-side (backend counterpart of F7) |
| F20 | LOW | 3/3 | `backend/src/auth/handlers.rs:165` | Login brute-force protection per-email only — no per-IP / global limit |

**Merge for fixing:** F2≡F3≡F4 (one keystore). F6≡F17 (one `sanitize_question` panic). F7≡F19 (enforce `must_reset_password` server-side). F1≡F9 share a root cause (`initdb` auth default). F15/F20 are the same rate-limiter — fix together.

## Recommended fix order

1. **F1 + F9** — `initdb -A scram-sha-256` in both `setup.sh` and `scripts/pg_init.sh`; regenerate `pg_hba.conf` so app rules precede any `trust` line; set real role passwords. *Highest impact: this is a full RLS/tenant-isolation bypass.*
2. **F2/F3/F4** — new signing key in CI secrets only; make `build.gradle.kts` read `keystore.properties` (CI already writes it); `git rm --cached` + purge history; rotate.
3. **F8** — gate OkHttp logging on `BuildConfig.DEBUG`; `redactHeader("Authorization")` / `redactHeader("Cookie")`.
4. **F6/F17 + F12** — rewrite `sanitize_question` as one linear pass on a single lowercased copy; cap `question` length in `ask_handler`; add `RequestBodyLimitLayer` to the chat router.
5. **F5** — SMS sender allowlist; treat SMS transactions as pending until user-confirmed; dedup on `(amount, date, payee, ref)`.
6. **F13** — recursive allowlist validation over all expression positions + revoke `pg_catalog` from `khata_ro` (defense in depth).
7. **F7/F19 + F14** — enforce `must_reset_password` in `CurrentUser`; gate `/api/auth/setup` on a bootstrap token or loopback.
8. **F10 + F15 + F20 + F11** — auth hardening batch in `auth/handlers.rs`: dummy Argon2 verify on unknown email, per-IP rate limiting, don't re-arm lockout while locked, derive cookie `Secure` from request scheme.
9. **F16** — SQLCipher with an Android-Keystore key, and/or `allowBackup=false` / backup-exclusion rules.
10. **F18** — one-line char-boundary truncation.

## Re-running / future scans

- The run directory (`.claude-security-run/`) was consumed by the renderer, so this specific run is no longer resumable. Start a fresh scan with `/claude-security` → **Scan codebase**.
- The `medium` whole-repo scan is large (~500 agents, ~2M tokens) and repeatedly hit the account usage limit. To avoid that, run it **scoped**: `/claude-security` then pick a scoped scan of `backend/src`, then `android/app`, then `frontend`+`infra` separately — or run right after a limit reset.
- After fixes land, use the **suggest-patches** job (`/claude-security` → Suggest patches) to turn findings into reviewable patch files, or re-scan the changed files to confirm closure.
- `~/.claude/settings.json` now has `enableWorkflows: true`, `env.CLAUDE_CODE_WORKFLOWS=1`, and `Workflow` in `permissions.allow` — needed for the scan pipeline on the Pro plan. Keep these.

## Suggested-patches job — ✅ COMPLETE (2026-09-02)

Ran `/claude-security` → **Suggest patches** over all 20 findings, patch base `76f86e19e1ba` (HEAD). Each patch was written only after: a `patch-generator` staged it in a throwaway clone → a `patch-verifier` reviewed it + ran the tests + returned three CONFIDENT claims → a fresh `scan-researcher` adversarially re-challenged the bare diff and came back clean.

**Products:** `CLAUDE-SECURITY-20260826-084441/patches/` — `PATCHES.md` (index), `F<n>.patch` (+ `F<n>.md` rationale) per patch, `patches.jsonl`. Apply with `git apply CLAUDE-SECURITY-20260826-084441/patches/F<n>.patch` from the repo root. Nothing was applied/committed/pushed.

### 9 patches written (11 findings closed)

| Patch | Closes | What it does | Tests |
|-------|--------|--------------|-------|
| `F1.patch` | F1 | `setup.sh`: `initdb` peer/scram, full least-privilege `pg_hba.conf` replacement, `password_encryption=scram-sha-256` | PG 13 harness + `cargo test` (no repo test covers `setup.sh`) |
| `F9.patch` | F9 | Same hardening for `scripts/pg_init.sh` | PG 13 harness (no repo test) |
| `F6.patch` | F6, **F12**, **F17** | `sanitize_question` rewritten as one linear boundary-safe pass; 4096-byte question cap; 64 KB `RequestBodyLimitLayer` on chat router | `cargo test` 54 pass + 5M fuzz |
| `F13.patch` | F13 | Hybrid `QueryChecker`/`LevelCollector` — allowlist enforced at every relation position, every depth; rejects `TABLE <name>`, table-functions, catalog-named CTEs | `cargo test` 76 pass + adversarial harness |
| `F7.patch` | F7, **F19** | `CurrentUser` returns 403 `password_reset_required` for flagged users except `/api/auth/me` + `/reset-password`; `/me` returns the flag; Android `checkAuth` gates on startup | `cargo test` 52 pass (3 new); Android hunks review-only |
| `F11.patch` | F11, **F14** | Cookie `Secure` from `COOKIE_SECURE` (default true); `/api/auth/setup` 403 on non-loopback bind unless `KHATA_ALLOW_REMOTE_SETUP=1` | `cargo test` 53 pass (4 new); handler wiring review-only |
| `F8.patch` | F8 | OkHttp logging gated on `BuildConfig.DEBUG` (`Level.NONE` in release) + `redactHeader` Authorization/Cookie | review-only (no Android SDK; AAPT2 arch mismatch) |
| `F16.patch` | F16 | `allowBackup=false` + `dataExtractionRules`/`fullBackupContent` excluding `khata.db` + sidecars + prefs + `crash_logs/` | XML validation, review-only (no SDK) |
| `F18.patch` | F18 | `&desc[..55]` → walk down to char boundary before slicing | `cargo test` 50 pass (1 new) |

### 6 findings with no patch (need a human / larger change)

- **F2/F3/F4** (committed keystore) — the `build.gradle.kts` rework is sound but closure needs a coordinated CI-workflow change + real key rotation + history purge; operational task, not one patch file. See `F2.md`.
- **F5** (unverified SMS origin) — security core (write SMS txns "unconfirmed", skip sync push) is sound, but a complete fix needs an in-app review/confirm screen that doesn't exist yet. Feature work.
- **F10 + F15 + F20** (auth hardening) — the dummy-Argon2-verify timing fix creates an unauthenticated Argon2 CPU/memory amplification vector that the attempted rate-limiting doesn't contain (adversarial pass CONFIRMED it). Needs `spawn_blocking` + a bounded semaphore over the whole login path. **F15's** time-boxed-lockout fix was independently verified clean and can be re-applied on its own.

## Changelog

- **2026-08-26 → 08-27:** Scan run (medium, whole repo). Interrupted 4× by usage limits, resumed from cache each time; final pass completed the full pipeline. 20 verified findings. Report delivered.
- **2026-08-28 → 09-02:** Suggest-patches job. Interrupted several times by usage limits, resumed each time. 9 patches written (11 findings closed), 6 findings left for a human, plus F12/F14/F17/F19 folded into another patch. Products in `CLAUDE-SECURITY-20260826-084441/patches/`.
