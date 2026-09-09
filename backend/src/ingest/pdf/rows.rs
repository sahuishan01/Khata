use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::ingest::models::RawRow;
use crate::ingest::profiles::{BankProfile, ColKind, StatementKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sign {
    Cr,
    Dr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Signed {
    pub value: f64,
    pub sign: Option<Sign>,
}

/// Parse one money cell. Handles Indian digit grouping ("1,23,456.78"),
/// currency prefixes ("₹1,234", "Rs. 1,234"), trailing "Cr"/"Dr" markers,
/// parenthesised and signed negatives. Returns `None` when there is no
/// parseable numeric content.
pub fn amount(cell: &str) -> Option<Signed> {
    let raw = cell.trim();
    if raw.is_empty() {
        return None;
    }

    let mut s = raw.to_string();
    let mut sign: Option<Sign> = None;

    // Trailing CR / DR token (case-insensitive), only when preceded by a
    // non-letter so we don't clip a real word.
    let trimmed = s.trim_end().to_string();
    let up = trimmed.to_uppercase();
    for (tok, tok_sign) in [("CR", Sign::Cr), ("DR", Sign::Dr)] {
        if up.ends_with(tok) {
            let before_ok = up.len() == tok.len()
                || !up.as_bytes()[up.len() - tok.len() - 1].is_ascii_alphabetic();
            if before_ok {
                sign = Some(tok_sign);
                s.truncate(trimmed.len() - tok.len());
                break;
            }
        }
    }

    let mut s = s.trim().to_string();

    let paren = s.starts_with('(') && s.ends_with(')');
    if paren {
        s = s[1..s.len() - 1].trim().to_string();
    }

    let trailing_minus = s.ends_with('-');

    // Strip currency symbols / whitespace / explicit signs.
    let no_currency: String = s
        .chars()
        .filter(|c| !matches!(c, '₹' | '$' | ' ' | '+' | '\t'))
        .collect();
    let no_currency = no_currency
        .strip_prefix("Rs.")
        .or_else(|| no_currency.strip_prefix("RS."))
        .or_else(|| no_currency.strip_prefix("rs."))
        .or_else(|| no_currency.strip_prefix("Rs"))
        .or_else(|| no_currency.strip_prefix("RS"))
        .or_else(|| no_currency.strip_prefix("rs"))
        .or_else(|| no_currency.strip_prefix("INR"))
        .or_else(|| no_currency.strip_prefix("inr"))
        .unwrap_or(&no_currency)
        .trim();
    let leading_minus = no_currency.starts_with('-');

    let core: String = no_currency.chars().filter(|c| *c != '-').collect();
    let core = core
        .strip_prefix("Rs.")
        .or_else(|| core.strip_prefix("RS."))
        .or_else(|| core.strip_prefix("rs."))
        .or_else(|| core.strip_prefix("Rs"))
        .or_else(|| core.strip_prefix("RS"))
        .or_else(|| core.strip_prefix("rs"))
        .or_else(|| core.strip_prefix("INR"))
        .or_else(|| core.strip_prefix("inr"))
        .unwrap_or(&core)
        .to_string();

    if core.is_empty() {
        return None;
    }
    // Must be entirely digits / commas / dots, with at least one digit.
    if !core.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '.') {
        return None;
    }
    if !core.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    let cleaned = core.replace(',', "");
    let cleaned = cleaned.trim_matches('.');
    let value: f64 = cleaned.parse().ok()?;

    if sign.is_none() && (paren || trailing_minus || leading_minus) {
        sign = Some(Sign::Dr);
    }

    Some(Signed { value, sign })
}

fn parse_date(s: &str, profile: &BankProfile) -> Option<NaiveDate> {
    let s = s.trim();
    profile
        .date_formats
        .iter()
        .find_map(|f| NaiveDate::parse_from_str(s, f).ok())
}

fn is_totals(cells: &BTreeMap<ColKind, String>) -> bool {
    cells
        .get(&ColKind::Description)
        .map(|d| {
            let d = d.to_lowercase();
            d.contains("closing balance")
                || d.contains("opening balance")
                || d.contains("grand total")
                || d.split(|c: char| !c.is_alphanumeric()).next() == Some("total")
        })
        .unwrap_or(false)
}

fn get<'a>(c: &'a BTreeMap<ColKind, String>, k: ColKind) -> &'a str {
    c.get(&k).map(|s| s.as_str()).unwrap_or("")
}

/// Build [`RawRow`]s from ordered per-row cell maps. A row starts a
/// transaction when its `TxnDate` cell parses against `profile.date_formats`.
/// Other rows after the first start are continuation lines merged into the
/// previous row. Preamble rows and totals rows are dropped.
pub fn assemble(rows: Vec<BTreeMap<ColKind, String>>, profile: &BankProfile) -> Vec<RawRow> {
    let mut out: Vec<RawRow> = Vec::new();
    let mut started = false;

    for cells in rows {
        let date = cells
            .get(&ColKind::TxnDate)
            .and_then(|s| parse_date(s, profile));

        if date.is_some() {
            if is_totals(&cells) {
                continue;
            }
            started = true;
            out.push(build_row(&cells, profile));
        } else if started {
            if is_totals(&cells) {
                continue;
            }
            if let Some(row) = out.last_mut() {
                merge_continuation(row, &cells);
            }
        }
        // else: preamble, ignore
    }

    out
}

fn build_row(c: &BTreeMap<ColKind, String>, profile: &BankProfile) -> RawRow {
    let txn_date = get(c, ColKind::TxnDate).trim().to_string();
    let value_date = {
        let v = get(c, ColKind::ValueDate).trim();
        if v.is_empty() {
            txn_date.clone()
        } else {
            v.to_string()
        }
    };

    let (mut debit, mut credit) = (None, None);
    match profile.statement_kind {
        StatementKind::CreditCard => {
            if let Some(a) = amount(get(c, ColKind::Amount)) {
                match a.sign {
                    Some(Sign::Cr) => credit = Some(a.value),
                    _ => debit = Some(a.value),
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
                        _ => debit = Some(a.value),
                    }
                }
            }
        }
    }

    let bank_ref = {
        let r = get(c, ColKind::Ref).trim();
        if r.is_empty() {
            None
        } else {
            Some(r.to_string())
        }
    };

    RawRow {
        txn_date,
        value_date,
        description: get(c, ColKind::Description).trim().to_string(),
        debit,
        credit,
        balance: amount(get(c, ColKind::Balance)).map(|a| a.value),
        bank_ref,
    }
}

fn merge_continuation(row: &mut RawRow, c: &BTreeMap<ColKind, String>) {
    let extra = get(c, ColKind::Description).trim();
    if !extra.is_empty() {
        if !row.description.is_empty() {
            row.description.push(' ');
        }
        row.description.push_str(extra);
    }
    if row.debit.is_none() {
        row.debit = amount(get(c, ColKind::Debit)).map(|a| a.value);
    }
    if row.credit.is_none() {
        row.credit = amount(get(c, ColKind::Credit)).map(|a| a.value);
    }
    if row.balance.is_none() {
        row.balance = amount(get(c, ColKind::Balance)).map(|a| a.value);
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(amount("Rs. -1,234.00").unwrap().sign, Some(Sign::Dr));
    }

    #[test]
    fn assemble_keeps_dated_row_with_total_in_narration() {
        let rows = vec![cell(&[
            (ColKind::TxnDate, "01/02/2024"),
            (ColKind::Description, "TOTALENERGIES FUEL"),
            (ColKind::Debit, "3,200.00"),
        ])];
        let out = assemble(rows, &generic::profile());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].description, "TOTALENERGIES FUEL");
        assert_eq!(out[0].debit, Some(3200.0));
    }

    fn cell(pairs: &[(ColKind, &str)]) -> BTreeMap<ColKind, String> {
        pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
    }

    #[test]
    fn assemble_account_rows_with_continuation_and_totals() {
        let rows = vec![
            cell(&[(ColKind::TxnDate, "col headers ignored")]),
            cell(&[
                (ColKind::TxnDate, "01/02/2024"),
                (ColKind::Description, "UPI swiggy"),
                (ColKind::Debit, "450.00"),
                (ColKind::Balance, "9,550.00"),
            ]),
            cell(&[(ColKind::Description, "ref 416823456789")]),
            cell(&[
                (ColKind::TxnDate, "02/02/2024"),
                (ColKind::Description, "SALARY"),
                (ColKind::Credit, "50,000.00"),
                (ColKind::Balance, "59,550.00"),
            ]),
            cell(&[
                (ColKind::Description, "Closing Balance"),
                (ColKind::Balance, "59,550.00"),
            ]),
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
            cell(&[
                (ColKind::TxnDate, "05/02/2024"),
                (ColKind::Description, "AMAZON"),
                (ColKind::Amount, "2,000.00 Dr"),
            ]),
            cell(&[
                (ColKind::TxnDate, "07/02/2024"),
                (ColKind::Description, "REFUND"),
                (ColKind::Amount, "500.00 Cr"),
            ]),
        ];
        let out = assemble(rows, &generic_cc::profile());
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].debit, Some(2000.0));
        assert_eq!(out[0].credit, None);
        assert_eq!(out[1].credit, Some(500.0));
        assert_eq!(out[1].debit, None);
    }
}
