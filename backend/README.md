# Khata Backend

Axum + sqlx (PostgreSQL) service for the Khata personal-finance tracker.

## Build & Run

```bash
cargo run                      # dev (reads ../.env)
cargo test                     # needs a test DB (DATABASE_URL) + PDFIUM_LIB_PATH for coord tests
cargo build --release
```

`#[sqlx::test]` tests create ephemeral databases from `DATABASE_URL` and run
`./migrations` themselves, so a reachable Postgres is enough:

```bash
DATABASE_URL=postgresql://khata:khata@127.0.0.1:5433/khata cargo test
```

## PDF statement parsing

PDF statements are parsed on a coordinate-aware path:

1. **`pdfium-render`** extracts positioned words (`Word { text, x0, x1, y0, y1, page }`)
   from every page. This needs the pdfium shared library — see `PDFIUM_LIB_PATH` below.
2. **Band / profile reconstruction** (`src/ingest/pdf/table.rs`): words are clustered
   into rows by `y`, then each bank's `BankProfile.pdf_columns` (a list of
   `PdfColumn { kind: ColKind, headers: &[&str] }`) is matched against the header row
   to locate vertical column bands. Each data row's words are bucketed into
   `ColKind` cells by which band their `x` range falls in.
3. **Assembly** (`src/ingest/pdf/rows.rs`): cells become `RawRow`s — amount parsing,
   multi-line description continuation, page-header / totals-line suppression.
4. **Reconciliation** (`src/ingest/pdf/reconcile.rs`): running-balance vs debit/credit
   deltas are checked to produce a `Confidence::{Ok, Low(reason)}` score.
5. **Orchestration** (`src/ingest/pdf/mod.rs`): on missing pdfium or very low
   confidence it falls back to the legacy `pdf_extract` line-heuristic
   (`src/ingest/pdf/legacy.rs`). Low confidence is surfaced as
   `UploadResponse.warnings` and a non-fatal `parse-confidence` email-sync entry.

Credit-card statements are detected in the shared `src/ingest/detect.rs` module,
which selects the `generic_cc` profile (single signed `Amount` column, `Cr`/`Dr`
suffix aware); its `statement_kind` is then read in `src/ingest/pdf/mod.rs`.

### Adding a bank's `pdf_columns`

Edit the bank's profile in `src/ingest/profiles/` (e.g. `hdfc.rs`, `icici.rs`,
`sbi.rs`, `axis.rs`; `generic.rs` / `generic_cc.rs` are the fallbacks). Add a
`pdf_columns: &[PdfColumn { kind, headers }]` array where `headers` are lowercase
substrings that appear in that bank's PDF header row. Every account profile must
cover `TxnDate`, `Description`, and money columns (`Balance` + `Debit`/`Amount`);
CC profiles need `Amount` — this is enforced by a unit test in `profiles/mod.rs`.

### Adding a fixture / test

Synthetic statement PDFs are generated in-process by `tests/pdf/gen.rs`
(`StatementSpec { cols, rows, repeat_header_pages, preamble }` → `gen::statement()`
renders an A4 PDF via `printpdf`; no PII). Add a test to `tests/pdf_parser.rs`:
build a `StatementSpec`, call `gen::statement(&spec)`, then
`parse_pdf(&pdf, &profiles::<bank>::profile())` and assert on the returned rows /
headers / low-confidence reason.

### `PDFIUM_LIB_PATH`

`tests/pdf_parser.rs` **self-skips** unless `PDFIUM_LIB_PATH` is set, so without it
the coordinate path gets zero real coverage. Local setup:

```bash
./scripts/fetch-pdfium.sh linux x64        # use arch `arm64` on ARM hosts
export PDFIUM_LIB_PATH="$PWD/.pdfium/lib"   # dir with libpdfium.so (the file also works)
cargo test
```

The pinned pdfium build is `chromium/8044`. In production the root `Dockerfile`
downloads it in the builder stage and sets
`ENV PDFIUM_LIB_PATH=/usr/local/lib/libpdfium.so`. No DB migration is involved.

Spec: `docs/superpowers/specs/2026-09-09-pdf-table-parser-design.md`
