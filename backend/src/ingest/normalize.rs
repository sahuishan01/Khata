use chrono::NaiveDate;
use uuid::Uuid;

use super::{
    categorize::categorize,
    models::{NormalizedTxn, RawRow},
    profiles::BankProfile,
};

pub fn normalize(
    rows: Vec<RawRow>,
    user_id: Uuid,
    statement_id: Uuid,
    profile: &BankProfile,
    account_label: &str,
) -> Vec<NormalizedTxn> {
    rows.into_iter()
        .filter_map(|r| {
            let value_date = parse_date(&r.value_date, profile.date_formats)
                .or_else(|| parse_date(&r.txn_date, profile.date_formats))?;
            let txn_date = parse_date(&r.txn_date, profile.date_formats).unwrap_or(value_date);

            let (amount, direction) = match (r.debit, r.credit) {
                (Some(d), _) if d > 0.0 => (d, "debit"),
                (_, Some(c)) if c > 0.0 => (c, "credit"),
                _ => return None,
            };

            Some(NormalizedTxn {
                user_id,
                statement_id,
                bank: profile.name.to_string(),
                account_label: account_label.to_string(),
                txn_date,
                value_date,
                description: r.description.trim().to_string(),
                raw_description: r.description.clone(),
                amount,
                direction: direction.to_string(),
                balance: r.balance,
                bank_ref: r.bank_ref.filter(|s| !s.is_empty()),
                category: categorize(r.description.trim(), direction).to_string(),
            })
        })
        .collect()
}

fn parse_date(s: &str, formats: &[&str]) -> Option<NaiveDate> {
    let s = s.trim();
    for fmt in formats {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::{models::RawRow, profiles::hdfc};

    #[test]
    fn normalizes_hdfc_row() {
        let profile = hdfc::profile();
        let rows = vec![RawRow {
            txn_date: "01/03/2024".into(),
            value_date: "01/03/2024".into(),
            description: "UPI/Amazon".into(),
            debit: Some(500.0),
            credit: None,
            balance: Some(10000.0),
            bank_ref: None,
        }];
        let txns = normalize(rows, Uuid::new_v4(), Uuid::new_v4(), &profile, "XX1234");
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].amount, 500.0);
        assert_eq!(txns[0].direction, "debit");
    }

    #[test]
    fn skips_rows_with_no_amount() {
        let profile = hdfc::profile();
        let txns = normalize(vec![RawRow::default()], Uuid::new_v4(), Uuid::new_v4(), &profile, "");
        assert_eq!(txns.len(), 0);
    }

    #[test]
    fn credit_row() {
        let profile = hdfc::profile();
        let rows = vec![RawRow {
            txn_date: "15/03/2024".into(),
            value_date: "15/03/2024".into(),
            description: "SALARY".into(),
            debit: None,
            credit: Some(50000.0),
            balance: Some(60000.0),
            bank_ref: Some("REF123".into()),
        }];
        let txns = normalize(rows, Uuid::new_v4(), Uuid::new_v4(), &profile, "SAVINGS");
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].direction, "credit");
        assert_eq!(txns[0].bank_ref, Some("REF123".into()));
    }
}
