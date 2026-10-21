# Statement parsing and comparison

Before importing statements, preserve the existing SMS history as a frozen
comparison baseline. The current baseline reference and capture time are in the
root-only `~/.local/state/khata-baselines/comparison-session.json`. Follow its
pinned manifest, rather than replacing it with a later snapshot.

## Protected baseline

The baseline directory contains a full PostgreSQL custom-format backup and a
JSON comparison export from the same exported, read-only database snapshot.
The export preserves transaction IDs, dates, amounts, references, raw descriptions,
transfer and duplicate flags, excluded source rows, SMS provenance, statement
records, accounts and category definitions. New SMS can arrive during capture
without changing the saved snapshot.

Archives use AES-256-GCM in authenticated 1 MiB chunks, with a separate derived
key per file. Their data key is wrapped by the host's `systemd-creds` key. The
directory is mode 0700; archive, wrapped-key and manifest files are mode 0400.
No plaintext financial export is written to disk or committed to this repository.

Verify hashes, decryption, PostgreSQL archive structure and comparison counts:

```sh
python3 ~/.local/state/khata-baselines/archive.py /path/from/the/pinned/manifest
```

The private helper's `load(directory)` function decrypts both payloads into
memory for analysis. The host credential key is required. Never print payloads
to chat, upload them to CI, or restore the backup over the live database to run
a comparison. Any restore for analysis must use a separate, isolated database.

## Parser findings before statement samples

Source inspection found these limitations; they are not yet a diagnosis of any
particular customer PDF:

- `profiles/registry` has HDFC and Axis account profiles, but no dedicated Kotak
  or Canara profile. Those banks use the generic account fallback.
- Credit-card detection selects `GENERIC_CC`, losing the issuer-specific bank
  label needed for reliable cross-source matching.
- Extraction reads PDF text; there is no OCR path for image-only statements.
- The coordinate parser reuses the first detected table's columns across pages.
  Changed page layouts and split headings need real-layout regression examples.
- The legacy fallback assumes a two-number amount/balance row is a debit. It
  cannot reliably distinguish a deposit on that evidence alone.
- Low confidence only adds a warning; the normal upload handler still inserts
  rows. Normalization may also skip rows with unreadable dates or amounts.
- Current PDF fixtures are generated documents. Their success does not prove
  compatibility with a particular bank's exported statement layout.

Relevant code is in `backend/src/ingest/pdf/`, `profiles/`, `detect.rs`,
`normalize.rs` and `handlers.rs`. Existing PDF tests are in
`backend/tests/pdf_parser.rs`.

## Next step and validation

Obtain one failing statement PDF privately, identify the issuer and account or
card type, and reproduce the problem without invoking the live upload handler.
That handler writes statement and transaction records; it is not a preview API.
The existing debug-headers function is not connected to the ingest router.

Check page coverage, dates, debit/credit direction, amounts, wrapped descriptions,
references and row counts against the PDF. Reconcile running balances in the
statement's actual order. Card closing dues can include an opening balance, fees
and interest, so net new transactions alone do not establish the closing amount.
Scanned PDFs need a separately validated OCR path if the supplied file requires it.

Implement fixes against the observed layout, using synthetic or properly redacted
fixtures. Keep customer PDFs, passwords and extracted financial text out of Git
and CI logs. Build, test and deploy changes through the existing CI workflows.

After parsing is verified, compare statement rows with the frozen baseline per
account, covered dates, reference, amount and direction. Report matches,
statement-only entries, SMS-only entries, duplicate alerts and field disagreements.
Retain ambiguous matches for review. Equal amounts alone do not establish a
duplicate. Preserve the baseline's own-account and card-payment treatment when
comparing totals, and keep the original snapshot unchanged throughout.
