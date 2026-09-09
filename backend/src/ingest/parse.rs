use anyhow::{Context, Result};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use std::io::Cursor;

use super::{detect::FileKind, models::RawRow, profiles::BankProfile};

const MAX_DECOMPRESSED_SIZE: u64 = 50 * 1024 * 1024; // 50 MB
const MAX_PARSE_ROWS: usize = 100_000;

/// Returns (raw_rows, column_headers, full_file_text_for_bank_detection)
/// The third value is all row text joined — used to detect the bank from preamble rows.
/// Run `f`, turning a panic into an `Err`.
///
/// `pdf_extract` asserts on malformed font tables (observed in the wild:
/// "assertion `left == right` failed: 257 vs 255"), and a panic in an axum
/// handler takes the request down rather than returning a response. Every
/// entry point into the parsers goes through this.
pub fn guard<T>(what: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(_) => Err(anyhow::anyhow!("{what} crashed on this file")),
    }
}

/// `parse_file`, but a parser panic becomes an `Err` instead of killing the
/// caller's task. Prefer this at every request/worker boundary.
pub fn parse_file_safe(
    bytes: &[u8],
    kind: FileKind,
    profile: &BankProfile,
) -> Result<(Vec<RawRow>, Vec<String>, String, Option<String>)> {
    guard("parser", || parse_file(bytes, kind, profile))
}

pub fn parse_file(
    bytes: &[u8],
    kind: FileKind,
    profile: &BankProfile,
) -> Result<(Vec<RawRow>, Vec<String>, String, Option<String>)> {
    match kind {
        FileKind::Excel => {
            let (r, h, t) = parse_excel(bytes, profile)?;
            Ok((r, h, t, None))
        }
        FileKind::Csv => {
            let (r, h, t) = parse_csv(bytes, profile)?;
            Ok((r, h, t, None))
        }
        FileKind::Pdf => crate::ingest::pdf::parse_pdf(bytes, profile),
    }
}

/// Check if any cell value starts with a formula-injection trigger character.
/// If found, neutralise by stripping the trigger char and prefixing with `'`.
pub fn has_formula_injection(row: &mut [String]) -> bool {
    let triggers = ['=', '+', '-', '@', '\t', '\r'];
    let mut found = false;
    for cell in row.iter_mut() {
        if let Some(c) = cell.chars().next() {
            if triggers.contains(&c) {
                let rest: String = cell.chars().skip(1).collect();
                *cell = format!("'{rest}");
                found = true;
            }
        }
    }
    found
}

/// Check total uncompressed size of the zip entries to guard against zip bombs.
fn check_excel_bomb(bytes: &[u8]) -> Result<(), anyhow::Error> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).context("invalid zip archive")?;

    let mut total: u64 = 0;
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .context("failed to read zip entry metadata")?;
        total += file.size();
        if total > MAX_DECOMPRESSED_SIZE {
            anyhow::bail!(
                "Decompressed size exceeds limit of {} MB",
                MAX_DECOMPRESSED_SIZE / (1024 * 1024)
            );
        }
    }
    Ok(())
}

/// `.xlsx` / `.ods` are zip containers; a legacy `.xls` is an OLE2 compound
/// file. Only the zip formats can be zip-bombs.
fn is_zip(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[..4] == [0x50, 0x4B, 0x03, 0x04]
}

fn parse_excel(bytes: &[u8], profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>, String)> {
    // Guard against zip bombs — but only for zip-based workbooks. A legacy .xls
    // is OLE2, not a zip, and would fail zip parsing outright.
    if is_zip(bytes) {
        check_excel_bomb(bytes)?;
    }

    // Calamine does not process XXE by default – it reads raw XML without entity resolution,
    // so XXE is not a concern with this parser.

    let cursor = Cursor::new(bytes);
    let mut wb = open_workbook_auto_from_rs(cursor).context("open excel")?;
    let sheet_name = wb.sheet_names().first().cloned().unwrap_or_default();
    let range = wb.worksheet_range(&sheet_name).context("read sheet")?;
    let rows: Vec<Vec<String>> = range
        .rows()
        .take(MAX_PARSE_ROWS)
        .map(|row| {
            row.iter()
                .map(|c| match c {
                    Data::String(s) => s.trim().to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::Empty => String::new(),
                    _ => c.to_string(),
                })
                .collect()
        })
        .collect();
    // Include sheet name in hint so keywords embedded there are found too
    let hint_prefix = format!("{sheet_name} ");
    extract_rows(rows, profile, &hint_prefix)
}

fn parse_csv(bytes: &[u8], profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>, String)> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(bytes);

    let mut rows: Vec<Vec<String>> = Vec::new();
    if let Ok(hdrs) = rdr.headers() {
        let mut h: Vec<String> = hdrs.iter().map(|s| s.to_string()).collect();
        has_formula_injection(&mut h);
        rows.push(h);
    }
    for rec in rdr.records().flatten() {
        if rows.len() >= MAX_PARSE_ROWS {
            break;
        }
        let mut r: Vec<String> = rec.iter().map(|s| s.to_string()).collect();
        has_formula_injection(&mut r);
        rows.push(r);
    }
    extract_rows(rows, profile, "")
}

fn extract_rows(
    rows: Vec<Vec<String>>,
    profile: &BankProfile,
    hint_prefix: &str,
) -> Result<(Vec<RawRow>, Vec<String>, String)> {
    let header_idx = rows
        .iter()
        .position(|r| {
            let joined = r.join(" ").to_lowercase();
            profile.description_aliases.iter().any(|a| joined.contains(a))
        })
        .unwrap_or(0);

    // Full file hint = sheet name + all rows up to and including the header row
    let file_hint = format!(
        "{hint_prefix}{}",
        rows.iter()
            .take(header_idx + 1)
            .map(|r| r.join(" "))
            .collect::<Vec<_>>()
            .join(" ")
    )
    .to_lowercase();

    let headers: Vec<String> = rows
        .get(header_idx)
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let col = |aliases: &[&str]| -> Option<usize> {
        aliases
            .iter()
            .find_map(|a| headers.iter().position(|h| h.contains(a)))
    };

    let txn_date_col = col(profile.txn_date_aliases);
    let val_date_col = col(profile.value_date_aliases);
    let desc_col = col(profile.description_aliases);
    let debit_col = col(profile.debit_aliases);
    let credit_col = col(profile.credit_aliases);
    let amount_col = col(profile.amount_aliases);
    let balance_col = col(profile.balance_aliases);
    let ref_col = col(profile.ref_aliases);

    let mut raw_rows = Vec::new();
    for row in rows.iter().skip(header_idx + 1) {
        if row.iter().all(|c| c.is_empty()) {
            continue;
        }

        let get = |c: Option<usize>| -> String {
            c.and_then(|i| row.get(i)).cloned().unwrap_or_default()
        };

        let parse_amount = |s: &str| -> Option<f64> {
            let s = s.replace([',', ' '], "");
            if s.is_empty() { None } else { s.parse().ok() }
        };

        let (debit, credit) = if !profile.amount_aliases.is_empty() {
            let v: f64 = parse_amount(&get(amount_col)).unwrap_or(0.0);
            if v < 0.0 {
                (Some(-v), None)
            } else {
                (None, Some(v))
            }
        } else {
            (parse_amount(&get(debit_col)), parse_amount(&get(credit_col)))
        };

        let bank_ref = {
            let r = get(ref_col);
            if r.is_empty() { None } else { Some(r) }
        };

        raw_rows.push(RawRow {
            txn_date: get(txn_date_col),
            value_date: get(val_date_col),
            description: get(desc_col),
            debit,
            credit,
            balance: parse_amount(&get(balance_col)),
            bank_ref,
        });
    }

    Ok((raw_rows, headers, file_hint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::profiles::generic;

    #[test]
    fn is_zip_only_matches_pk_header() {
        assert!(is_zip(b"PK\x03\x04rest-of-xlsx"));
        // Legacy .xls OLE2 compound-file magic must NOT be treated as a zip.
        assert!(!is_zip(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]));
        assert!(!is_zip(b"%PDF-1.7"));
        assert!(!is_zip(b"PK"));
    }

    #[test]
    fn guard_turns_a_panic_into_an_err() {
        // pdf_extract asserts on malformed font tables; in an axum handler that
        // would kill the request instead of returning a response.
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {})); // keep the test output clean
        let r: Result<()> = guard("parser", || panic!("assertion failed: 257 == 255"));
        std::panic::set_hook(prev);
        let e = r.unwrap_err().to_string();
        assert!(e.contains("crashed on this file"), "got: {e}");
    }

    #[test]
    fn guard_passes_through_ok_and_err() {
        assert_eq!(guard("parser", || Ok(7)).unwrap(), 7);
        let e: Result<u8> = guard("parser", || Err(anyhow::anyhow!("normal failure")));
        assert!(e.unwrap_err().to_string().contains("normal failure"));
    }

    #[test]
    fn parse_file_csv_reports_no_warning() {
        let csv = b"Date,Narration,Debit,Credit,Balance\n01/02/2024,Test,100.00,,900.00\n02/02/2024,Two,,50.00,950.00\n";
        let (_rows, _h, _t, reason) =
            parse_file(csv, FileKind::Csv, &generic::profile()).unwrap();
        assert_eq!(reason, None);
    }

    #[test]
    fn legacy_xls_skips_the_zip_bomb_guard() {
        // An OLE2 blob is not a valid workbook, so parsing still fails — but it
        // must fail in calamine, never in the zip-bomb guard ("invalid zip
        // archive"), which used to turn every .xls upload into a 400.
        let ole2 = [0xD0u8, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
            .iter()
            .copied()
            .chain(std::iter::repeat(0).take(512))
            .collect::<Vec<u8>>();
        let err = parse_excel(&ole2, &generic::profile()).unwrap_err().to_string();
        assert!(
            !err.contains("zip"),
            "xls must not hit the zip-bomb guard, got: {err}"
        );
    }
}
