# Background SMS decisions

Live SMS receipt and **Add Data → Scan SMS inbox** use the same persistent Android
WorkManager queue. Laya first decides whether a message reports a completed
transaction. Only a positive transaction decision asks the second question: did
money leave the account described by the SMS? A negative debit decision means
credit only when the transaction decision and direction are both sufficiently
clear. An uncertain answer never defaults to credit.

The source text stays in Notes and the title remains **Auto-captured via SMS
(bank)**. The existing whole-transaction editor continues to work on Android and
web. Payment method (IMPS, NEFT, RTGS, UPI, ATM, card, cheque or cash) is extracted
when present and shown in transaction details on both clients. It can be corrected or cleared
through Edit transaction. Missing methods
remain unknown. Spending categories still use existing categorization rules.

## Privacy and ownership

Before queueing or networking, Android requires a currency marker (`INR`, `Rs`
or `₹`) next to a numeric amount. This generic gate does not depend on the bank,
sender or posting wording. Monetary offers can still pass this gate; Laya and
the extractor decide whether they describe a completed transaction. Ordinary
personal texts, delivery codes and other messages without a currency amount stay
on the device. The gate supports prefix/suffix currency, Indian digit grouping,
line breaks and Unicode spaces.

Messages containing `otp` (case insensitive), one-time passwords/passcodes/codes,
login codes, verification codes or authentication codes are excluded before
insertion into the Android decision queue and before any decision API call.
The Rust gateway and local model service independently repeat that check for
older or custom clients. Debug networking logs only request metadata.

Queue identities include the signed-in user and server, original sender,
timestamp, full message hash and decision policy. Inbox scans use `date_sent`, the same SMS-centre timestamp
as live capture, rather than the often slightly later inbox arrival time.
Missing or invalid sent times fall back to arrival. Older arrival-based imports
are reused only when their original body hash and receipt timestamp verify;
custom edits stay intact, and unverified identity conflicts require review.
WorkManager input contains
only the owner hash. Network work captures its original server and bearer token
together, and every endpoint verifies that the requested owner matches the
authenticated user. The worker reads existing transaction candidates through an
owner-scoped server lookup, rather than trusting a device cache left by another
sign-in. Edits and exclusions are preserved by the existing conditional sync
protocol. Server changes while processing cause a review conflict.

Old-version pending and review jobs are checked against the current gate before
any decision request. Ineligible queued bodies and senders are discarded locally;
the tombstone remains to avoid repeat work. Saved transactions and custom edits
are not deleted. Filtering is scoped to the current user and server.

Offline and transient failures remain pending with exponential retry. A batch
that made progress starts a fresh work item so successful partial batches do not
accumulate hours of backoff. Opening the app, scanning again or **Retry SMS
processing** restarts pending work without resetting its saved decisions. A changed
sign-in stops that owner's work; signing back in resumes it. Successfully
processed or ignored jobs retain a deduplication tombstone and discard queued
text/sender. Undecided jobs retain their original text privately on the device
for review. Review lets the user confirm amount and direction, retry, or ignore.
Clear All Data also clears the local decision queue.

## Progress

Inbox scan counts distinguish messages read, queued for classification, skipped
locally and already handled. Queueing does not mean a transaction was recognized
or imported. The persistent classification summary shows exact pending, review,
accepted, filtered and non-transaction counts on Upload and Transactions. Accepted
SMS may reuse an existing transaction, so this count is not a count of new rows.
Classification is complete only when pending and review counts are both zero.

Both screens also show **X of Y queued SMS finished (Z%)** and an overall progress
bar. The total includes every queued SMS for the current account/server on this
device, including previously finished work. Accepted, ignored and locally filtered
jobs count as finished; pending and review jobs remain unfinished. Percentages
round down so an outstanding message cannot display 100%. Counts survive app
restarts and do not reset on retries or rescans. While an inbox scan is still
adding jobs, the bar is indeterminate and explains that the total is growing;
completion is shown only after scanning and all queued work are finished.

Saved transaction uploads have a separate sync status. Their completion cannot
complete SMS classification. Transactions and dashboard totals refresh as SMS
are accepted, including when a later message is waiting for a connection or
server capacity. No inbox rescan or screen restart is required to see partial
results. These phone-inbox controls are Android-specific; saved transactions and
the whole-transaction editor remain available on Android and web.

## Model and validation

The runtime uses upstream [Laya](https://github.com/NandhaKishorM/laya), Python
package `laya==0.4.15`, CPU PyTorch, and the English
[`convaiinnovations/laya`](https://huggingface.co/convaiinnovations/laya)
checkpoint pinned to `7b928d828b7b0e022f929d9bd2e44165aa270148`.
Weights are installed locally; inference runs offline on this VPS. No SMS goes
to Hugging Face or an external inference provider. The Rust backend authenticates
requests and forwards only to a literal loopback HTTP address.

Policy `sms-choice-v1` uses typed yes/no choices. Transaction probability at
least 0.75 proceeds to direction; at most 0.2 is ignored; other values require
review. The second decision averages three phrasings of the same debit question
to reduce sensitivity to one phrasing. Direction needs a maximum averaged choice
probability of at least 0.55. These scores are model outputs, not a measured guarantee of accuracy for bank SMS.
The tested checkpoint still incorrectly labels some advertisements and future
payments as transactions. Therefore model decisions alone never save money.

The Android extractor requires a positive, finite amount read from the SMS, with
at most two decimals, and rejects contextual request/future/failed-payment,
application/fund-block and offer clauses. Balance and limit amounts are not
posting amounts. A known posting or direction that contradicts the model goes to review.
Unknown wording needs completed-payment evidence and one unambiguous amount;
otherwise it goes to review. The generic grammar accepts posting verbs before
or after the amount. Messages longer than 2000 characters also require review
to bound inference work without truncating their contents. The grammar works
across spaces and line breaks, including multiline IMPS alerts. Issuer aliases label banks and do not select bank-specific templates.

## Operations

On a Python 3.13 Linux host with `uv`, run `sudo bash ops/laya/install.sh`.
Dependencies are pinned in `ops/laya/requirements.txt`; CPU wheels come from the
official PyTorch index. The installer downloads only the pinned English
checkpoint, installs the service under `/opt/khata-laya`, and stores its local
model under `/var/lib/khata-laya/model`.

`khata-laya.service` runs as a dedicated unprivileged user on `127.0.0.1:8091`.
It processes one SMS at a time with two CPU threads, a 150% CPU quota, a 2500 MiB
memory soft limit and 3500 MiB hard limit. Network access is restricted to
loopback. Inference starts only when `/proc/stat` reports at least 80% CPU idle;
busy requests remain queued for retry. Filesystem protections are enabled,
access logs are disabled, and
errors do not include message bodies. Health: `GET http://127.0.0.1:8091/health`.
The API gateway defaults to that service's `/decide` route; `KHATA_LAYA_URL` may
override it only with another literal loopback HTTP address. No public model
port or extra reverse proxy is needed.

API routes `POST /api/sms/decide`, `/lookup` and `/import` all require normal
Khata authentication. `/decide` binds the response to SHA-256 of the exact
request text and bounds response size, latency and concurrency. `/lookup`
returns only the authenticated owner's matching transaction. `/import` uses
conditional sync and returns the canonical saved row; identical retries are
idempotent. Model version and payment method are optional SMS provenance fields,
retained for old clients' unchanged replays. No backend database migration is
needed. Android Room migrates from version 2 to 3 without changing transactions.

## Verification

Run Python policy tests with
`python -m unittest discover -s ops/laya -p 'test_*.py'` in the service environment,
Android release unit tests and lint, backend tests with an isolated PostgreSQL
database, Clippy, and the web build. Native emulator checks must include SMS
receipt, OTP exclusion, process/offline recovery and manual review; host JVM
regex tests cannot prove Android regex compatibility. Use synthetic SMS in
public tests. Exact user reports belong only in private local regression data.
