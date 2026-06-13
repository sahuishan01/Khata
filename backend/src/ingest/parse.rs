use anyhow::{Context, Result};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use std::io::Cursor;

use super::{detect::FileKind, models::RawRow, profiles::BankProfile};

const MAX_DECOMPRESSED_SIZE: u64 = 50 * 1024 * 1024; // 50 MB
const MAX_PARSE_ROWS: usize = 100_000;

/// Returns (raw_rows, column_headers, full_file_text_for_bank_detection)
/// The third value is all row text joined — used to detect the bank from preamble rows.
pub fn parse_file(
    bytes: &[u8],
    kind: FileKind,
    profile: &BankProfile,
) -> Result<(Vec<RawRow>, Vec<String>, String)> {
    match kind {
        FileKind::Excel => parse_excel(bytes, profile),
        FileKind::Csv => parse_csv(bytes, profile),
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
    let archive = zip::ZipArchive::new(cursor).context("invalid zip archive")?;

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

fn parse_excel(bytes: &[u8], profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>, String)> {
    // Guard against zip bombs: reject archives whose total uncompressed size exceeds the limit
    check_excel_bomb(bytes)?;

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
