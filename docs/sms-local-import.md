# Local SMS imports

From v0.45.30, live receipt and Scan SMS inbox use SmsParser on the device.
Shared posting grammar handles account Dr/Cr alerts and verbs before or after
amounts, including multiline IMPS messages. Issuer names label banks rather than
selecting bank-specific recognition rules. OTP/authentication messages, requests,
offers, future payments, balances and unsupported messages are not queued or sent.
Only locally recognized transactions are persisted for upload. Original text
stays in Notes, generated titles and whole-transaction editing are preserved,
and payment methods remain visible/editable on Android and web.

## Existing queues and ownership

The Room 3 table and kept SmsDecisionWorker class remain compatible with installed
v0.45.27-v0.45.29 jobs. They now run the local parser, with no decision HTTP call.
Old pending and review items are rechecked locally; unsupported items lose their
queued body/sender and retain a deduplication tombstone. Saved transactions are
never deleted by this conversion. New upload conflicts remain available for review.

Workers capture their account, server and token together. An account-scoped paged
server snapshot supplies identity and edit checks; a shared device cache never
decides which account owns a saved record. Recognized transactions use the normal
bulk sync endpoint, up to 200 per request, followed by canonical row refresh.
Retries acknowledge previously saved rows, and conditional revisions protect
concurrent edits. Timestamp/legacy-identity safeguards remain in place.

## Progress

Upload and Transactions show finished/total, percentage, waiting and review counts.
Scanning remains distinct from transaction uploads; the queue is complete only
when all queued work and reviews finish. Counts survive app restart. Network
failures retain pending work for retry; there is no CPU-idle or inference wait.
These phone-inbox controls are Android-specific. Shared saved transaction editing
and payment-method controls remain available on Android and web.

## Server

The Laya service is stopped and disabled. Its Python deployment and policy CI
were removed. The retired /api/sms/decide endpoint returns a terminal client
error without contacting a model; old apps need the v0.45.30 update. Existing
lookup/import endpoints and historic classifier provenance remain compatible.
No database migration, financial-record backfill or model download is required.
