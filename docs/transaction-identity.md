# SMS duplicates and incoming credits

Multiple SMS alerts can describe one payment. The device ID identifies the
message, not necessarily the financial event. Missing bank references previously
allowed two different messages about one payment into totals.

Migration 0035 keeps excluded records, their client IDs and a link to the
retained transaction. They are sync tombstones: ordinary sync updates and SMS
rescans cannot restore them. The explicit restore action clears that link.
Financial summaries, transaction history, budgets and predefined chat queries
exclude deleted records. Sync downloads include them so devices learn exclusions.

Reference matching automatically excludes a repeated event when a named bank,
10–20 digit numeric reference, amount, direction and date within two days match
one retained record. Other no-reference lookalikes need anchored SMS evidence or
a complete unique card repayment chain; equal amounts alone never exclude a row.
A unique opposite-direction pair with that reference and amount in two different
named banks is classified as a transfer between own accounts. Both sides remain
visible while leaving totals. Existing unambiguous pairs are repaired once;
later manual transfer overrides remain effective.

Web and Android Transactions offer **Possible duplicates**. Suggestions include
same-bank/day/payee/amount/direction/reference matches (including missing references)
and card confirmations whose dates can differ by up to two days. Recurring
purchases can also look identical, so suggestions require review.
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

SMS imports now use the exact message as their description, including whitespace
and line breaks. This text syncs to the server and web like other descriptions.
The separate provenance retains receipt time, dates, hashes and parsed payee;
it does not duplicate the SMS text. Automatic no-reference
matching requires identical normalized text containing an account plus balance
or transaction date/time, the same bank/amount/direction/day, and receipt within
two minutes. Different known references and generic SMS lookalikes stay separate.
Older records gain this evidence on rescan; existing records without sufficient
evidence remain in Possible duplicates for review.

UTR and UPI slash references are now extracted, and sync updates persist newly
extracted references. Rescanning preserves previously acknowledged client IDs,
including old parser identities and exclusions. The device database upgrades
from schema 1 to 2 with a nullable metadata column, without deleting queued work.

Verified follow-up on 2026-10-21: backend and web run
[39552892838](https://github.com/sahuishan01/Khata/actions/runs/39552892838)
passed 181 backend tests (one ignored), Clippy and the web build. Migration 0036
is live. Production checks confirmed no future SMS dates, retained original
date values, correct totals, sync exclusions, authentication and forced RLS.

The signed
[v0.45.14 APK](https://github.com/sahuishan01/Khata/releases/tag/v0.45.14)
passed 40 release tests and the minified-field check in
[run 39574262637](https://github.com/sahuishan01/Khata/actions/runs/39574262637).
Its signature matches v0.45.12 and its version code is 198. v0.45.13 was withheld
when artifact inspection found that R8 removed the SMS DTO's Gson fields.
`SmsSource` lives outside the API model package, so it has an explicit ProGuard
keep rule. The workflow checks these fields in the final DEX files before
publishing. Invalid legacy metadata is ignored during upload and rebuilt by
rescan, preserving the financial row. Physical-device UI checks remain unavailable.

## Card repayment confirmations and scrolling

Transactions load further pages as the list approaches its end on Android and
web. Requests retain search, sort, category, date and duplicate filters. Failed
pages retain loaded rows and expose retry; stale responses cannot append to a
new filter. The API orders tied rows by their unique IDs. Background web refresh
checks the first page before reloading the accumulated range, preserving scroll
position without repeatedly downloading an unchanged history.

A unique, closely timed funding debit and card receipt establish the two transfer
legs. Funding can be a CRED payment or an explicit card-bill payment, including
payment to a card issued by a different bank. Both legs remain visible as
transfers and do not count as spending or income. CRED purchases without a
unique receipt remain expenses.

Reconciliation runs after the complete upload batch and also when a generic
confirmation arrives in a separate upload. Receipt and confirmation dates can
differ by up to two days. Explicit receipt/confirmation roles and the same card
fingerprint identify later acknowledgements even if the funding account is
missing. An explicit confirmation arriving within seconds of the receipt does
not count as a second receipt when establishing uniqueness.

Older messages without card fingerprints require a complete unique chain:
one recorded funding debit, one receipt within ten minutes, one later generic
credit in the same bank, valid body dates, and a delay of ten minutes to
48 hours. Different known references/card identifiers, ambiguous funding,
multiple candidate credits and explicit manual choices block automatic
exclusion. Cases without enough evidence remain in Possible duplicates,
including candidates on an adjacent date. Raw SMS stays on the device.

Excluded confirmations retain their source identity, canonical link and an
internal `card_confirmation` reason. Explicit Undo sets a persistent override;
subsequent reference/hash updates and card reconciliation preserve that choice.
The API requires another matching live row for a manual exclusion and retains
the same ownership checks on both clients.


Verified in Android [v0.45.15](https://github.com/sahuishan01/Khata/releases/tag/v0.45.15)
(run `39581391908`, 46 tests, versionCode 199, signing certificate matching
v0.45.14) and backend/web run `39582103790` (commit `76d658a`, 187 tests passed,
one ignored, Clippy passed). Migration 0037 is deployed; its backfill drains
deferred row events before restoring forced RLS. Tests cover a populated
backfill, a complete sync batch with ambiguous funding, card fingerprints,
manual transfer choices and stable ordering across tied pages. Browser checks
cover phone/desktop pagination, retry, stale responses, refresh and Undo; live
checks verify complete history paging, summary totals, preserved source IDs,
reported confirmation exclusions and device sync tombstones. Evidence paths
are listed in `DEPLOYMENT.md`. Physical-phone UI verification remains pending.


The general reconciliation follow-up is deployed with migration 0038 from the
successful backend job in
[run 39587141836](https://github.com/sahuishan01/Khata/actions/runs/39587141836)
(commit `f4149f0`): 194 backend tests passed, one ignored, and Clippy, artifact
generation and the web build passed on the intended `heaven` runner.
Regression cases cover multiple banks and amounts, direct bill payments,
separate upload batches in either order, adjacent dates, immediate explicit
confirmations, missing funding/card identifiers, conflicting references,
ambiguous credits and persistent Undo. Live checks preserved every original
source identity and financial field, verified all history pages and sync
exclusions, and compared summary totals against the database. The existing
Android v0.45.15 and web clients consume these shared API results.

## Retried device uploads and conflict revisions

A lost upload response or a rebuilt device cache can leave a CREATE queued with
`base_rev=0` after the server has saved it. Identical fields now acknowledge the
existing record instead of demanding conflict resolution. An identical SMS may
also acknowledge the server's automatic transfer classification or duplicate
exclusion; it cannot replace a manual transfer override. Different amounts,
dates, references, notes, categories or SMS evidence still require review when
the base revision is stale.

A replay advances only the download cursor. This makes the canonical row visible
to a phone that already pulled past it while keeping its dirty local copy. The
financial fields, conflict revision, timestamps, source IDs and exclusions stay
unchanged. This recovery also works in v0.45.15 without an APK update.

Push and explicit resolution flush deferred reconciliation before returning
acknowledged revisions or conflict snapshots. Earlier rows changed by later rows
in the batch therefore return their final committed revision. Owner locks also
serialize concurrent create retries. Regression tests cover 405 queued records,
canonical re-download, preserved exclusions, manual overrides, actual stale
edits, concurrent requests, tenant isolation and final reconciliation revisions.

Verified server rollout: [39597159197](https://github.com/sahuishan01/Khata/actions/runs/39597159197) passed
202 backend tests (1 ignored), Clippy and the web build.
Production replay acknowledged 228 existing records with zero conflicts and
unchanged stored financial fields, revisions, timestamps and exclusions. All
2959 source records remain in paged sync. Android v0.45.15 can
retry its pending uploads without another scan or data clear.
