# Pending payment requests

Android v0.45.19 distinguishes requests and future payments from completed
postings. “UPI mandate request received for Rs.129600” describes receipt of a
request, not a credit. Pending approvals and “will be debited” alerts are also
excluded. Completed mandate payments and refunds remain eligible for import.

The filter checks clauses around each amount match. A request elsewhere in the
same SMS does not replace or suppress a completed posting. Sentence, semicolon
and line boundaries are respected; the abbreviation “Rs.” is not a sentence
boundary. Existing loan-offer filtering remains in place.

Install the signed APK as an update, then run **Add Data → Scan SMS inbox** with
the original SMS still present. A newly rejected false import is excluded only
when its original parser result, stable identity, body hash, timestamp and
unchanged fields can be verified. Edited, referenced, transfer and unverifiable
records are preserved. Normal revision-protected sync distributes exclusions to
the web and other devices without replacing financial fields or identities.

The exact reported HP Telecom approval-request SMS was already rejected by
v0.45.18. A read-only lookup found no matching provenance hash or matching
company/amount entry on this server. Its actual import path and the installed
phone version remain unconfirmed. Do not claim this specific saved entry has
been removed, fabricate its old parser result, or exclude a row by amount alone.
These checks do not modify live user transactions.

Related request-received and future-debit variants reproduced three failing
tests before the fix. All 69 Android release tests and release lint now pass.
Seven new tests cover the exact example, pending request variants, future debits,
completed mandate postings/refunds, mixed messages, proven rescan exclusions,
idempotence and protection of renamed/unverifiable rows. The unchanged backend
email extractor already rejects these request messages; all 15 tests in its
standalone parity harness pass. No web/backend deployment or migration is needed.

Evidence: `~/.local/state/khata-sms-requests/`.

Signed release CI [40108105231](https://github.com/sahuishan01/Khata/actions/runs/40108105231)
completed successfully on `heaven`, including all 69 release unit tests and
minified SMS metadata checks. The APK is versionCode 205 and its signing
certificate matches v0.45.18. ZIP integrity, the GitHub asset digest, request and
future-posting filters, required metadata, HTTPS-only configuration and retained
title/widget/tile/launcher components were verified. Security Scan `40108105133`
passed. The existing default CodeQL Java unsigned-autobuild configuration issue
remains separate from signed release CI. See `DEPLOYMENT.md` for artifact details.

## IPO application receipts (v0.45.21)

The reported Jio Platforms IPO acknowledgement reproduced a false credit of
Rs 107424.00 in v0.45.19: the application amount was followed by “received”,
while the collect request and blocking-of-funds instruction appeared in the next
sentence. Application receipt is not money received. No application/VPA identity
is needed to distinguish the wording; regression fixtures use neutral examples.

The parser now checks request/application/fund-block context across the SMS.
In that context, a receipt clause needs its own evidence of money posted, such
as credited/debited/refund wording or an identified account. A separate real
posting cannot legitimize the application amount. Completed IPO debits/refunds
and real postings in other clauses remain eligible. Fund blocks are excluded.
Case, currency/identifier abbreviations, digit grouping and line wrapping are
covered. This is shared by live SMS capture and explicit inbox scans.

Install the update and run **Add Data → Scan SMS inbox** while the source SMS
remains present. The existing guarded exclusion verifies the previous parse,
identity, body hash, receipt time and unchanged fields before queuing a soft
exclusion through revision-protected sync. Renamed, referenced, transfer or
unverifiable records remain available for review. No live financial data is
modified during development. The backend email extractor already rejects the
reported wording; no backend/web deployment or migration is needed.

Evidence: `~/.local/state/khata-sms-ipo/`.

All 77 Android release unit tests and release lint pass locally. Eight new tests
cover the reported wording, application/currency/line-wrap variants, fund blocks,
completed IPO payments/refunds, mixed messages, guarded rescan exclusion,
idempotence and edited/unverifiable row protection. The pre-fix baseline had six
failures in the seven initial tests. A private direct JVM check also rejects the
exact unmodified user-provided SMS for three sender cases. All 16 backend
email-parser parity tests pass using a neutral alert subject.

A separate regression preserves genuine receipts with a “download/use our mobile
application” footer. The context guard recognizes application identifiers and
acknowledgements rather than the generic word “application”. The extra regression
failed before narrowing the pattern. The unpublished v0.45.20 CI build was
cancelled; v0.45.21 contains the complete fix.

Signed CI [40114558787](https://github.com/sahuishan01/Khata/actions/runs/40114558787)
passed all 77 release tests on `heaven`. The downloaded v0.45.21 APK has versionCode
207 and matches v0.45.19's signing certificate. Archive integrity, GitHub asset
digest, application/fund-block guards, required SMS metadata, HTTPS-only setup
and retained title/widget/tile/launcher components were verified. Security Scan
`40114558309` passed. The existing default CodeQL Java unsigned-autobuild failure
remains separate from successful signed release CI. See `DEPLOYMENT.md` for
artifact links and checksum.
