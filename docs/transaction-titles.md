# Transaction titles

Open **Transactions**, expand a transaction, then choose **Edit title**. Android
also offers the action in the transaction's three-dot menu. The editor prefills
the current title. Save trims surrounding whitespace; Cancel discards the draft.
Blank titles are rejected and titles are limited to 500 characters. A failed save
keeps the draft open with an error for retry. Saving requires a server connection.

The title is the existing `description` field. The authenticated endpoint is
`PATCH /api/txns/:id/description` with `{"description":"New title"}`. It scopes
the update to the current user's non-excluded transaction, rejects unrelated
payload fields and advances both version and sync revision. Import identities,
raw source descriptions, amounts, directions, dates, references and SMS metadata
are preserved. Normal device sync distributes the title; stale queued writes
conflict rather than overwriting the edit. No database migration is required.

SMS rescans preserve the saved description even when adding transfer evidence or
repairing dates. New imports still receive the normal generated description.
The promotion cleanup also preserves renamed entries for manual review.

Local validation: 62 Android release tests and release lint pass. Three backend
tests cover owner isolation, missing/excluded records, empty/long titles, payload
restrictions, field preservation, sync delivery and stale-write conflicts.
Desktop and mobile-browser checks cover the prefilled title, Cancel, whitespace
validation, failed saves, retained drafts, retry, escaped text and reload.
Verification artifacts are under `~/.local/state/khata-transaction-titles/`.
