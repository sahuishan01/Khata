# Transaction editing and exact SMS descriptions

Expand a transaction and select **Edit transaction** on web or Android. Android
also offers this action in the transaction's three-dot menu. Edit description,
amount, expense/income type, transaction date, value date, category, bank, bank
reference, optional balance, transfer status and notes. Cancel discards the draft.
Saving requires a server connection. Failures retain the draft for retry; if the
record changed on another device, close and reopen the editor to review it.

New SMS imports use the exact message as their description, preserving line
breaks and surrounding whitespace. Existing automatic titles can be backfilled
with **Add Data → Scan SMS inbox**, while the original message remains present.
Backfill requires matching message hash and receipt time plus a recognized
generated title. Custom descriptions and unverified legacy descriptions remain
unchanged. Full transaction edits survive subsequent scans, including dates,
references, categorization and explicit transfer choices.

Changes through existing inline category, notes and transfer controls also
advance revisions. An already-open full editor cannot overwrite those changes.
Older scanners that omit new metadata cannot replace full edits through ordinary
uploads, even after pulling the latest revision. Unchanged upload retries still
acknowledge saved rows; changed queued rows require explicit conflict review.

The authenticated `PUT /api/txns/:id` accepts the complete editable field set
and `base_rev` from the opened transaction. Ownership is enforced with RLS and
explicit tenant scoping. Missing/excluded records return 404; changed revisions
return 409. Unknown fields are rejected. The update is atomic and returns the
canonical row after deferred reconciliation. IDs, original import identity,
raw source description and creation time remain stable. A new revision makes
the edit available to normal device sync and rejects stale queued writes.

Descriptions must be nonblank and contain at most 16000 characters. Amounts
must be positive and fit NUMERIC(14,2); optional balances may be negative. Both
use at most two decimal places. Category/bank are required (100 characters each),
references allow 500 characters and notes allow 10000 characters. Blank balance
clears it. Dates must be valid ISO dates. Description whitespace is preserved.

Migration 0039 updates automatic inference functions without backfilling user
data. SMS `user_edited` metadata and manual override flags protect saved edits;
`parsed_payee` preserves existing CRED funding evidence while the description
contains the full SMS. Generic confirmation matching also uses that provenance,
retaining the existing unique-chain proof and stricter treatment of known card
confirmations without a card key. Existing duplicate restore safeguards remain.
Derived payees require the description to match the original SMS hash, so
descriptions customized through older title editors keep their previous matching
behavior.
Explicit refund, salary, loan and similar economic-credit wording in original
SMS descriptions stays outside generic-confirmation exclusion.
Android's immediate cache update and conditional inbox write prevent a scan
from replacing an edit made while that scan was parsing. Dirty queued work
remains available for normal conflict review.

Regression checks cover field preservation, owner isolation, exclusions, stale
revisions, malformed input, exact long/multiline SMS text, guarded backfill,
rescan protection, card reconciliation and monetary validation. Browser and
emulator evidence is under `~/.local/state/khata-transaction-edit/`.


Verified release: [APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.23/khata_0.45.23-release.apk) · [Release page](https://github.com/sahuishan01/Khata/releases/tag/v0.45.23).
Android CI `40155432187` passed 81 release tests; backend/web CI `40161566580` passed
214 tests (1 ignored), Clippy and the web build. Release lint, browser flows,
API 35 emulator flows, APK signature/metadata and production deployment checks
passed. The existing default CodeQL unsigned Java autobuild issue remains separate.
