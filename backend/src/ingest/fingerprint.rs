use sha2::{Digest, Sha256};
use super::models::NormalizedTxn;

pub fn compute_fingerprint(t: &NormalizedTxn) -> String {
    let mut h = Sha256::new();
    h.update(t.user_id.as_bytes());
    h.update(t.account_label.as_bytes());

    if let Some(ref r) = t.bank_ref {
        h.update(r.as_bytes());
        h.update(t.amount.to_bits().to_le_bytes());
    } else {
        h.update(t.value_date.to_string().as_bytes());
        h.update(t.amount.to_bits().to_le_bytes());
        h.update(t.direction.as_bytes());
        if let Some(b) = t.balance {
            h.update(b.to_bits().to_le_bytes());
        }
        let narration = normalize_narration(&t.description);
        h.update(narration.as_bytes());
    }
    hex::encode(h.finalize())
}

fn normalize_narration(s: &str) -> String {
    let upper = s.to_uppercase();
    let cleaned = strip_trailing_numeric_refs(&upper);
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_trailing_numeric_refs(s: &str) -> String {
    s.split('/')
        .filter(|part| {
            !(part.chars().all(|c| c.is_ascii_digit()) && part.len() >= 8)
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn base_txn(uid: Uuid) -> NormalizedTxn {
        NormalizedTxn {
            user_id: uid,
            statement_id: Uuid::nil(),
            bank: "HDFC".into(),
            account_label: "XX1234".into(),
            txn_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            value_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            description: "UPI/AMAZON".into(),
            raw_description: "UPI/AMAZON/xyz123".into(),
            amount: 500.0,
            direction: "debit".into(),
            balance: Some(10000.0),
            bank_ref: None,
            category: "Shopping".into(),
        }
    }

    #[test]
    fn same_txn_same_fingerprint() {
        let uid = Uuid::new_v4();
        let t = base_txn(uid);
        assert_eq!(compute_fingerprint(&t), compute_fingerprint(&t));
    }

    #[test]
    fn different_ref_different_fingerprint() {
        let uid = Uuid::new_v4();
        let mut a = base_txn(uid);
        a.bank_ref = Some("REF001".into());
        let mut b = base_txn(uid);
        b.bank_ref = Some("REF002".into());
        assert_ne!(compute_fingerprint(&a), compute_fingerprint(&b));
    }

    #[test]
    fn different_user_different_fingerprint() {
        let t1 = base_txn(Uuid::new_v4());
        let t2 = base_txn(Uuid::new_v4());
        assert_ne!(compute_fingerprint(&t1), compute_fingerprint(&t2));
    }
}
