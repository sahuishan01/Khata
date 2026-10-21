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

Verified deployment on 2026-10-21: backend and web artifacts from
[Backend Tests run 39539446022](https://github.com/sahuishan01/Khata/actions/runs/39539446022)
passed 175 backend tests (one ignored), Clippy and the web build. Browser checks
passed on desktop and mobile layouts, including automatic refresh and review/undo.
Production checks verified summary totals, retained sync tombstones, authentication,
row-level security and public asset hashes after an encrypted database backup.

[v0.45.12](https://github.com/sahuishan01/Khata/releases/tag/v0.45.12) passed 31 Android
release unit tests in
[run 39540420171](https://github.com/sahuishan01/Khata/actions/runs/39540420171).
The downloaded APK has version code 196 and matches the v0.45.10 signing certificate;
its ZIP integrity and expected feature code were verified. Physical-device UI
validation was unavailable. Backend/web sources match the release tag.

## SMS dates and rescan evidence

SMS dates are bounded by the original message day in Asia/Kolkata. A future
body date is rejected rather than silently swapping day and month. The fallback
order is valid body date, original message date, then persisted queue date.
Clock values, due dates and reference fields must not supply transaction dates.

Migration 0036 corrects untouched, future-dated SMS imports using their server
import day when the original receipt time is unavailable. `sms_source` preserves
the original dates and identifies this queue fallback. An inbox rescan replaces
that fallback with the message date, or a valid transaction date from the SMS.
Ordinary past manual edits and deleted aliases are preserved.

The app retains message receipt time, date provenance and SHA-256 hashes, without
uploading raw SMS text, sender or account identifiers. Automatic no-reference
matching requires identical normalized text containing an account plus balance
or transaction date/time, the same bank/amount/direction/day, and receipt within
two minutes. Different known references and generic SMS lookalikes stay separate.
Older records gain this evidence on rescan; existing records without sufficient
evidence remain in Possible duplicates for review.

UTR and UPI slash references are now extracted, and sync updates persist newly
extracted references. Rescanning preserves previously acknowledged client IDs,
including old parser identities and exclusions. The device database upgrades
from schema 1 to 2 with a nullable metadata column, without deleting queued work.
