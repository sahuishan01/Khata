# SECURITY.md — Khata Security Policy & Remediation

**Status:** Authoritative. **Scope:** Web (`khata.algosculptor.com`), Android app, and the backend/API they share.
**Context:** Khata stores **financial PII** — bank/UPI transaction history, payee names, partial account numbers, IMPS references — for **multiple users under an `admin` role**, and exposes an **LLM "Ask Claude" feature that generates SQL**. Treat every item here as protecting real money data.

Severities: **P0 = fix now / block release · P1 = fix this sprint · P2 = fix soon.**

---

## Part A — Current issues found (from the shipped UI) and how to fix them

### A1. `Ask Claude` → natural-language-to-SQL  **[P0, Critical]**
The chat shows a **"show SQL"** disclosure, i.e. user text is compiled to SQL and (almost certainly) executed against the transactions DB. This is the single highest-risk surface in the product. An attacker (or a careless prompt) can attempt cross-user data exfiltration, schema discovery, or destructive statements.

**Mandatory controls (all of them):**
1. **The LLM output is untrusted input. Never execute it directly.** Prefer generating a *constrained query object* (allowed tables/columns/operators) that your backend compiles into a **parameterized** query — not raw SQL strings.
2. If you must run generated SQL, run it through a **read-only database role** that has `SELECT` only, on a **whitelist of views** (not base tables), with **no access to `users`, auth, or other tenants' rows**.
3. **Row-level tenancy:** every query **MUST** be forced through a `WHERE user_id = :currentUser` filter injected server-side — never trust the model or client to scope it.
4. **Statement allowlist:** reject anything that is not a single `SELECT`. Block `;`, multiple statements, DDL/DML (`INSERT/UPDATE/DELETE/DROP/ALTER/COPY/GRANT`), and DB-specific exfil (`pg_sleep`, `COPY … TO`, file functions).
5. **Hard limits:** statement timeout (e.g. 3s), `LIMIT` capped, max rows returned.
6. **Prompt-injection defense:** transaction descriptions are attacker-controllable (a payee can be named `IGNORE PREVIOUS INSTRUCTIONS …`). Never let row content alter system instructions; keep the SQL-builder deterministic and schema-bound.
7. **Egress/privacy:** if requests go to a third-party model, document it, get user consent, and **minimize** — send aggregates/schema, not raw PII rows, where possible.
8. **Log + rate-limit** every query (who, when, generated query, row count).

### A2. User provisioning with a plaintext password field  **[P0]**
"Manage Users → Add User (Email / Password)" lets an admin set a user's password directly. Verify and enforce:
- Passwords **hashed with Argon2id** (or bcrypt cost ≥ 12) server-side; **never** stored or logged in plaintext.
- Field uses masked/secure input on both platforms; never echoed, never in URLs, never in analytics.
- Admin-set passwords are **one-time / force-reset on first login**; prefer an **invite link** over admin-chosen passwords.
- Enforce a password policy (length ≥ 12, breached-password check via k-anonymity API).

### A3. Broad `admin` role / authorization  **[P1]**
A single `admin` can add/delete users and "Clear All Data." Implement **least-privilege RBAC**: separate "view own data" from "manage users" from "destructive ops." Every server endpoint **MUST** authorize on the server (never rely on the client hiding a button). Add **server-side object-level checks** (IDOR): a user requesting transaction `:id` must own it.

### A4. `Clear All Data` one-tap destructive  **[P1]**
Sits flush beside "Logout," same visual weight. Require typed confirmation (DESIGN.md §4.6), **re-authentication** before execution, server-side soft-delete with a recovery window, and an audit-log entry.

### A5. PII rendered in full  **[P1]**
Full names, UPI IDs, partial account numbers, and complete IMPS reference strings are shown across Dashboard/Transactions/Rules. Mask account numbers by default (`••••5914`), offer a privacy/blur mode, and **never** put full references in headline cards. Ensure PII is **not** sent to analytics/crash reporting.

### A6. Statement upload (CSV / XLS / XLSX)  **[P1]**
"Add Data → Upload" ingests bank statements. File parsing is a classic attack surface:
- **CSV formula injection:** never write user cell values back into a CSV/Excel export without prefixing dangerous leading chars (`= + - @ tab CR`). On import, treat cells as data, never evaluate.
- **XLSX = zip+XML:** use a parser with **XXE disabled** and **entity expansion limits**; guard against zip bombs (cap decompressed size and entry count).
- Enforce **max file size**, **MIME + magic-byte validation** (not just extension), parse in a **sandboxed worker** with a timeout, and scope every imported row to the uploading user.

### A7. Session / token handling  **[P1]**
Verify (web): auth tokens in **`HttpOnly`, `Secure`, `SameSite=Strict` cookies**, *not* `localStorage` (XSS-exfiltratable). (Android): tokens in the **Keystore/EncryptedSharedPreferences**, never plaintext prefs. Short-lived access tokens + rotating refresh tokens; **invalidate server-side on logout** and on "Clear All Data."

---

## Part B — Preventive standards (apply to all new code)

### B1. Transport & headers
- HTTPS only; **HSTS** with preload. No mixed content.
- Security headers: `Content-Security-Policy` (no `unsafe-inline`/`unsafe-eval`), `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `X-Frame-Options: DENY`.
- Android: `network_security_config` enforcing cleartext = false; consider certificate pinning for the API.

### B2. Authentication
- Argon2id hashing; per-user salt (built in). MFA/TOTP available for accounts. Lockout/backoff on repeated failures.
- "Reset Password" / "Change email": rate-limited, single-use time-boxed tokens, notify the existing email on change, re-auth for sensitive changes.

### B3. Authorization (the big one for multi-tenant finance data)
- **Default deny.** Authorize on the server for every request.
- **Row-level security**: scope all reads/writes to `user_id`. Add an automated test that proves user A cannot read user B's transactions, rules, budgets, accounts, or chat history.

### B4. Input validation & output encoding
- Validate/normalize all input server-side (allowlist, types, ranges). Amounts as integer paise.
- Use parameterized queries / an ORM everywhere — **no string-built SQL** anywhere, not just Ask Claude.
- Context-correct output encoding to prevent stored XSS (payee/notes fields are user-controlled and rendered in lists, charts, and the LLM context).

### B5. Data protection
- Encrypt PII **at rest** (DB-level or column-level for sensitive fields) and **in transit**.
- Data minimization & retention policy; provide **export** and **delete-my-data** (also a compliance need under India's DPDP Act).
- Backups encrypted and access-controlled; test restores.

### B6. Secrets & config
- No secrets in the repo, the client bundle, or the APK. Use a secrets manager / env injection. **Client apps cannot hold API keys for paid services** (e.g. the model API) — those calls go through your backend.
- Rotate credentials; scope DB users (the read-only role for Ask Claude is separate from the app's read-write role).

### B7. Logging, audit & monitoring
- Audit log for: logins, user CRUD, destructive ops, exports, and every Ask Claude query. **Never log** passwords, tokens, full PAN/account numbers, or raw PII.
- Alert on anomalies (bulk reads, repeated auth failures, Ask Claude rejections).

### B8. Dependencies & pipeline
- Automated dependency scanning (`npm audit` / `pip-audit` / Gradle) and SAST in CI; block on high severity.
- Pin versions; review the bank-statement parser and any SQL/LLM libraries closely.

---

## Part C — Release gate (paste into the PR template)
- [ ] No raw/string-built SQL introduced; all queries parameterized.
- [ ] Any LLM-generated query passes the §A1 allowlist + read-only role + forced `user_id` scope.
- [ ] New endpoints enforce server-side authN **and** object-level authZ (IDOR test added).
- [ ] No secrets/keys in client bundle or APK.
- [ ] User input validated server-side; output encoded; uploads size/type/sandbox-checked.
- [ ] Tokens stored securely (HttpOnly cookie / Keystore); none in `localStorage`.
- [ ] PII not sent to logs/analytics; account numbers masked in UI.
- [ ] Destructive actions require confirm + re-auth + audit entry.

---

### Disclaimer
This is engineering security guidance, not legal/compliance advice. Because Khata handles Indian financial data, review obligations under the **DPDP Act 2023** and any RBI/payment-data localization requirements with a qualified professional, and consider a third-party penetration test before scaling the user base.

