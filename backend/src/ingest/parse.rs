use anyhow::{Context, Result};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use std::io::Cursor;

use super::{detect::FileKind, models::RawRow, profiles::BankProfile};

pub fn parse_file(bytes: &[u8], kind: FileKind, profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>)> {
    match kind {
        FileKind::Excel => parse_excel(bytes, profile),
        FileKind::Csv => parse_csv(bytes, profile),
    }
}

fn parse_excel(bytes: &[u8], profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>)> {
    let cursor = Cursor::new(bytes);
    let mut wb = open_workbook_auto_from_rs(cursor).context("open excel")?;
    let sheet_name = wb.sheet_names().first().cloned().unwrap_or_default();
    let range = wb.worksheet_range(&sheet_name).context("read sheet")?;
    let rows: Vec<Vec<String>> = range
        .rows()
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
    extract_rows(rows, profile)
}

fn parse_csv(bytes: &[u8], profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>)> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(bytes);

    let mut rows: Vec<Vec<String>> = Vec::new();
    if let Ok(hdrs) = rdr.headers() {
        rows.push(hdrs.iter().map(|s| s.to_string()).collect());
    }
    for rec in rdr.records().flatten() {
        rows.push(rec.iter().map(|s| s.to_string()).collect());
    }
    extract_rows(rows, profile)
}

fn extract_rows(rows: Vec<Vec<String>>, profile: &BankProfile) -> Result<(Vec<RawRow>, Vec<String>)> {
    let header_idx = rows
        .iter()
        .position(|r| {
            let joined = r.join(" ").to_lowercase();
            profile.description_aliases.iter().any(|a| joined.contains(a))
        })
        .unwrap_or(0);

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

    Ok((raw_rows, headers))
}
