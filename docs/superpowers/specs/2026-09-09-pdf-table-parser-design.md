# PDF statement parser: coordinate-aware table reconstruction

**Date:** 2026-09-09
**Status:** Approved (design sections 1–3 approved in chat; spec-review gate waived by user)

## Problem

`src/ingest/parse.rs::parse_pdf` extracts text with `pdf-extract` (no layout
information), then splits each line on whitespace and assumes the numbers appear
in `debit, credit, balance` order. Observed failures on real statements:

- **Wrong amounts** — UPI references, cheque numbers and tokens like `NEFT-00123`
  inside the description parse as `f64` and are taken as amounts.
- **Wrong sign** — a credit-only row's single number is always recorded as a
  debit (`[amt] => (Some(*amt), None, None)`).
- **Missed transactions** — `is_date` only fires when the first token contains
  `/` or `-`, so `01 Jan 2024`-style dates and rows that wrap before the date
  are dropped.
- `pdf-extract` also glues adjacent columns together on CID-font PDFs
  (`500.0012,345.67`).
- The `BankProfile` is entirely unused in the PDF path.

Credit-card statements (e.g. the user's) have a different shape: `Txn Date |
Posting Date | Description | Amount` with a `Cr`/`Dr` suffix and **no running
balance**.

## Approach

Replace text-dump parsing with coordinate-aware extraction plus table
reconstruction, driven by per-bank column profiles.

```
parse_pdf(bytes, profile)
  1. pdfium-render → per-page positioned words {text, x0, x1, y0, y1, page}
  2. row clustering   → group words into visual rows by Y proximity (page-aware)
  3. header detection → find the row whose words match the profile column
                        regexes; derive column X-bands from those word positions
  4. column assignment→ bucket each word into a band → cells keyed by ColKind
  5. row assembly     → date-anchored; a row with no date in the date band is a
                        continuation line → append to previous row's description
  6. amount parsing   → only from Debit/Credit/Amount bands; handle Cr/Dr,
                        trailing '-', parentheses, commas, ₹/Rs; never parse
                        numbers from the Description band
  7. reconciliation   → confidence score (see below)
  8. fallback         → if pdfium errs, OR < 2 rows, OR confidence very low,
                        fall back to the legacy line-heuristic; flag low-confidence
```

Library: **`pdfium-render`** with a vendored prebuilt `libpdfium.so`
(permissive licence, clean Alpine deployment). It is touched in exactly one
module (`pdf/extract.rs`).

## Modules

New directory `src/ingest/pdf/` (parse.rs is already large; PDF logic moves out):

| File | Responsibility |
|---|---|
| `mod.rs` | `pub fn parse_pdf(bytes, &BankProfile) -> Result<(Vec<RawRow>, Vec<String>, String)>` — orchestrates steps 1–8, owns fallback + confidence. Same return tuple as today. |
| `extract.rs` | `Word { text: String, x0,x1,y0,y1: f32, page: usize }`; `fn words(bytes: &[u8]) -> Result<Vec<Word>>`. Only place `pdfium-render` is referenced. |
| `table.rs` | `fn rows(Vec<Word>) -> Vec<Vec<Word>>` (Y-clustering, page-aware); `Band { kind: ColKind, x0: f32, x1: f32 }`; `fn columns(header: &[Word], &BankProfile) -> Option<Vec<Band>>`; `fn cells(row: &[Word], &[Band]) -> BTreeMap<ColKind, String>`. |
| `rows.rs` | `fn assemble(Vec<BTreeMap<ColKind,String>>, &BankProfile) -> Vec<RawRow>` (date-anchoring, continuation merge); `fn amount(cell: &str) -> Option<Signed>` where `Signed { value: f64, sign: Option<Sign> }`. |
| `reconcile.rs` | `enum Confidence { Ok, Low(String) }`; `fn confidence(&[RawRow], StatementKind) -> Confidence`. |
| `legacy.rs` | today's `parse_pdf` body verbatim, reached only via fallback. |

`src/ingest/parse.rs`: `parse_pdf` becomes a one-line delegate to
`super::pdf::parse_pdf`. CSV/Excel paths untouched. `parse_file` signature and
all callers unchanged.

## Profile changes

`src/ingest/profiles/mod.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatementKind { Account, CreditCard }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColKind { TxnDate, ValueDate, Description, Debit, Credit, Amount, Balance, Ref }

pub struct PdfColumn {
    pub kind: ColKind,
    pub headers: &'static [&'static str], // lowercased substrings; header word matches if it contains one
}

// added to BankProfile:
pub statement_kind: StatementKind,   // default Account
pub pdf_columns: &'static [PdfColumn],
```

- Account profiles (HDFC, ICICI, SBI, Axis, generic): columns
  `[TxnDate, ValueDate?, Description, Debit, Credit, Balance, Ref?]` using each
  profile's existing alias lists as the header substrings.
- A `CreditCard` variant profile per bank is **out of scope for v1**; instead the
  generic profile gains a `CreditCard` sibling `generic_cc` with columns
  `[TxnDate, ValueDate (posting date), Description, Amount]`. Bank detection adds
  a CC check: if the text contains "credit card" / "statement of account" and no
  balance column header is found, select `generic_cc`.
- For a `CreditCard` statement, `amount()` sign handling: `Cr` suffix → credit,
  `Dr` or no suffix → debit.

## Header / band detection

- `columns()` scans candidate rows (first ~15 rows of each page) for the row
  maximising matched `PdfColumn.headers`. Require ≥ 3 matches to accept.
- Each matched header word contributes a band centred on its x-range; band
  boundaries are the midpoints between adjacent header word centres, with the
  first band extending to `-inf` and the last to `+inf`.
- Repeated per-page headers: detected the same way on each page; if a page has no
  header row, reuse the previous page's bands (statements keep column x-positions
  stable across pages).

## Row assembly

- A row is a *transaction start* if its `TxnDate` band contains a token parseable
  by any `profile.date_formats`.
- Rows before the first transaction start (preamble) and after a detected totals
  row (`Description` cell matches `total|closing balance|grand total` and date
  band empty) are skipped.
- Non-start rows: their `Description`-band text is appended (space-joined) to the
  current row's `description`; numeric-band text on a continuation row is
  appended only if the current row's corresponding field is still `None` (handles
  amount-on-next-line layouts).

## Amount parsing (`rows::amount`)

Accept: `1,23,456.78`, `123456.78`, `₹1,234`, `Rs. 1,234`, `1,234.00 Cr`,
`1,234.00 Dr`, `(1,234.00)` (→ negative), `1,234.00-` (trailing minus →
negative), `-1,234.00`. Reject: empty, pure alpha, bare ref numbers with no
decimal *when the band is Description* (Description is never parsed for amounts,
so this is naturally handled). Returns `None` on unparseable.

## Confidence (`reconcile::confidence`)

- **Account:** iterate rows with a known `balance`; count rows where
  `|prev_balance + credit - debit - balance| > 0.01`. `Low("balances do not
  reconcile (N/M rows)")` if mismatches > 10% of checked rows. Also
  `Low("only N transactions found")` if `rows.len() < 2`.
- **CreditCard:** `Low` if any row lacks a parseable amount; if the text contains
  a "total amount due" / "total dues" figure, `Low` if
  `|sum(debit) - sum(credit) - total_due| > 1.0`.

## Fallback & surfacing

- `pdf::parse_pdf` runs the new pipeline; on `Err` from `extract::words`, on
  `columns() == None`, or on `< 2` assembled rows, it runs `legacy::parse_pdf`
  and tags confidence `Low("<reason>; used fallback parser")`.
- The chosen `(rows, confidence)` propagate up: `parse_pdf` and `parse_file`
  gain a 4th tuple element, `Option<String>` (the low-confidence reason, `None`
  when `Confidence::Ok`). CSV/Excel paths always return `None`. Callers updated:
  - `src/ingest/handlers.rs` upload: add `warnings: Vec<String>` to
    `UploadResponse` (`src/ingest/models.rs`); populate from the reason.
  - `src/ingest/email/run.rs`: on `Some(reason)`, `counters.push_error(
    "parse-confidence", filename, &reason)` — non-fatal, not the `last_error`
    banner.
- Frontend (`frontend/src/pages/UploadPage.tsx`): render `warnings` as an amber
  banner after import.
- Android: `UploadResponse` model gains `warnings`; shown as a toast/side note.
  (Android change is small; include it.)

## Deployment

- `backend/Dockerfile`: add `libpdfium.so` — download a pinned
  `pdfium-binaries` release in the build stage, copy into the runtime image at
  a path set by `PDFIUM_LIB_PATH` (env var read at startup; `extract.rs` calls
  `Pdfium::bind_to_library` with it, falling back to `bind_to_system_library`).
- Local dev: `PDFIUM_LIB_PATH` documented in `backend/README` / `AGENTS.md`;
  a `just fetch-pdfium` / script helper drops it in `backend/.pdfium/`.
- `.gitignore`: `backend/.pdfium/`.
- Startup: attempt the bind once, log `pdfium: loaded` or
  `pdfium: unavailable (<err>) — PDF parsing will use the fallback parser`.

## Testing

Synthetic fixtures generated with `printpdf` (pure Rust, permissive) — no PII.
`backend/tests/pdf/` :

- `gen.rs` helper: build a PDF from a table spec (headers at given x-positions,
  rows, optional multi-line narration, optional repeated per-page header,
  configurable date format, Cr/Dr markers).
- Fixture cases (each: generated PDF bytes + expected `Vec<RawRow>` + expected
  `Confidence`):
  1. `account_hdfc_basic` — 5 rows, `DD/MM/YY`, debit+credit+balance, balances
     reconcile.
  2. `account_spaced_date` — `01 Jan 2024` dates (regression: legacy missed
     these).
  3. `account_multiline_narration` — 3 txns, narration wraps to 2–3 lines.
  4. `account_number_in_desc` — description contains a 12-digit UPI ref and a
     `NEFT-000123` token; amounts must be unaffected (regression: wrong amount).
  5. `account_credit_only` — credit-only rows; must be `credit`, not `debit`
     (regression: wrong sign).
  6. `account_repeated_header` — 2 pages, header row on both.
  7. `account_broken_balance` — one balance deliberately wrong → `Low`.
  8. `cc_generic` — CC layout, `Cr`/`Dr` suffixes, no balance, a "Total Amount
     Due" line that matches the row sum.
  9. `garbage_pdf` — text with no recognisable header → fallback + `Low`.
- Unit tests: `rows::amount` table (all accepted/rejected forms);
  `table::columns` band boundaries; `reconcile::confidence` both kinds.
- `extract::words` against a checked-in tiny generated PDF is `#[ignore]`d
  unless `PDFIUM_LIB_PATH` resolves (same pattern as the existing
  qpdf-gated test).
- Keep every existing `parse.rs` test green.

## Out of scope

- Per-bank CC profiles (only `generic_cc` in v1).
- OCR for scanned/image-only PDFs (detected → fallback + `Low`).
- Re-parsing already-imported statements (user re-uploads if they want the
  better parse).
- LLM-assisted parsing.

## Build sequence

1. profile types (`StatementKind`, `ColKind`, `PdfColumn`, `BankProfile`
   fields) + populate all profiles + `generic_cc` + CC detection.
2. `pdf/extract.rs` + `pdfium-render` dep + `PDFIUM_LIB_PATH` bind + Dockerfile.
3. `pdf/table.rs` (rows, columns, cells) + unit tests.
4. `pdf/rows.rs` (assemble, amount) + unit tests.
5. `pdf/reconcile.rs` + unit tests.
6. `pdf/legacy.rs` (move current body) + `pdf/mod.rs` orchestration + fallback.
7. 4th tuple element through `parse_file`; `UploadResponse.warnings`; handler +
   email worker + frontend + Android surfacing.
8. `tests/pdf/` fixtures via `printpdf` + full integration tests.
9. Manual check against a real statement if the user can provide one; else ship
   on synthetic coverage.
