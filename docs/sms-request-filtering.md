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
