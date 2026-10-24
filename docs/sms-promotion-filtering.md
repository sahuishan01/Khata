# Promotional SMS filtering

Android v0.45.17 rejects loan offers such as “Get Rs.276000 credited in minutes”
at each candidate amount. Instructional wording in the governing clause marks an
offer; a completed loan credit remains a transaction. A promotional footer does
not suppress a genuine credit or debit elsewhere in the SMS.

Install the signed APK as an update, then open **Add Data → Scan SMS inbox**.
The original SMS must still be in the inbox. New live capture and historical
scans share the same parser.

The rescan also corrects a previously imported offer when all safeguards pass:

- The old parser accepts the SMS and the corrected parser rejects it entirely.
- The stable client ID, stored SMS hash and original receive timestamp match.
- Amount, direction, bank, description, dates and default import notes/category
  remain unchanged; there is no reference, balance or transfer marker.
- The row has valid SMS provenance and no pending edit to a synced record.
- A Room transaction confirms the complete row is still the inspected snapshot.

The correction marks the existing row excluded and queues normal sync with its
original revision. It retains financial fields and provenance, rather than
overwriting an amount or creating another transaction. Repeated rescans are
idempotent. Successful sync removes the false credit from normal Android/web
history and totals. Upload failures remain queued and visible for retry.

Edited, referenced, already excluded and legacy records without matching proof
are preserved for manual review. The app does not persist raw SMS text for this
repair. Regression fixtures replace the personalized offer link with a generic
suffix; the exact reported message was separately checked against compiled code.

Validation: 61 release unit tests and Android release lint pass locally. Tests
cover the reported offer, nearby wording, real loan disbursements, mixed alert and
offer messages, protected edits, missing provenance, repeated scans, concurrent
edits and revision-preserving sync. The existing backend email extractor already
rejects the reported message; its 15 parser tests pass without backend changes.

Signed release CI [40085238240](https://github.com/sahuishan01/Khata/actions/runs/40085238240)
completed successfully on `heaven`, including all 61 release tests. The downloaded
APK is version 0.45.17 / versionCode 203, with the same signing certificate as
v0.45.16. ZIP integrity, the GitHub asset digest, HTTPS-only configuration, all
nine SMS metadata fields and existing widget/tile/launcher components were
verified. Security Scan `40085238241` passed. The existing GitHub default CodeQL
Java autobuild signing failure remains separate from the signed release build.
Evidence is stored at `~/.local/state/khata-sms-promotions/`.
