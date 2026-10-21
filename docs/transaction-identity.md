# SMS duplicates and incoming credits

Multiple SMS alerts can describe one payment. The device ID identifies the
message, not necessarily the financial event. Missing bank references previously
allowed two different messages about one payment into totals.

Migration 0035 keeps excluded records, their client IDs and a link to the
retained transaction. They are sync tombstones: ordinary sync updates and SMS
rescans cannot restore them. The explicit restore action clears that link.
Financial summaries, transaction history, budgets and predefined chat queries
exclude deleted records. Sync downloads include them so devices learn exclusions.

New imports automatically exclude a repeated event only when a named bank,
10–20 digit numeric reference, amount, direction and date within two days match
one retained record. No-reference lookalikes are never automatically merged.
A unique opposite-direction pair with that reference and amount in two different
named banks is classified as a transfer between own accounts. Both sides remain
visible while leaving totals. Existing unambiguous pairs are repaired once;
later manual transfer overrides remain effective.

Web and Android Transactions offer **Possible duplicates**. Suggestions share
bank, value date, payee, amount, direction and reference (including both missing).
This is evidence for review, not proof: recurring purchases can look identical.
Expand a web row, or use the Android row menu, and select **Exclude duplicate…**
only after confirming it represents the same payment. **Undo** restores the last
excluded entry. The API requires another matching live row and enforces ownership;
it never excludes the last remaining record in a group. Older aliases are
reparented when another member of a group is excluded.

**Money In** means all recorded non-transfer credits, including refunds. It does
not establish salary, profit or lifetime earnings. Add own-account identifiers
under Accounts and mark remaining own-account movements as transfers; classify
receipts before using them to infer income.

Regression coverage in `backend/tests/card_repayments.rs` checks reference
matching, separate tenants, ambiguous references, deleted totals, sync tombstones,
manual overrides, migration repair, duplicate review and undo. Android
`ServerSnapshotTest` checks that a downloaded exclusion survives SMS rescanning.

Build backend and web artifacts in the existing Backend Tests workflow. Signed
APKs use the existing Build Android APK workflow. Deploy verified artifacts only,
with a fresh encrypted database backup; retain source rows rather than deleting
lookalikes in bulk.
