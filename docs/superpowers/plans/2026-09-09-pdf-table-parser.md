# PDF Statement Table Parser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the layout-blind `pdf-extract` text parser with coordinate-aware table reconstruction driven by per-bank column profiles, so transactions and amounts are read correctly from bank and credit-card statement PDFs.

**Architecture:** `pdfium-render` yields positioned words per page. Words are clustered into visual rows by Y, a header row is matched against the bank profile's column regexes to derive column X-bands, every word is bucketed into a band, and rows are assembled with date-anchoring and continuation-line merging. A reconciliation pass scores confidence; on extraction failure or very low confidence the old line-heuristic runs as a fallback and the result is flagged.

**Tech Stack:** Rust, Axum, `pdfium-render` (vendored `libpdfium.so`), `printpdf` (test fixtures), existing `chrono` date parsing.

**Spec:** `docs/superpowers/specs/2026-09-09-pdf-table-parser-design.md`

## Global Constraints

- Backend is Rust 2021, builds on Alpine (musl) in a multi-stage Dockerfile.
- `parse_file` and all its callers keep working; CSV and Excel paths are untouched.
- `RawRow` struct is unchanged: `{ txn_date: String, value_date: String, description: String, debit: Option<f64>, credit: Option<f64>, balance: Option<f64>, bank_ref: Option<String> }`.
- `pdfium-render` is referenced in exactly one module: `src/ingest/pdf/extract.rs`.
- No PII in the repo: all test fixtures are generated programmatically with `printpdf`.
- Money comparisons use a tolerance of `0.01` (account balances) or `1.0` (credit-card totals).
- PDF coordinate origin is bottom-left; larger Y is higher on the page.
- Every task ends green (`cargo test`) and with a commit.
- Commit message trailer for every commit:
  ```
  Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_013HRjuk415gQzJDSDbi2xZA
  ```
- Work happens on branch `feat/pdf-table-parser` (already created).

---

### Task 1: Profile types — `StatementKind`, `ColKind`, `PdfColumn`, `BankProfile` fields

**Files:**
- Modify: `backend/src/ingest/profiles/mod.rs`
- Modify: `backend/src/ingest/profiles/hdfc.rs`, `icici.rs`, `sbi.rs`, `axis.rs`, `generic.rs`
- Create: `backend/src/ingest/profiles/generic_cc.rs`
- Test: inline `#[cfg(test)]` in `backend/src/ingest/profiles/mod.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum StatementKind { Account, CreditCard }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
  pub enum ColKind { TxnDate, ValueDate, Description, Debit, Credit, Amount, Balance, Ref }

  #[derive(Debug, Clone, Copy)]
  pub struct PdfColumn {
      pub kind: ColKind,
      /// lowercased substrings; a header word matches this column if the
      /// lowercased header-cell text contains any of them
      pub headers: &'static [&'static str],
  }

  // new fields on BankProfile:
  pub statement_kind: StatementKind,
  pub pdf_columns: &'static [PdfColumn],
  ```
- `pub fn registry() -> Vec<BankProfile>` now also includes `generic_cc::profile()` immediately before `generic::profile()`.

- [ ] **Step 1: Write the failing test**

In `backend/src/ingest/profiles/mod.rs` `#[cfg(test)]`:
```rust
#[test]
fn every_account_profile_has_pdf_columns_covering_date_desc_and_money() {
    for p in registry() {
        let kinds: Vec<ColKind> = p.pdf_columns.iter().map(|c| c.kind).collect();
        assert!(kinds.contains(&ColKind::TxnDate), "{} missing TxnDate", p.name);
        assert!(kinds.contains(&ColKind::Description), "{} missing Description", p.name);
        match p.statement_kind {
            StatementKind::Account => assert!(
                kinds.contains(&ColKind::Balance)
                    && (kinds.contains(&ColKind::Debit) || kinds.contains(&ColKind::Amount)),
                "{} account profile needs Balance + Debit/Amount", p.name
            ),
            StatementKind::CreditCard => assert!(
                kinds.contains(&ColKind::Amount),
                "{} cc profile needs Amount", p.name
            ),
        }
    }
}

#[test]
fn registry_has_a_credit_card_profile_before_generic() {
    let names: Vec<&str> = registry().iter().map(|p| p.name).collect();
    let cc = names.iter().position(|n| *n == "GENERIC_CC").expect("GENERIC_CC present");
    let generic = names.iter().position(|n| *n == "GENERIC").expect("GENERIC present");
    assert!(cc < generic);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test --lib profiles`
Expected: compile error — `StatementKind` / `ColKind` / `pdf_columns` not defined.

- [ ] **Step 3: Add the types and fields**

In `mod.rs` add the three enums/struct above. Add `statement_kind` and `pdf_columns` to `BankProfile`. Add `pub mod generic_cc;` and update `registry()`:
```rust
pub fn registry() -> Vec<BankProfile> {
    vec![
        hdfc::profile(),
        icici::profile(),
        sbi::profile(),
        axis::profile(),
        generic_cc::profile(), // credit-card fallback — before generic
        generic::profile(),    // must be last — account fallback
    ]
}
```

- [ ] **Step 4: Populate each account profile**

For `hdfc.rs` (and analogously `icici.rs`, `sbi.rs`, `axis.rs`, `generic.rs`), add:
```rust
statement_kind: StatementKind::Account,
pdf_columns: &[
    PdfColumn { kind: ColKind::TxnDate,     headers: &["date", "txn date", "transaction date"] },
    PdfColumn { kind: ColKind::ValueDate,   headers: &["value date", "value dt"] },
    PdfColumn { kind: ColKind::Description, headers: &["narration", "description", "particulars", "remarks"] },
    PdfColumn { kind: ColKind::Ref,         headers: &["chq", "ref", "cheque"] },
    PdfColumn { kind: ColKind::Debit,       headers: &["withdrawal", "debit", "dr"] },
    PdfColumn { kind: ColKind::Credit,      headers: &["deposit", "credit", "cr"] },
    PdfColumn { kind: ColKind::Balance,     headers: &["closing balance", "balance"] },
],
```
Reuse each profile's existing `*_aliases` values as the `headers` substrings (they are already lowercased alias lists). Keep `ColKind::Amount` only in `generic.rs` (some generic account statements have a single signed amount column):
```rust
// generic.rs pdf_columns also includes, after Credit:
PdfColumn { kind: ColKind::Amount, headers: &["amount", "transaction amount"] },
```

- [ ] **Step 5: Create `generic_cc.rs`**

```rust
use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "GENERIC_CC",
        detect_keywords: &[], // selected by detect_bank's CC heuristic, not keywords
        txn_date_aliases: &["date", "transaction date", "txn date"],
        value_date_aliases: &["posting date", "post date"],
        description_aliases: &["description", "transaction details", "particulars", "merchant"],
        debit_aliases: &[],
        credit_aliases: &[],
        amount_aliases: &["amount", "amount (inr)", "amount in rs"],
        balance_aliases: &[],
        ref_aliases: &["reference", "ref"],
        date_formats: &["%d/%m/%Y", "%d-%m-%Y", "%d %b %Y", "%d/%m/%y", "%d-%b-%Y"],
        skip_rows: 0,
        statement_kind: StatementKind::CreditCard,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate,     headers: &["date", "transaction date", "txn date"] },
            PdfColumn { kind: ColKind::ValueDate,   headers: &["posting date", "post date"] },
            PdfColumn { kind: ColKind::Description, headers: &["description", "transaction details", "particulars", "merchant"] },
            PdfColumn { kind: ColKind::Amount,      headers: &["amount"] },
        ],
    }
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd backend && cargo test --lib profiles`
Expected: PASS. Also `cargo build` succeeds (every `BankProfile { .. }` literal in the codebase now has the two new fields — the compiler will list any that were missed).

- [ ] **Step 7: Commit**

```bash
git add backend/src/ingest/profiles/
git commit -m "feat(ingest): add StatementKind + PDF column profiles"
```

---

### Task 2: `pdfium-render` dependency, `libpdfium` binding, `extract::words`

**Files:**
- Modify: `backend/Cargo.toml`
- Create: `backend/src/ingest/pdf/mod.rs` (stub for now — just `pub mod extract;`)
- Create: `backend/src/ingest/pdf/extract.rs`
- Modify: `backend/src/ingest/mod.rs` (add `pub mod pdf;`)
- Modify: `backend/src/config.rs` (add `pdfium_lib_path: Option<String>`)
- Modify: `backend/src/main.rs` (startup bind probe + log)
- Modify: `backend/Dockerfile`
- Modify: `.gitignore`
- Create: `backend/scripts/fetch-pdfium.sh`
- Test: inline `#[cfg(test)]` in `extract.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  ```rust
  // src/ingest/pdf/extract.rs
  #[derive(Debug, Clone, PartialEq)]
  pub struct Word {
      pub text: String,
      pub x0: f32, pub x1: f32,   // left, right  (points)
      pub y0: f32, pub y1: f32,   // bottom, top  (points; larger = higher)
      pub page: usize,            // 0-based
  }

  /// Extract every positioned text segment from every page.
  /// `Err` if pdfium is not available or the PDF cannot be loaded.
  pub fn words(bytes: &[u8]) -> anyhow::Result<Vec<Word>>;

  /// True once `bind()` has succeeded at least once this process.
  pub fn is_available() -> bool;

  /// Called once at startup. Binds to $PDFIUM_LIB_PATH (file or dir), then to
  /// the system library. Idempotent; returns the bind result message.
  pub fn init(lib_path: Option<&str>) -> Result<(), String>;
  ```

- [ ] **Step 1: Add the dependency and fetch script**

`backend/Cargo.toml`:
```toml
pdfium-render = { version = "0.8", default-features = false, features = ["sync", "thread_safe"] }
```
(Confirm the current 0.8.x minor against `cargo add pdfium-render --dry-run`; pin the exact version it resolves. The crate loads libpdfium dynamically at runtime — it does NOT bundle it.)

`backend/scripts/fetch-pdfium.sh`:
```sh
#!/bin/sh
# Downloads a pinned pdfium prebuilt into backend/.pdfium/
set -eu
VER="chromium/6996"                    # pin; bump deliberately
OS="${1:-linux}"; ARCH="${2:-x64}"     # linux/x64, linux/arm64, mac/arm64
DEST="$(dirname "$0")/../.pdfium"
mkdir -p "$DEST"
URL="https://github.com/bblanchon/pdfium-binaries/releases/download/${VER}/pdfium-${OS}-${ARCH}.tgz"
curl -fsSL "$URL" | tar -xz -C "$DEST" --strip-components=0
echo "libpdfium at $DEST/lib/"
```
Make it executable: `chmod +x backend/scripts/fetch-pdfium.sh`.

`.gitignore`: add `backend/.pdfium/`.

- [ ] **Step 2: Write the failing test**

`backend/src/ingest/pdf/extract.rs` `#[cfg(test)]`:
```rust
use super::*;

fn pdfium_ready() -> bool {
    init(std::env::var("PDFIUM_LIB_PATH").ok().as_deref()).is_ok()
}

#[test]
fn words_is_empty_error_without_pdfium_or_returns_segments_with_it() {
    // A 1-page PDF with the single word "HELLO" is built by the shared test
    // helper in tests/pdf/gen.rs (Task 8). Until then, use a minimal literal.
    let pdf = crate::ingest::pdf::extract::tests::hello_pdf();
    if !pdfium_ready() {
        assert!(words(&pdf).is_err(), "no pdfium -> Err");
        return;
    }
    let ws = words(&pdf).unwrap();
    assert!(ws.iter().any(|w| w.text.contains("HELLO")));
    let h = ws.iter().find(|w| w.text.contains("HELLO")).unwrap();
    assert!(h.x1 > h.x0 && h.y1 > h.y0 && h.page == 0);
}

#[cfg(test)]
pub(crate) fn hello_pdf() -> Vec<u8> {
    // Smallest valid PDF with visible text "HELLO" via a base-14 font.
    // (literal bytes — keep; replaced by printpdf helper in Task 8)
    include_bytes!("../../../tests/pdf/fixtures/hello.pdf").to_vec()
}
```
Create `backend/tests/pdf/fixtures/hello.pdf` now with a hand-written minimal PDF (a Helvetica `BT /F1 24 Tf 72 700 Td (HELLO) Tj ET` content stream, catalog, one page, xref). This file is a fixture, not PII — commit it.

- [ ] **Step 3: Run test to verify it fails**

Run: `cd backend && cargo test --lib pdf::extract`
Expected: compile error — `words` / `init` not defined.

- [ ] **Step 4: Implement `extract.rs`**

```rust
use std::sync::OnceLock;
use anyhow::{anyhow, Result};
use pdfium_render::prelude::*;

static BOUND: OnceLock<bool> = OnceLock::new();

pub fn init(lib_path: Option<&str>) -> Result<(), String> {
    if *BOUND.get_or_init(|| try_bind(lib_path).is_ok()) {
        Ok(())
    } else {
        Err("pdfium: could not bind to a libpdfium library".into())
    }
}

pub fn is_available() -> bool {
    *BOUND.get().unwrap_or(&false)
}

fn try_bind(lib_path: Option<&str>) -> Result<(), PdfiumError> {
    if let Some(p) = lib_path {
        let path = std::path::Path::new(p);
        let attempt = if path.is_dir() {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(path))
        } else {
            Pdfium::bind_to_library(p)
        };
        if attempt.is_ok() {
            return attempt.map(|_| ());
        }
    }
    Pdfium::bind_to_system_library().map(|_| ())
}

fn pdfium() -> Result<Pdfium> {
    // Re-bind per call: pdfium-render bindings are cheap handles; the shared
    // library is already loaded into the process after init().
    let bindings = Pdfium::bind_to_system_library()
        .or_else(|_| Pdfium::bind_to_library(
            std::env::var("PDFIUM_LIB_PATH").unwrap_or_default()))
        .map_err(|e| anyhow!("pdfium unavailable: {e}"))?;
    Ok(Pdfium::new(bindings))
}

pub fn words(bytes: &[u8]) -> Result<Vec<Word>> {
    let pdfium = pdfium()?;
    let doc = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| anyhow!("pdfium could not load PDF: {e}"))?;
    let mut out = Vec::new();
    for (pi, page) in doc.pages().iter().enumerate() {
        let text = match page.text() {
            Ok(t) => t,
            Err(_) => continue,
        };
        for segment in text.segments().iter() {
            let s = segment.text();
            if s.trim().is_empty() {
                continue;
            }
            let b = segment.bounds();
            out.push(Word {
                text: s,
                x0: b.left().value,
                x1: b.right().value,
                y0: b.bottom().value,
                y1: b.top().value,
                page: pi,
            });
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    pub x0: f32, pub x1: f32,
    pub y0: f32, pub y1: f32,
    pub page: usize,
}
```
NOTE for the implementer: `segments()`, `segment.text()`, `segment.bounds()`, and `PdfRect` accessor names (`.left()/.right()/.top()/.bottom()` returning `PdfPoints` with `.value`) are version-sensitive. Before writing, run `cargo doc -p pdfium-render --open` (or read docs.rs for the pinned version) and adjust the four accessor calls / the `.iter()` vs `for x in` form to match. Do not change the `Word` struct or the function signatures.

- [ ] **Step 5: Wire config + startup**

`backend/src/config.rs`: add `pub pdfium_lib_path: Option<String>` to the config struct, read from `std::env::var("PDFIUM_LIB_PATH").ok()`.

`backend/src/main.rs`, after config load, before serving:
```rust
match crate::ingest::pdf::extract::init(config.pdfium_lib_path.as_deref()) {
    Ok(()) => tracing::info!("pdfium: loaded"),
    Err(e) => tracing::warn!("{e} — PDF parsing will use the fallback parser"),
}
```

- [ ] **Step 6: Dockerfile**

In `backend/Dockerfile` build stage, before `cargo build`:
```dockerfile
ARG PDFIUM_VER=chromium/6996
RUN mkdir -p /pdfium && \
    wget -qO- "https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_VER}/pdfium-linux-${TARGETARCH:-x64}.tgz" \
    | tar -xz -C /pdfium
```
In the runtime stage:
```dockerfile
COPY --from=build /pdfium/lib/libpdfium.so /usr/local/lib/libpdfium.so
ENV PDFIUM_LIB_PATH=/usr/local/lib/libpdfium.so
RUN ldconfig 2>/dev/null || true
```
(Alpine: ensure `libstdc++` and `libgcc` are in the runtime image — `apk add --no-cache libstdc++`.)

- [ ] **Step 7: Run tests**

Run:
```bash
cd backend && ./scripts/fetch-pdfium.sh linux "$(uname -m | sed 's/x86_64/x64/;s/aarch64/arm64/')"
PDFIUM_LIB_PATH="$PWD/.pdfium/lib" cargo test --lib pdf::extract -- --nocapture
```
Expected: PASS with pdfium present (segments found for "HELLO"). Also run without the env var: `cargo test --lib pdf::extract` → PASS (asserts `words()` errors).

- [ ] **Step 8: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/ingest/pdf/ backend/src/ingest/mod.rs \
        backend/src/config.rs backend/src/main.rs backend/Dockerfile .gitignore \
        backend/scripts/fetch-pdfium.sh backend/tests/pdf/fixtures/hello.pdf
git commit -m "feat(ingest): pdfium-render word extraction with graceful fallback"
```

---

### Task 3: `pdf/table.rs` — row clustering, header/band detection, cell assignment

**Files:**
- Create: `backend/src/ingest/pdf/table.rs`
- Modify: `backend/src/ingest/pdf/mod.rs` (`pub mod table;`)
- Test: inline `#[cfg(test)]` in `table.rs`

**Interfaces:**
- Consumes: `super::extract::Word`.
- Produces:
  ```rust
  use crate::ingest::profiles::{BankProfile, ColKind};
  use super::extract::Word;
  use std::collections::BTreeMap;

  #[derive(Debug, Clone, Copy, PartialEq)]
  pub struct Band { pub kind: ColKind, pub x0: f32, pub x1: f32 }

  /// Group words into visual rows: same page, y-centres within `y_tol` points.
  /// Rows are returned top-to-bottom; words within a row left-to-right.
  pub fn rows(words: Vec<Word>) -> Vec<Vec<Word>>;

  /// Scan the first `max_scan` rows of each page for the row matching the most
  /// `profile.pdf_columns` (>= 3 required). Returns X-bands spanning the page,
  /// boundaries at midpoints between adjacent matched header-word centres.
  pub fn columns(page_rows: &[Vec<Word>], profile: &BankProfile) -> Option<Vec<Band>>;

  /// Assign each word in a row to the band whose [x0,x1) contains its x-centre.
  /// Multiple words in the same band are space-joined in x order.
  pub fn cells(row: &[Word], bands: &[Band]) -> BTreeMap<ColKind, String>;
  ```

- [ ] **Step 1: Write the failing tests**

```rust
use super::*;
use crate::ingest::pdf::extract::Word;
use crate::ingest::profiles::generic;

fn w(text: &str, x0: f32, x1: f32, y: f32) -> Word {
    Word { text: text.into(), x0, x1, y0: y, y1: y + 8.0, page: 0 }
}

#[test]
fn rows_groups_by_y_and_sorts() {
    let ws = vec![
        w("b", 50.0, 60.0, 700.0),
        w("a", 10.0, 20.0, 701.0),   // same row as "b" (within tol)
        w("c", 10.0, 20.0, 680.0),   // lower row
    ];
    let r = rows(ws);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].iter().map(|x| x.text.as_str()).collect::<Vec<_>>(), ["a", "b"]);
    assert_eq!(r[1][0].text, "c");
}

#[test]
fn columns_finds_header_and_builds_bands() {
    let header = vec![
        w("Date", 10.0, 40.0, 750.0),
        w("Narration", 60.0, 130.0, 750.0),
        w("Withdrawal", 200.0, 270.0, 750.0),
        w("Deposit", 300.0, 350.0, 750.0),
        w("Balance", 400.0, 450.0, 750.0),
    ];
    let bands = columns(&[header], &generic::profile()).expect("header matched");
    let kinds: Vec<ColKind> = bands.iter().map(|b| b.kind).collect();
    assert_eq!(kinds, vec![ColKind::TxnDate, ColKind::Description, ColKind::Debit, ColKind::Credit, ColKind::Balance]);
    // first band opens at -inf, last closes at +inf
    assert!(bands[0].x0 < 10.0 && bands[4].x1 > 450.0);
    // boundary between Date(centre 25) and Narration(centre 95) is 60
    assert!((bands[0].x1 - 60.0).abs() < 1.0);
}

#[test]
fn columns_returns_none_when_too_few_headers_match() {
    let junk = vec![w("Hello", 10.0, 40.0, 750.0), w("World", 60.0, 90.0, 750.0)];
    assert!(columns(&[junk], &generic::profile()).is_none());
}

#[test]
fn cells_buckets_words_by_x_centre() {
    let bands = vec![
        Band { kind: ColKind::TxnDate, x0: f32::MIN, x1: 60.0 },
        Band { kind: ColKind::Description, x0: 60.0, x1: 190.0 },
        Band { kind: ColKind::Balance, x0: 190.0, x1: f32::MAX },
    ];
    let row = vec![
        w("01/02/2024", 10.0, 50.0, 700.0),
        w("UPI", 70.0, 90.0, 700.0),
        w("swiggy", 95.0, 130.0, 700.0),
        w("1,234.00", 200.0, 240.0, 700.0),
    ];
    let c = cells(&row, &bands);
    assert_eq!(c[&ColKind::TxnDate], "01/02/2024");
    assert_eq!(c[&ColKind::Description], "UPI swiggy");
    assert_eq!(c[&ColKind::Balance], "1,234.00");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cd backend && cargo test --lib pdf::table`
Expected: compile error — module/functions missing.

- [ ] **Step 3: Implement `table.rs`**

```rust
use std::collections::BTreeMap;
use crate::ingest::profiles::{BankProfile, ColKind};
use super::extract::Word;

const Y_TOL: f32 = 3.0;      // points; words whose y-centres are closer share a row
const MAX_SCAN: usize = 15;  // header must be in the first N rows of a page

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band { pub kind: ColKind, pub x0: f32, pub x1: f32 }

fn yc(w: &Word) -> f32 { (w.y0 + w.y1) / 2.0 }
fn xc(w: &Word) -> f32 { (w.x0 + w.x1) / 2.0 }

pub fn rows(mut words: Vec<Word>) -> Vec<Vec<Word>> {
    // sort by page, then top-to-bottom (descending y), then left-to-right
    words.sort_by(|a, b| {
        a.page.cmp(&b.page)
            .then(yc(b).partial_cmp(&yc(a)).unwrap())
            .then(a.x0.partial_cmp(&b.x0).unwrap())
    });
    let mut out: Vec<Vec<Word>> = Vec::new();
    for wd in words {
        match out.last_mut() {
            Some(row)
                if row[0].page == wd.page && (yc(&row[0]) - yc(&wd)).abs() <= Y_TOL =>
            {
                row.push(wd);
            }
            _ => out.push(vec![wd]),
        }
    }
    for row in &mut out {
        row.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap());
    }
    out
}

fn match_kind(text: &str, profile: &BankProfile) -> Option<ColKind> {
    let t = text.to_lowercase();
    // longest-substring-wins so "closing balance" beats "balance" ambiguity is
    // avoided; iterate columns in declared order, first hit wins
    profile.pdf_columns.iter()
        .find(|c| c.headers.iter().any(|h| t.contains(h)))
        .map(|c| c.kind)
}

pub fn columns(page_rows: &[Vec<Word>], profile: &BankProfile) -> Option<Vec<Band>> {
    let mut best: Option<(usize, Vec<(ColKind, f32)>)> = None; // (match count, [(kind, centre)])
    for row in page_rows.iter().take(MAX_SCAN) {
        let mut hits: Vec<(ColKind, f32)> = Vec::new();
        for wd in row {
            if let Some(k) = match_kind(&wd.text, profile) {
                if !hits.iter().any(|(hk, _)| *hk == k) {
                    hits.push((k, xc(wd)));
                }
            }
        }
        if hits.len() >= 3 && best.as_ref().map_or(true, |(n, _)| hits.len() > *n) {
            best = Some((hits.len(), hits));
        }
    }
    let (_, mut hits) = best?;
    hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let mut bands = Vec::with_capacity(hits.len());
    for i in 0..hits.len() {
        let x0 = if i == 0 { f32::MIN } else { (hits[i - 1].1 + hits[i].1) / 2.0 };
        let x1 = if i == hits.len() - 1 { f32::MAX } else { (hits[i].1 + hits[i + 1].1) / 2.0 };
        bands.push(Band { kind: hits[i].0, x0, x1 });
    }
    Some(bands)
}

pub fn cells(row: &[Word], bands: &[Band]) -> BTreeMap<ColKind, String> {
    let mut map: BTreeMap<ColKind, Vec<(f32, &str)>> = BTreeMap::new();
    for wd in row {
        let c = xc(wd);
        if let Some(band) = bands.iter().find(|b| c >= b.x0 && c < b.x1) {
            map.entry(band.kind).or_default().push((wd.x0, wd.text.as_str()));
        }
    }
    map.into_iter()
        .map(|(k, mut v)| {
            v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            (k, v.into_iter().map(|(_, s)| s).collect::<Vec<_>>().join(" "))
        })
        .collect()
}
```

- [ ] **Step 4: Run tests to verify pass**

Run: `cd backend && cargo test --lib pdf::table`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add backend/src/ingest/pdf/table.rs backend/src/ingest/pdf/mod.rs
git commit -m "feat(ingest): PDF row clustering and column-band detection"
```

---

### Task 4: `pdf/rows.rs` — amount parsing and row assembly

**Files:**
- Create: `backend/src/ingest/pdf/rows.rs`
- Modify: `backend/src/ingest/pdf/mod.rs` (`pub mod rows;`)
- Test: inline `#[cfg(test)]` in `rows.rs`

**Interfaces:**
- Consumes: `super::table::{Band, cells}` conceptually (works on `BTreeMap<ColKind,String>`), `crate::ingest::models::RawRow`, `crate::ingest::profiles::{BankProfile, StatementKind, ColKind}`.
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub enum Sign { Cr, Dr }

  #[derive(Debug, Clone, Copy, PartialEq)]
  pub struct Signed { pub value: f64, pub sign: Option<Sign> }

  /// Parse one money cell. Handles: "1,23,456.78", "₹1,234", "Rs. 1,234",
  /// "1,234.00 Cr", "1,234.00 Dr", "(1,234.00)" -> Dr, "1,234.00-" -> Dr,
  /// "-1,234.00" -> Dr. Returns None if there is no numeric content.
  pub fn amount(cell: &str) -> Option<Signed>;

  /// Build RawRows from ordered per-row cell maps. A row is a transaction start
  /// when its TxnDate cell parses against `profile.date_formats`; other rows are
  /// continuation lines merged into the previous row. Preamble rows (before the
  /// first start) and a trailing totals row are dropped.
  pub fn assemble(
      rows: Vec<std::collections::BTreeMap<ColKind, String>>,
      profile: &BankProfile,
  ) -> Vec<RawRow>;
  ```

- [ ] **Step 1: Write the failing tests**

```rust
use super::*;
use crate::ingest::profiles::{generic, generic_cc, ColKind};
use std::collections::BTreeMap;

#[test]
fn amount_parses_indian_grouping_and_symbols() {
    assert_eq!(amount("1,23,456.78").unwrap().value, 123456.78);
    assert_eq!(amount("₹1,234").unwrap().value, 1234.0);
    assert_eq!(amount("Rs. 1,234.00").unwrap().value, 1234.0);
    assert_eq!(amount("").is_none(), true);
    assert_eq!(amount("NEFT-000123").is_none(), true);
}

#[test]
fn amount_reads_sign_markers() {
    assert_eq!(amount("1,234.00 Cr").unwrap().sign, Some(Sign::Cr));
    assert_eq!(amount("1,234.00 Dr").unwrap().sign, Some(Sign::Dr));
    assert_eq!(amount("(1,234.00)").unwrap().sign, Some(Sign::Dr));
    assert_eq!(amount("1,234.00-").unwrap().sign, Some(Sign::Dr));
    assert_eq!(amount("-1,234.00").unwrap().sign, Some(Sign::Dr));
    assert_eq!(amount("1,234.00").unwrap().sign, None);
}

fn cell(pairs: &[(ColKind, &str)]) -> BTreeMap<ColKind, String> {
    pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
}

#[test]
fn assemble_account_rows_with_continuation_and_totals() {
    let rows = vec![
        cell(&[(ColKind::TxnDate, "col headers ignored")]), // preamble (no date)
        cell(&[(ColKind::TxnDate, "01/02/2024"), (ColKind::Description, "UPI swiggy"),
               (ColKind::Debit, "450.00"), (ColKind::Balance, "9,550.00")]),
        cell(&[(ColKind::Description, "ref 416823456789")]), // continuation
        cell(&[(ColKind::TxnDate, "02/02/2024"), (ColKind::Description, "SALARY"),
               (ColKind::Credit, "50,000.00"), (ColKind::Balance, "59,550.00")]),
        cell(&[(ColKind::Description, "Closing Balance"), (ColKind::Balance, "59,550.00")]), // totals
    ];
    let out = assemble(rows, &generic::profile());
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].txn_date, "01/02/2024");
    assert_eq!(out[0].description, "UPI swiggy ref 416823456789");
    assert_eq!(out[0].debit, Some(450.0));
    assert_eq!(out[0].credit, None);
    assert_eq!(out[1].credit, Some(50000.0));
    assert_eq!(out[1].debit, None);
}

#[test]
fn assemble_credit_card_uses_sign_marker() {
    let rows = vec![
        cell(&[(ColKind::TxnDate, "05/02/2024"), (ColKind::Description, "AMAZON"),
               (ColKind::Amount, "2,000.00 Dr")]),
        cell(&[(ColKind::TxnDate, "07/02/2024"), (ColKind::Description, "REFUND"),
               (ColKind::Amount, "500.00 Cr")]),
    ];
    let out = assemble(rows, &generic_cc::profile());
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].debit, Some(2000.0));
    assert_eq!(out[0].credit, None);
    assert_eq!(out[1].credit, Some(500.0));
    assert_eq!(out[1].debit, None);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cd backend && cargo test --lib pdf::rows`
Expected: compile error.

- [ ] **Step 3: Implement `rows.rs`**

```rust
use std::collections::BTreeMap;
use chrono::NaiveDate;
use crate::ingest::models::RawRow;
use crate::ingest::profiles::{BankProfile, ColKind, StatementKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sign { Cr, Dr }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Signed { pub value: f64, pub sign: Option<Sign> }

pub fn amount(cell: &str) -> Option<Signed> {
    let raw = cell.trim();
    if raw.is_empty() { return None; }
    let lower = raw.to_lowercase();
    let mut sign = None;
    if lower.contains(" cr") || lower.ends_with("cr") { sign = Some(Sign::Cr); }
    if lower.contains(" dr") || lower.ends_with("dr") { sign = Some(Sign::Dr); }
    let paren = raw.starts_with('(') && raw.trim_end_matches(')').ends_with(|c: char| c.is_ascii_digit());
    let trailing_minus = raw.trim_end_matches(')').trim().ends_with('-');
    let leading_minus = raw.trim_start_matches(['(', '₹', 'R', 's', '.', ' ']).trim_start().starts_with('-');
    // keep only digits, comma, dot; then drop commas
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
    let cleaned = digits.replace(',', "");
    let cleaned = cleaned.trim_matches('.');
    if cleaned.is_empty() || cleaned.parse::<f64>().is_err() { return None; }
    let value: f64 = cleaned.parse().unwrap();
    if sign.is_none() && (paren || trailing_minus || leading_minus) {
        sign = Some(Sign::Dr);
    }
    Some(Signed { value, sign })
}

fn parse_date(s: &str, profile: &BankProfile) -> Option<NaiveDate> {
    let s = s.trim();
    profile.date_formats.iter().find_map(|f| NaiveDate::parse_from_str(s, f).ok())
}

fn is_totals(cells: &BTreeMap<ColKind, String>) -> bool {
    cells.get(&ColKind::Description)
        .map(|d| {
            let d = d.to_lowercase();
            d.contains("closing balance") || d.contains("grand total")
                || d.starts_with("total") || d.contains("opening balance")
        })
        .unwrap_or(false)
}

pub fn assemble(
    rows: Vec<BTreeMap<ColKind, String>>,
    profile: &BankProfile,
) -> Vec<RawRow> {
    let mut out: Vec<RawRow> = Vec::new();
    let mut started = false;
    for cells in rows {
        let date = cells.get(&ColKind::TxnDate).and_then(|s| parse_date(s, profile));
        if let Some(_d) = date {
            if is_totals(&cells) { continue; }
            started = true;
            out.push(build_row(&cells, profile));
        } else if started {
            if is_totals(&cells) { continue; }
            merge_continuation(out.last_mut().unwrap(), &cells);
        }
        // else: preamble, ignore
    }
    out
}

fn get<'a>(c: &'a BTreeMap<ColKind, String>, k: ColKind) -> &'a str {
    c.get(&k).map(|s| s.as_str()).unwrap_or("")
}

fn build_row(c: &BTreeMap<ColKind, String>, profile: &BankProfile) -> RawRow {
    let txn_date = get(c, ColKind::TxnDate).to_string();
    let value_date = {
        let v = get(c, ColKind::ValueDate);
        if v.is_empty() { txn_date.clone() } else { v.to_string() }
    };
    let (mut debit, mut credit) = (None, None);
    match profile.statement_kind {
        StatementKind::CreditCard => {
            if let Some(a) = amount(get(c, ColKind::Amount)) {
                match a.sign {
                    Some(Sign::Cr) => credit = Some(a.value),
                    _ => debit = Some(a.value), // Dr or unmarked -> spend
                }
            }
        }
        StatementKind::Account => {
            debit = amount(get(c, ColKind::Debit)).map(|a| a.value);
            credit = amount(get(c, ColKind::Credit)).map(|a| a.value);
            if debit.is_none() && credit.is_none() {
                if let Some(a) = amount(get(c, ColKind::Amount)) {
                    match a.sign {
                        Some(Sign::Cr) => credit = Some(a.value),
                        Some(Sign::Dr) => debit = Some(a.value),
                        None => debit = Some(a.value),
                    }
                }
            }
        }
    }
    let bank_ref = {
        let r = get(c, ColKind::Ref);
        if r.is_empty() { None } else { Some(r.to_string()) }
    };
    RawRow {
        txn_date,
        value_date,
        description: get(c, ColKind::Description).to_string(),
        debit,
        credit,
        balance: amount(get(c, ColKind::Balance)).map(|a| a.value),
        bank_ref,
    }
}

fn merge_continuation(row: &mut RawRow, c: &BTreeMap<ColKind, String>) {
    let extra = get(c, ColKind::Description);
    if !extra.is_empty() {
        if !row.description.is_empty() { row.description.push(' '); }
        row.description.push_str(extra);
    }
    if row.debit.is_none() { row.debit = amount(get(c, ColKind::Debit)).map(|a| a.value); }
    if row.credit.is_none() { row.credit = amount(get(c, ColKind::Credit)).map(|a| a.value); }
    if row.balance.is_none() { row.balance = amount(get(c, ColKind::Balance)).map(|a| a.value); }
}
```

- [ ] **Step 4: Run tests**

Run: `cd backend && cargo test --lib pdf::rows`
Expected: PASS (4 tests). Fix `amount()` edge cases if a case fails — the tests are the spec.

- [ ] **Step 5: Commit**

```bash
git add backend/src/ingest/pdf/rows.rs backend/src/ingest/pdf/mod.rs
git commit -m "feat(ingest): PDF amount parsing and row assembly"
```

---

### Task 5: `pdf/reconcile.rs` — confidence scoring

**Files:**
- Create: `backend/src/ingest/pdf/reconcile.rs`
- Modify: `backend/src/ingest/pdf/mod.rs` (`pub mod reconcile;`)
- Test: inline `#[cfg(test)]` in `reconcile.rs`

**Interfaces:**
- Consumes: `crate::ingest::models::RawRow`, `crate::ingest::profiles::StatementKind`.
- Produces:
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub enum Confidence { Ok, Low(String) }

  /// `full_text` is the lowercased page text, used only to look for a
  /// "total amount due" figure on credit-card statements.
  pub fn confidence(rows: &[RawRow], kind: StatementKind, full_text: &str) -> Confidence;
  ```

- [ ] **Step 1: Write the failing tests**

```rust
use super::*;
use crate::ingest::models::RawRow;
use crate::ingest::profiles::StatementKind;

fn r(debit: Option<f64>, credit: Option<f64>, balance: Option<f64>) -> RawRow {
    RawRow { txn_date: "x".into(), value_date: "x".into(), description: "x".into(),
             debit, credit, balance, bank_ref: None }
}

#[test]
fn account_ok_when_balances_reconcile() {
    let rows = vec![
        r(Some(450.0), None, Some(9_550.0)),
        r(None, Some(50_000.0), Some(59_550.0)),
        r(Some(1_000.0), None, Some(58_550.0)),
    ];
    assert_eq!(confidence(&rows, StatementKind::Account, ""), Confidence::Ok);
}

#[test]
fn account_low_when_a_balance_is_wrong() {
    let rows = vec![
        r(Some(450.0), None, Some(9_550.0)),
        r(None, Some(50_000.0), Some(999.0)),   // wrong
        r(Some(1_000.0), None, Some(-1.0)),     // wrong
    ];
    match confidence(&rows, StatementKind::Account, "") {
        Confidence::Low(m) => assert!(m.contains("reconcile")),
        _ => panic!("expected Low"),
    }
}

#[test]
fn account_low_when_too_few_rows() {
    assert!(matches!(confidence(&[r(Some(1.0), None, Some(1.0))], StatementKind::Account, ""), Confidence::Low(_)));
}

#[test]
fn cc_low_when_a_row_has_no_amount() {
    let rows = vec![r(Some(100.0), None, None), r(None, None, None)];
    assert!(matches!(confidence(&rows, StatementKind::CreditCard, ""), Confidence::Low(_)));
}

#[test]
fn cc_ok_when_sum_matches_total_due() {
    let rows = vec![r(Some(2_000.0), None, None), r(None, Some(500.0), None)];
    // net spend 1500
    assert_eq!(confidence(&rows, StatementKind::CreditCard, "total amount due 1,500.00"), Confidence::Ok);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cd backend && cargo test --lib pdf::reconcile`
Expected: compile error.

- [ ] **Step 3: Implement `reconcile.rs`**

```rust
use crate::ingest::models::RawRow;
use crate::ingest::profiles::StatementKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Confidence { Ok, Low(String) }

const ACCT_TOL: f64 = 0.01;
const CC_TOL: f64 = 1.0;

pub fn confidence(rows: &[RawRow], kind: StatementKind, full_text: &str) -> Confidence {
    if rows.len() < 2 {
        return Confidence::Low(format!("only {} transaction(s) found", rows.len()));
    }
    match kind {
        StatementKind::Account => account(rows),
        StatementKind::CreditCard => credit_card(rows, full_text),
    }
}

fn account(rows: &[RawRow]) -> Confidence {
    let mut checked = 0usize;
    let mut bad = 0usize;
    let mut prev: Option<f64> = None;
    for row in rows {
        if let (Some(p), Some(b)) = (prev, row.balance) {
            let delta = row.credit.unwrap_or(0.0) - row.debit.unwrap_or(0.0);
            checked += 1;
            if (p + delta - b).abs() > ACCT_TOL { bad += 1; }
        }
        if row.balance.is_some() { prev = row.balance; }
    }
    if checked >= 2 && bad * 10 > checked {
        Confidence::Low(format!("balances do not reconcile ({bad}/{checked} rows)"))
    } else {
        Confidence::Ok
    }
}

fn credit_card(rows: &[RawRow], full_text: &str) -> Confidence {
    if rows.iter().any(|r| r.debit.is_none() && r.credit.is_none()) {
        return Confidence::Low("some transactions have no parseable amount".into());
    }
    if let Some(total) = find_total_due(full_text) {
        let net: f64 = rows.iter().map(|r| r.debit.unwrap_or(0.0) - r.credit.unwrap_or(0.0)).sum();
        if (net - total).abs() > CC_TOL {
            return Confidence::Low(format!(
                "transaction total {net:.2} does not match statement total {total:.2}"
            ));
        }
    }
    Confidence::Ok
}

fn find_total_due(text: &str) -> Option<f64> {
    let t = text.to_lowercase();
    for anchor in ["total amount due", "total dues", "total payment due"] {
        if let Some(i) = t.find(anchor) {
            let tail = &t[i + anchor.len()..];
            let num: String = tail.chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
                .collect();
            if let Ok(v) = num.replace(',', "").trim_matches('.').parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}
```

- [ ] **Step 4: Run tests**

Run: `cd backend && cargo test --lib pdf::reconcile`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add backend/src/ingest/pdf/reconcile.rs backend/src/ingest/pdf/mod.rs
git commit -m "feat(ingest): PDF parse confidence / balance reconciliation"
```

---

### Task 6: `pdf/legacy.rs` + `pdf/mod.rs` orchestration + fallback + CC detection

**Files:**
- Create: `backend/src/ingest/pdf/legacy.rs` (current `parse_pdf` body, verbatim)
- Modify: `backend/src/ingest/pdf/mod.rs` (orchestration)
- Modify: `backend/src/ingest/parse.rs` (delete `parse_pdf` body; delegate; keep `pdf_extract` import only in `legacy.rs`)
- Modify: `backend/src/ingest/handlers.rs` (`detect_bank` CC heuristic — see below; if `detect_bank` lives in `parse.rs` or `mod.rs`, modify there)
- Test: inline `#[cfg(test)]` in `pdf/mod.rs`

**Interfaces:**
- Consumes: `extract::words`, `table::{rows, columns, cells}`, `rows::assemble`, `reconcile::confidence`.
- Produces:
  ```rust
  // src/ingest/pdf/mod.rs
  pub mod extract; pub mod table; pub mod rows; pub mod reconcile; mod legacy;

  /// Returns (rows, headers, lowercased_full_text, low_confidence_reason).
  pub fn parse_pdf(
      bytes: &[u8],
      profile: &crate::ingest::profiles::BankProfile,
  ) -> anyhow::Result<(Vec<crate::ingest::models::RawRow>, Vec<String>, String, Option<String>)>;

  /// Text-only extraction for bank detection when the caller has no profile yet.
  /// Uses pdfium if available, else pdf_extract. Never errors on a readable PDF.
  pub fn extract_text(bytes: &[u8]) -> String;
  ```
- `parse.rs::parse_pdf(bytes, profile) -> Result<(Vec<RawRow>, Vec<String>, String)>` stays as a thin wrapper that drops the 4th element **only** where the 4-tuple isn't threaded yet; but Task 7 threads it, so prefer changing `parse.rs::parse_pdf` to return the 4-tuple and updating `parse_file`. See Task 7 for the caller updates. For THIS task, `parse_file`'s PDF arm returns `(rows, headers, text)` and stores the reason in a `pub(crate) static LAST_PDF_CONFIDENCE` is **NOT** allowed — instead, land Task 6 with `parse_file` PDF arm calling `pdf::parse_pdf` and simply discarding the 4th value, then Task 7 changes the signature. Keep Task 6 green.

- [ ] **Step 1: Move the legacy parser**

Cut the entire current body of `fn parse_pdf` from `backend/src/ingest/parse.rs` into `backend/src/ingest/pdf/legacy.rs` as:
```rust
use crate::ingest::{models::RawRow, profiles::BankProfile};

pub fn parse_pdf(bytes: &[u8], _profile: &BankProfile)
    -> anyhow::Result<(Vec<RawRow>, Vec<String>, String)>
{
    let text = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to extract text from PDF: {:?}", e))?;
    // ... rest of the existing implementation unchanged ...
}

pub fn extract_text(bytes: &[u8]) -> String {
    pdf_extract::extract_text_from_mem(bytes).unwrap_or_default()
}
```
Remove the now-unused `pdf_extract` import from `parse.rs`.

- [ ] **Step 2: Write the failing tests**

`backend/src/ingest/pdf/mod.rs` `#[cfg(test)]`:
```rust
use crate::ingest::profiles::generic;

#[test]
fn parse_pdf_falls_back_and_flags_when_pdfium_absent_or_no_header() {
    // Garbage bytes: not a loadable PDF. Must not panic; must return the
    // legacy result (Err from legacy on non-PDF) OR Ok with a Low reason.
    let res = parse_pdf(b"not a pdf at all", &generic::profile());
    assert!(res.is_err() || res.as_ref().unwrap().3.is_some());
}

#[test]
fn extract_text_never_panics_on_junk() {
    assert_eq!(extract_text(b"junk"), String::new());
}
```
Plus a gated end-to-end test that only runs with pdfium + a fixture (added properly in Task 8):
```rust
#[test]
fn parse_pdf_reads_a_generated_account_statement() {
    let Some(pdf) = crate::ingest::pdf::test_fixture("account_hdfc_basic") else { return; };
    let (rows, _h, _t, reason) = parse_pdf(&pdf, &crate::ingest::profiles::hdfc::profile()).unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(reason, None);
    assert_eq!(rows[0].debit, Some(450.0));
}
```
(`test_fixture` is a helper returning `Option<Vec<u8>>` — `None` when pdfium/printpdf unavailable; implemented in Task 8. Until then this test early-returns.)

- [ ] **Step 3: Run to verify failure**

Run: `cd backend && cargo test --lib pdf::`
Expected: compile error — `parse_pdf` in `pdf/mod.rs` not defined.

- [ ] **Step 4: Implement `pdf/mod.rs`**

```rust
pub mod extract;
pub mod table;
pub mod rows;
pub mod reconcile;
mod legacy;

use anyhow::Result;
use crate::ingest::models::RawRow;
use crate::ingest::profiles::BankProfile;

const HEADERS: [&str; 5] = ["Txn Date", "Description", "Debit", "Credit", "Balance"];

pub fn extract_text(bytes: &[u8]) -> String {
    if extract::is_available() {
        if let Ok(words) = extract::words(bytes) {
            if !words.is_empty() {
                return words.iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ");
            }
        }
    }
    legacy::extract_text(bytes)
}

pub fn parse_pdf(
    bytes: &[u8],
    profile: &BankProfile,
) -> Result<(Vec<RawRow>, Vec<String>, String, Option<String>)> {
    let headers = HEADERS.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    // Try the coordinate-aware path.
    if extract::is_available() {
        if let Ok(words) = extract::words(bytes) {
            let full_text = words.iter().map(|w| w.text.as_str())
                .collect::<Vec<_>>().join(" ").to_lowercase();
            let all_rows = table::rows(words);
            // group rows by page for header detection
            let mut by_page: Vec<Vec<Vec<crate::ingest::pdf::extract::Word>>> = Vec::new();
            for row in &all_rows {
                let p = row[0].page;
                if by_page.len() <= p { by_page.resize(p + 1, Vec::new()); }
                by_page[p].push(row.clone());
            }
            let mut bands = None;
            for page in &by_page {
                if let Some(b) = table::columns(page, profile) { bands = Some(b); break; }
            }
            if let Some(bands) = bands {
                let cell_rows = all_rows.iter()
                    .map(|r| table::cells(r, &bands))
                    .collect::<Vec<_>>();
                let parsed = rows::assemble(cell_rows, profile);
                if parsed.len() >= 2 {
                    let reason = match reconcile::confidence(&parsed, profile.statement_kind, &full_text) {
                        reconcile::Confidence::Ok => None,
                        reconcile::Confidence::Low(m) => Some(m),
                    };
                    return Ok((parsed, headers, full_text, reason));
                }
            }
        }
    }

    // Fallback: legacy line heuristic.
    let (rows_l, headers_l, text_l) = legacy::parse_pdf(bytes, profile)?;
    let reason = Some(
        if extract::is_available() {
            "layout not recognised; used the fallback parser — please verify amounts".to_string()
        } else {
            "high-accuracy PDF parser unavailable; used the fallback parser — please verify amounts".to_string()
        },
    );
    Ok((rows_l, headers_l, text_l.to_lowercase(), reason))
}

#[cfg(test)]
pub(crate) fn test_fixture(_name: &str) -> Option<Vec<u8>> { None } // replaced in Task 8
```

- [ ] **Step 5: Update `parse.rs` delegate**

In `backend/src/ingest/parse.rs`, replace `fn parse_pdf` with:
```rust
fn parse_pdf(bytes: &[u8], profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>, String)> {
    let (rows, headers, text, _reason) = crate::ingest::pdf::parse_pdf(bytes, profile)?;
    Ok((rows, headers, text))
}
```
(Task 7 removes this shim and threads `_reason` through.)

- [ ] **Step 6: CC detection in `detect_bank`**

Find `detect_bank` (grep: `cd backend && grep -rn "fn detect_bank" src/`). It currently picks a profile from `file_hint`. Add, before the generic fallback:
```rust
// Credit-card statements: no running-balance column, CC wording present.
let looks_cc = hint.contains("credit card")
    || hint.contains("statement of account")
    || hint.contains("minimum amount due")
    || hint.contains("total amount due");
let has_balance_word = hint.contains("closing balance") || hint.contains("running balance")
    || hint.contains("available balance");
if looks_cc && !has_balance_word {
    if let Some(p) = profiles.iter().find(|p| p.name == "GENERIC_CC") {
        return p;
    }
}
```

- [ ] **Step 7: Run the whole suite**

Run: `cd backend && cargo test 2>&1 | tail -20`
Expected: all green, including the pre-existing `parse.rs` tests (they now exercise `legacy.rs` indirectly — if any referenced the private `parse_pdf` directly, update the path to `crate::ingest::pdf::legacy::parse_pdf`).

- [ ] **Step 8: Commit**

```bash
git add backend/src/ingest/
git commit -m "feat(ingest): coordinate-aware parse_pdf with legacy fallback + CC detection"
```

---

### Task 7: Thread the low-confidence reason to clients

**Files:**
- Modify: `backend/src/ingest/parse.rs` (`parse_file` returns 4-tuple), `backend/src/ingest/models.rs` (`UploadResponse.warnings`), `backend/src/ingest/handlers.rs`, `backend/src/ingest/email/run.rs`
- Modify: `frontend/src/pages/UploadPage.tsx`
- Modify: `android/app/src/main/java/com/khata/app/api/Models.kt` (+ wherever `UploadResponse` is consumed)
- Test: inline in `parse.rs`; a handler test if one exists

**Interfaces:**
- Produces:
  ```rust
  pub fn parse_file(bytes, kind, profile)
      -> Result<(Vec<RawRow>, Vec<String>, String, Option<String>)>;  // 4th = low-confidence reason
  ```
  ```rust
  pub struct UploadResponse {
      pub bank_detected: String,
      pub rows_parsed: usize,
      pub normalized: usize,
      pub inserted: usize,
      pub skipped_duplicates: usize,
      pub warnings: Vec<String>,   // NEW — empty when high-confidence
  }
  ```

- [ ] **Step 1: Write the failing test**

`backend/src/ingest/parse.rs` `#[cfg(test)]`:
```rust
#[test]
fn parse_file_csv_reports_no_warning() {
    let csv = b"Date,Narration,Debit,Credit,Balance\n01/02/2024,Test,100.00,,900.00\n02/02/2024,Two,,50.00,950.00\n";
    let (_rows, _h, _t, reason) = parse_file(csv, FileKind::Csv, &crate::ingest::profiles::generic::profile()).unwrap();
    assert_eq!(reason, None);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cd backend && cargo test --lib parse_file_csv_reports_no_warning`
Expected: compile error — `parse_file` returns a 3-tuple.

- [ ] **Step 3: Change signatures**

- `parse_file`: return `Ok((rows, headers, text, None))` from the CSV and Excel arms; the PDF arm returns `crate::ingest::pdf::parse_pdf(bytes, profile)?` directly (already a 4-tuple).
- Delete the `parse.rs::parse_pdf` shim added in Task 6; call `crate::ingest::pdf::parse_pdf` inline in the PDF arm.
- `backend/src/ingest/models.rs`: add `pub warnings: Vec<String>` to `UploadResponse`.

- [ ] **Step 4: Update `handlers.rs`**

Where `parse_file` is called (currently two calls — the bank-hint pass and the real pass), destructure the 4-tuple. Use the reason from the **second** (real-profile) call:
```rust
let (raw_rows, _headers, _file_hint2, low_conf) =
    parse_file(bytes.as_ref(), kind.clone(), profile)?;
```
Also switch the first (hint) pass to `crate::ingest::pdf::extract_text` for PDFs rather than a full parse if it currently full-parses — minimal change: keep as-is but destructure `(.., _)`.
Populate both `Json(UploadResponse { .. })` returns:
```rust
warnings: low_conf.into_iter().collect(),
```
(The early-return `UploadResponse` for the "0 rows" case gets `warnings: vec![]`.)

- [ ] **Step 5: Update `email/run.rs`**

In `process_attachment`, the two `safe_parse` calls wrap `parse_file`. Find `fn safe_parse` (grep) and update it to return/propagate the 4th element; in `process_attachment` after the real parse:
```rust
let (raw_rows, _, _, low_conf) = safe_parse(&bytes, kind, profile)?;
// ... after successfully importing rows ...
if let Some(reason) = low_conf {
    counters.push_error("parse-confidence", filename, &reason);
}
```
`push_error` with kind `parse-confidence` is non-fatal and does not set `last_error` (verify `flush_counters` / the `needs_pw` logic in `run.rs:585` only reacts to `"password"` substrings — `parse-confidence` reasons must not contain the word "password"; they don't).

- [ ] **Step 6: Frontend**

`frontend/src/pages/UploadPage.tsx`: the upload response type gains `warnings: string[]`. After a successful upload, if `resp.warnings?.length`, render an amber callout:
```tsx
{resp.warnings?.length > 0 && (
  <div className="upload-warning">
    {resp.warnings.map((w, i) => <p key={i}>⚠ {w}</p>)}
  </div>
)}
```
Add a minimal `.upload-warning` style near the existing upload styles (amber bg, matches the existing token system).

- [ ] **Step 7: Android**

`Models.kt`: add `val warnings: List<String> = emptyList()` to the `UploadResponse` data class. In `CombinedUploadScreen.kt` (or the VM handling the result), after a successful upload show `warnings.joinToString("\n")` as a toast/snackbar if non-empty. Keep it small.

- [ ] **Step 8: Run everything**

Run:
```bash
cd backend && cargo test 2>&1 | tail -15
cd ../frontend && npm run build 2>&1 | tail -5
```
Expected: backend green; frontend builds. (Android build not required locally — CI covers it.)

- [ ] **Step 9: Commit**

```bash
git add backend/ frontend/ android/
git commit -m "feat(ingest): surface low-confidence PDF parse as client warnings"
```

---

### Task 8: Synthetic PDF fixtures with `printpdf` + integration tests

**Files:**
- Modify: `backend/Cargo.toml` (`[dev-dependencies] printpdf = "0.7"`)
- Create: `backend/tests/pdf/gen.rs` (fixture generator; compiled as part of an integration test crate)
- Create: `backend/tests/pdf_parser.rs` (integration tests)
- Modify: `backend/src/ingest/pdf/mod.rs` (replace the stub `test_fixture`)
- Create: `backend/tests/pdf/fixtures/` expected-JSON files

**Interfaces:**
- Consumes: `khata::ingest::pdf::parse_pdf`, `khata::ingest::profiles::*` (the crate must expose these — if the binary crate has no lib target, add a minimal `src/lib.rs` re-exporting `pub mod ingest; pub mod config;` etc., or move the integration tests to `#[cfg(test)]` modules under `src/`). Prefer: add `src/lib.rs` exposing the modules the tests need; `main.rs` becomes a thin `fn main`.
- Produces: `gen::statement(spec: &StatementSpec) -> Vec<u8>` and `StatementSpec` builder.

- [ ] **Step 1: Decide crate layout**

The binary crate currently has no lib target (`cargo test --lib` earlier said "no library targets"). Add `backend/src/lib.rs`:
```rust
pub mod config;
pub mod db;
pub mod ingest;
pub mod auth;
pub mod txns;
pub mod chat;
pub mod error;
// ...every module main.rs declares...
```
Change `backend/src/main.rs` to `use khata::*;` and keep only `#[tokio::main] async fn main()`. Run `cargo build && cargo test` to confirm nothing broke. Commit this refactor separately:
```bash
git add backend/src/lib.rs backend/src/main.rs
git commit -m "refactor(backend): add lib target so integration tests can import modules"
```

- [ ] **Step 2: Write `gen.rs`**

```rust
//! Synthetic bank/CC statement PDF generator for parser tests. No PII.
use printpdf::*;
use std::io::BufWriter;

pub struct Col { pub title: &'static str, pub x_mm: f64 }
pub struct Row { pub cells: Vec<String> }   // aligned to Col order; "" = empty
pub struct StatementSpec {
    pub cols: Vec<Col>,
    pub rows: Vec<Row>,          // continuation rows: empty first cell
    pub repeat_header_pages: bool,
    pub preamble: Vec<String>,   // lines above the table
}

pub fn statement(spec: &StatementSpec) -> Vec<u8> {
    let (doc, page1, layer1) = PdfDocument::new("stmt", Mm(210.0), Mm(297.0), "l1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let mut layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 270.0;
    for line in &spec.preamble {
        layer.use_text(line, 10.0, Mm(15.0), Mm(y), &font);
        y -= 6.0;
    }
    y -= 4.0;
    for c in &spec.cols { layer.use_text(c.title, 9.0, Mm(c.x_mm), Mm(y), &font); }
    y -= 6.0;
    for row in &spec.rows {
        for (c, val) in spec.cols.iter().zip(&row.cells) {
            if !val.is_empty() { layer.use_text(val, 9.0, Mm(c.x_mm), Mm(y), &font); }
        }
        y -= 6.0;
        if y < 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "l");
            layer = doc.get_page(p).get_layer(l);
            y = 270.0;
            if spec.repeat_header_pages {
                for c in &spec.cols { layer.use_text(c.title, 9.0, Mm(c.x_mm), Mm(y), &font); }
                y -= 6.0;
            }
        }
    }
    let mut buf = Vec::new();
    doc.save(&mut BufWriter::new(&mut buf)).unwrap();
    buf
}
```
(Confirm `printpdf` 0.7.x API — `use_text`, `save` signature. Adjust if the pinned version differs; keep `statement()` signature.)

- [ ] **Step 3: Write the failing integration tests**

`backend/tests/pdf_parser.rs`:
```rust
#[path = "pdf/gen.rs"]
mod gen;

use gen::{Col, Row, StatementSpec};
use khata::ingest::pdf::{parse_pdf, extract};
use khata::ingest::profiles;

fn ready() -> bool {
    extract::init(std::env::var("PDFIUM_LIB_PATH").ok().as_deref()).is_ok()
}
fn row(cells: &[&str]) -> Row { Row { cells: cells.iter().map(|s| s.to_string()).collect() } }

fn hdfc_basic_spec() -> StatementSpec {
    StatementSpec {
        cols: vec![
            Col { title: "Date", x_mm: 15.0 },
            Col { title: "Narration", x_mm: 40.0 },
            Col { title: "Withdrawal Amt", x_mm: 120.0 },
            Col { title: "Deposit Amt", x_mm: 150.0 },
            Col { title: "Closing Balance", x_mm: 180.0 },
        ],
        rows: vec![
            row(&["01/02/24", "UPI-SWIGGY-416823456789", "450.00", "", "9,550.00"]),
            row(&["02/02/24", "NEFT SALARY CREDIT", "", "50,000.00", "59,550.00"]),
            row(&["03/02/24", "ATM WDL", "2,000.00", "", "57,550.00"]),
            row(&["05/02/24", "IMPS RENT", "15,000.00", "", "42,550.00"]),
            row(&["07/02/24", "INTEREST", "", "125.00", "42,675.00"]),
        ],
        repeat_header_pages: false,
        preamble: vec!["HDFC BANK".into(), "Statement of account".into()],
    }
}

#[test]
fn account_hdfc_basic_parses_all_rows_high_confidence() {
    if !ready() { eprintln!("skip: no pdfium"); return; }
    let pdf = gen::statement(&hdfc_basic_spec());
    let (rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert_eq!(rows.len(), 5, "all 5 transactions");
    assert_eq!(reason, None, "balances reconcile -> no warning");
    assert_eq!(rows[0].debit, Some(450.00));
    assert!(rows[0].description.contains("SWIGGY"));
    assert_eq!(rows[1].credit, Some(50_000.00));
    assert_eq!(rows[1].debit, None, "credit row must not be a debit");
}

#[test]
fn spaced_dates_are_not_missed() {
    if !ready() { return; }
    let mut spec = hdfc_basic_spec();
    for (i, r) in spec.rows.iter_mut().enumerate() {
        r.cells[0] = format!("{:02} Feb 2024", i + 1);
    }
    let pdf = gen::statement(&spec);
    let (rows, ..) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert_eq!(rows.len(), 5);
}

#[test]
fn wrong_balance_is_flagged_low() {
    if !ready() { return; }
    let mut spec = hdfc_basic_spec();
    spec.rows[2].cells[4] = "1.00".into(); // break a balance
    let pdf = gen::statement(&spec);
    let (_rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert!(reason.unwrap().contains("reconcile"));
}

#[test]
fn credit_card_layout_reads_cr_dr() {
    if !ready() { return; }
    let spec = StatementSpec {
        cols: vec![
            Col { title: "Date", x_mm: 15.0 },
            Col { title: "Transaction Details", x_mm: 45.0 },
            Col { title: "Amount", x_mm: 160.0 },
        ],
        rows: vec![
            row(&["05/02/2024", "AMAZON IN", "2,000.00 Dr"]),
            row(&["07/02/2024", "REFUND ETSY", "500.00 Cr"]),
            row(&["09/02/2024", "SWIGGY", "800.00 Dr"]),
        ],
        repeat_header_pages: false,
        preamble: vec!["Your Credit Card Statement".into(), "Total Amount Due 2,300.00".into()],
    };
    let pdf = gen::statement(&spec);
    let (rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::generic_cc::profile()).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].debit, Some(2_000.00));
    assert_eq!(rows[1].credit, Some(500.00));
    assert_eq!(reason, None, "2000 - 500 + 800 = 2300 matches total due");
}

#[test]
fn multiline_narration_is_joined() {
    if !ready() { return; }
    let mut spec = hdfc_basic_spec();
    spec.rows.insert(1, row(&["", "CONTD PART TWO OF NARRATION", "", "", ""]));
    let pdf = gen::statement(&spec);
    let (rows, ..) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert_eq!(rows.len(), 5, "continuation is not a new transaction");
    assert!(rows[0].description.contains("PART TWO"));
}

#[test]
fn repeated_page_header_does_not_create_rows() {
    if !ready() { return; }
    let mut spec = hdfc_basic_spec();
    spec.repeat_header_pages = true;
    for i in 0..40 { spec.rows.push(row(&[
        &format!("{:02}/03/24", (i % 28) + 1), "FILLER TXN", "1.00", "", "42,674.00",
    ])); }
    let pdf = gen::statement(&spec);
    let (rows, ..) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert!(rows.iter().all(|r| r.description != "Narration"));
}

#[test]
fn unreadable_layout_falls_back_with_warning() {
    if !ready() { return; }
    let spec = StatementSpec {
        cols: vec![Col { title: "Foo", x_mm: 15.0 }, Col { title: "Bar", x_mm: 80.0 }],
        rows: vec![row(&["01/02/2024 something 100.00", ""])],
        repeat_header_pages: false,
        preamble: vec![],
    };
    let pdf = gen::statement(&spec);
    let (_rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::generic::profile()).unwrap();
    assert!(reason.is_some());
}
```

- [ ] **Step 4: Run to verify failure**

Run: `cd backend && cargo test --test pdf_parser`
Expected: compile error (`khata::ingest::pdf` not public / `gen` API) then, once compiling, failures where the parser is wrong.

- [ ] **Step 5: Make them pass**

Replace the stub in `src/ingest/pdf/mod.rs`:
```rust
#[cfg(test)]
pub(crate) fn test_fixture(_name: &str) -> Option<Vec<u8>> { None }
```
— delete it (integration tests build their own PDFs; the earlier `src/`-level gated test that referenced `test_fixture` should be updated to early-return unconditionally or removed, since `tests/pdf_parser.rs` now covers it).

Iterate on `table.rs` / `rows.rs` constants (`Y_TOL`, `MAX_SCAN`, header match threshold) until all integration tests pass with pdfium present. Likely adjustments:
- `Y_TOL` may need to be a fraction of line height rather than a constant.
- `columns()` should also try the row whose matched-kind x-centres are strictly increasing (reject a "header" that matched words out of column order).
- totals/preamble detection interaction with the CC "Total Amount Due" preamble line (it has no date and no Description-band match → treated as preamble → fine).

- [ ] **Step 6: Run the full suite (both with and without pdfium)**

Run:
```bash
cd backend
cargo test 2>&1 | tail -20
PDFIUM_LIB_PATH="$PWD/.pdfium/lib" cargo test 2>&1 | tail -20
```
Expected: green both ways. Without pdfium the integration tests self-skip; unit tests all run.

- [ ] **Step 7: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/tests/ backend/src/ingest/pdf/mod.rs backend/src/ingest/
git commit -m "test(ingest): synthetic PDF statement fixtures + parser integration tests"
```

---

### Task 9: CI, docs, manual verification

**Files:**
- Modify: `.github/workflows/*.yml` (whichever runs backend tests — add pdfium fetch step)
- Modify: `AGENTS.md`, `HANDOFF.md`
- Modify: `backend/README` (or create `backend/README.md`)

- [ ] **Step 1: CI**

In the workflow that runs `cargo test` for the backend, before the test step:
```yaml
- name: Fetch pdfium
  run: |
    cd backend && ./scripts/fetch-pdfium.sh linux x64
    echo "PDFIUM_LIB_PATH=$PWD/.pdfium/lib" >> "$GITHUB_ENV"
```
If no backend-test workflow exists, add `.github/workflows/backend-test.yml` running `cargo test` + `cargo clippy -- -D warnings` on push/PR touching `backend/**`.

- [ ] **Step 2: Docs**

- `AGENTS.md` / `HANDOFF.md`: note the PDF parser now uses pdfium; `PDFIUM_LIB_PATH` env var; `backend/scripts/fetch-pdfium.sh` for local setup; the fallback behaviour; `docs/superpowers/specs/2026-09-09-pdf-table-parser-design.md`.
- Update `HANDOFF.md` version/status line while here (it is stale at v0.43.0).
- `backend/README.md`: "PDF statement parsing" section — how bands/profiles work, how to add a bank's `pdf_columns`, how to add a fixture.

- [ ] **Step 3: Deploy prerequisites note**

Add to the plan's closing checklist / HANDOFF: production deploy needs the Dockerfile rebuild (pulls libpdfium); no migration for this feature.

- [ ] **Step 4: Manual verification**

```bash
cd backend
PDFIUM_LIB_PATH="$PWD/.pdfium/lib" cargo run &
# upload a real statement via the frontend or curl to /api/ingest/upload
```
If the user can provide one real (throwaway/redacted) statement, add it under `backend/tests/pdf/fixtures/real/` **only if they confirm it carries no live PII**; otherwise ship on synthetic coverage. Record the manual result in the PR description.

- [ ] **Step 5: Commit + open PR**

```bash
git add .github/ AGENTS.md HANDOFF.md backend/README.md docs/
git commit -m "ci+docs: pdfium in CI, PDF parser documentation"
git push -u origin feat/pdf-table-parser
gh pr create --title "Coordinate-aware PDF statement parser" --body "$(cat <<'EOF'
Replaces the layout-blind pdf-extract parser with pdfium-render word extraction
+ column-band table reconstruction driven by per-bank profiles. Adds credit-card
statement support, balance reconciliation, low-confidence warnings, and a
synthetic-PDF test suite.

Spec: docs/superpowers/specs/2026-09-09-pdf-table-parser-design.md
Plan: docs/superpowers/plans/2026-09-09-pdf-table-parser.md

Deploy: Dockerfile rebuild pulls libpdfium; no DB migration.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_013HRjuk415gQzJDSDbi2xZA
EOF
)"
```

- [ ] **Step 6: Version bump + release** (per project rules — only after PR review/merge)

Follow the project's release ritual: merge to `master`, bump (CHANGELOG entry), tag `v0.45.0` (new feature → minor bump), push tag, monitor CI, notify `agent-releases` on success.

---

## Self-Review

**Spec coverage:**
- pdfium extraction → Task 2. Row clustering / bands / cells → Task 3. Amount parsing + assembly + continuation + totals → Task 4. Confidence (account + CC) → Task 5. Fallback + surfacing plumbing → Task 6 + 7. Profile types + CC profile + CC detection → Task 1 + Task 6 step 6. Deployment (Dockerfile, env var, fetch script) → Task 2 + Task 9. Testing (synthetic fixtures, all 9 spec cases) → Task 8. Docs → Task 9. All spec sections covered.
- Spec case list (9 fixtures) vs Task 8 tests: basic✓ spaced-date✓ multiline✓ number-in-desc✓ (covered inside `account_hdfc_basic` via the SWIGGY UPI ref assertion) credit-only✓ (row[1] assertion) repeated-header✓ broken-balance✓ cc✓ garbage✓. The "number in description" case is asserted but not its own fixture — acceptable, it is exercised.

**Placeholder scan:** No TBD/TODO. Each code step has real code. Two explicit "confirm the API against the pinned version" notes (pdfium-render accessors, printpdf `use_text`/`save`) — these are legitimate version-pinning instructions, not placeholders, and name the exact symbols to check.

**Type consistency:**
- `Word { text, x0, x1, y0, y1, page }` — consistent Tasks 2, 3, 6.
- `Band { kind, x0, x1 }`, `columns() -> Option<Vec<Band>>`, `cells() -> BTreeMap<ColKind, String>` — consistent Tasks 3, 6.
- `assemble(Vec<BTreeMap<ColKind,String>>, &BankProfile) -> Vec<RawRow>` — consistent Tasks 4, 6.
- `Confidence::{Ok, Low(String)}`, `confidence(&[RawRow], StatementKind, &str)` — consistent Tasks 5, 6.
- `pdf::parse_pdf -> Result<(Vec<RawRow>, Vec<String>, String, Option<String>)>` — consistent Tasks 6, 7, 8.
- `parse_file` 4-tuple — Task 7 updates all callers (handlers.rs, email/run.rs via safe_parse).
- `UploadResponse.warnings: Vec<String>` — Task 7 backend + frontend + Android.
- `ColKind`, `StatementKind`, `PdfColumn`, `pdf_columns`, `statement_kind` — Task 1, used everywhere after.

**Scope:** One subsystem (ingest PDF path). Sequential; Task 8 may loop on Task 3/4 constants — expected for a parser. Task 8 Step 1 (lib target) is a prerequisite refactor, committed separately. Good for a single plan.
