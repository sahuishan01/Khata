use crate::ingest::models::RawRow;
use crate::ingest::profiles::StatementKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Confidence {
    Ok,
    Low(String),
}

const ACCT_TOL: f64 = 0.01;
const CC_TOL: f64 = 1.0;

/// `full_text` is the lowercased page text, used only to look for a
/// "total amount due" figure on credit-card statements.
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
    let with_balance = rows.iter().filter(|r| r.balance.is_some()).count();
    if with_balance * 2 < rows.len() {
        return Confidence::Low(
            "could not verify amounts: no running balance column found — please check the transactions"
                .into(),
        );
    }
    let no_amount = rows
        .iter()
        .filter(|r| r.debit.is_none() && r.credit.is_none())
        .count();
    if no_amount > 0 {
        return Confidence::Low(format!("{no_amount} transaction(s) have no parseable amount"));
    }

    let mut checked = 0usize;
    let mut bad = 0usize;
    let mut prev: Option<f64> = None;
    for row in rows {
        if let (Some(p), Some(b)) = (prev, row.balance) {
            let delta = row.credit.unwrap_or(0.0) - row.debit.unwrap_or(0.0);
            checked += 1;
            if (p + delta - b).abs() > ACCT_TOL {
                bad += 1;
            }
        }
        if row.balance.is_some() {
            prev = row.balance;
        }
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
        let net: f64 = rows
            .iter()
            .map(|r| r.debit.unwrap_or(0.0) - r.credit.unwrap_or(0.0))
            .sum();
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
            let num: String = tail
                .chars()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::models::RawRow;
    use crate::ingest::profiles::StatementKind;

    fn r(debit: Option<f64>, credit: Option<f64>, balance: Option<f64>) -> RawRow {
        RawRow {
            txn_date: "x".into(),
            value_date: "x".into(),
            description: "x".into(),
            debit,
            credit,
            balance,
            bank_ref: None,
        }
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
            r(None, Some(50_000.0), Some(999.0)), // wrong
            r(Some(1_000.0), None, Some(-1.0)),   // wrong
        ];
        match confidence(&rows, StatementKind::Account, "") {
            Confidence::Low(m) => assert!(m.contains("reconcile")),
            _ => panic!("expected Low"),
        }
    }

    #[test]
    fn account_low_when_too_few_rows() {
        assert!(matches!(
            confidence(&[r(Some(1.0), None, Some(1.0))], StatementKind::Account, ""),
            Confidence::Low(_)
        ));
    }

    #[test]
    fn account_low_when_no_running_balance_column() {
        let rows = vec![
            r(Some(450.0), None, None),
            r(None, Some(50_000.0), None),
            r(Some(1_000.0), None, None),
        ];
        match confidence(&rows, StatementKind::Account, "") {
            Confidence::Low(m) => assert!(m.contains("no running balance")),
            _ => panic!("expected Low"),
        }
    }

    #[test]
    fn account_low_when_a_row_has_no_amount() {
        let rows = vec![
            r(Some(450.0), None, Some(9_550.0)),
            r(None, None, Some(9_550.0)), // no debit or credit
            r(Some(1_000.0), None, Some(8_550.0)),
        ];
        match confidence(&rows, StatementKind::Account, "") {
            Confidence::Low(m) => assert!(m.contains("no parseable amount")),
            _ => panic!("expected Low"),
        }
    }

    #[test]
    fn cc_low_when_a_row_has_no_amount() {
        let rows = vec![r(Some(100.0), None, None), r(None, None, None)];
        assert!(matches!(
            confidence(&rows, StatementKind::CreditCard, ""),
            Confidence::Low(_)
        ));
    }

    #[test]
    fn cc_ok_when_sum_matches_total_due() {
        let rows = vec![r(Some(2_000.0), None, None), r(None, Some(500.0), None)];
        // net spend 1500
        assert_eq!(
            confidence(&rows, StatementKind::CreditCard, "total amount due 1,500.00"),
            Confidence::Ok
        );
    }
}
