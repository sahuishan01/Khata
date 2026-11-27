use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::{fingerprint::compute_fingerprint, models::NormalizedTxn};

#[derive(Debug, Clone)]
pub struct StoredTxnInfo {
    pub id: Uuid,
    pub is_transfer: bool,
    pub inserted: bool,
}

pub async fn store_transactions(
    pool: &PgPool,
    user_id: Uuid,
    txns: &[NormalizedTxn],
) -> Result<(usize, usize, Vec<StoredTxnInfo>)> {
    if txns.is_empty() {
        return Ok((0, 0, Vec::new()));
    }

    let mut tx = pool.begin().await?;

    // Scope RLS policy to this user for the transaction
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let mut inserted = 0usize;
    let mut skipped = 0usize;
    let mut stored_infos = Vec::with_capacity(txns.len());

    for t in txns {
        let fp = compute_fingerprint(t);
        let is_transfer = is_cc_bill_settlement(&t.direction, &t.description);
        let row = sqlx::query_as::<_, (Uuid, bool, bool)>(
            r#"INSERT INTO transactions
               (user_id, statement_id, bank, account_label, txn_date, value_date,
                description, raw_description, amount, direction, balance, bank_ref, fingerprint, category, is_transfer)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
               ON CONFLICT (user_id, fingerprint) DO UPDATE
               SET updated_at = transactions.updated_at
               RETURNING id, is_transfer, (xmax = 0) AS inserted"#,
        )
        .bind(t.user_id)
        .bind(t.statement_id)
        .bind(&t.bank)
        .bind(&t.account_label)
        .bind(t.txn_date)
        .bind(t.value_date)
        .bind(&t.description)
        .bind(&t.raw_description)
        .bind(t.amount)
        .bind(&t.direction)
        .bind(t.balance)
        .bind(&t.bank_ref)
        .bind(&fp)
        .bind(&t.category)
        .bind(is_transfer)
        .fetch_one(&mut *tx)
        .await?;

        if row.2 {
            inserted += 1;
        } else {
            skipped += 1;
        }
        stored_infos.push(StoredTxnInfo {
            id: row.0,
            is_transfer: row.1,
            inserted: row.2,
        });
    }

    tx.commit().await?;
    Ok((inserted, skipped, stored_infos))
}

fn is_cc_bill_settlement(direction: &str, description: &str) -> bool {
    let d = description.to_lowercase();
    if ["refund", "cashback", "reversal", "reward", "waiver", "credit adjustment", "payment at "]
        .iter()
        .any(|k| d.contains(k))
    {
        return false;
    }
    if direction == "credit" {
        return [
            "payment received",
            "payment - thank you",
            "payment thank you",
            "bbps",
            "billdesk",
            "cred",
            "neft",
            "imps",
            "rtgs",
            "upi",
            "auto debit",
            "nach",
            "ecs",
            "cheque",
            "internet banking",
            "mobile banking",
            "payment credited",
            "payment settlement",
        ]
        .iter()
        .any(|k| d.contains(k));
    }
    if direction == "debit" {
        let is_cred = d.contains("cred club")
            || d.contains("credclub")
            || d.contains("cred payment")
            || d.contains("cred bill")
            || d.contains("upi/cred")
            || d.contains("upi-cred")
            || d.contains("ach d- cred");
        if is_cred {
            return true;
        }
        let is_cc_words = d.contains("credit card")
            || d.contains("creditcard")
            || d.contains(" cc ")
            || d.starts_with("cc ")
            || d.ends_with(" cc")
            || d.contains("ccpmt")
            || d.contains("hdfccc")
            || d.contains("sbicard")
            || d.contains("icicicc")
            || d.contains("billdesk cc")
            || d.contains("bbps cc");
        let is_payment_words = d.contains("payment")
            || d.contains("repayment")
            || d.contains("pmt")
            || d.contains("bill")
            || d.contains("towards")
            || d.contains("paid")
            || d.contains("autodebit")
            || d.contains("autopay");
        return is_cc_words && is_payment_words;
    }
    false
}

/// Insert a statement record with RLS context set
pub async fn insert_statement(
    pool: &PgPool,
    user_id: Uuid,
    bank: &str,
    file_name: &str,
    file_sha256: &str,
    row_count: i32,
) -> Result<Uuid> {
    let mut tx = pool.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let (stmt_id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO statements (user_id, bank, file_name, file_sha256, row_count) VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(user_id)
    .bind(bank)
    .bind(file_name)
    .bind(file_sha256)
    .bind(row_count)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(stmt_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::ingest::models::NormalizedTxn;

    fn make_txn(user_id: Uuid, stmt_id: Uuid) -> NormalizedTxn {
        NormalizedTxn {
            user_id,
            statement_id: stmt_id,
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

    #[sqlx::test(migrations = "./migrations")]
    async fn dedup_same_txn(pool: PgPool) {
        // Users table has no RLS — insert directly
        let (user_id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash) VALUES ('a@b.com','x') RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let stmt_id = insert_statement(&pool, user_id, "HDFC", "f.csv", "abc123", 1)
            .await
            .unwrap();

        let txns = vec![make_txn(user_id, stmt_id)];

        let (ins1, skip1, infos1) = store_transactions(&pool, user_id, &txns).await.unwrap();
        assert_eq!(ins1, 1, "first insert should add 1 row");
        assert_eq!(skip1, 0);
        assert_eq!(infos1.len(), 1);
        assert_eq!(infos1[0].inserted, true);

        // Re-store identical transactions (simulate overlapping re-upload)
        let (ins2, skip2, infos2) = store_transactions(&pool, user_id, &txns).await.unwrap();
        assert_eq!(ins2, 0, "duplicate must be skipped");
        assert_eq!(skip2, 1);
        assert_eq!(infos2.len(), 1);
        assert_eq!(infos2[0].inserted, false);
    }

    #[test]
    fn test_is_cc_bill_settlement() {
        // Credit card statement payments (credits)
        assert!(is_cc_bill_settlement("credit", "PAYMENT RECEIVED - THANK YOU"));
        assert!(is_cc_bill_settlement("credit", "Auto Debit Payment Received"));
        assert!(is_cc_bill_settlement("credit", "CRED payment"));
        assert!(!is_cc_bill_settlement("credit", "Refund from Swiggy"));
        assert!(!is_cc_bill_settlement("credit", "Cashback received"));

        // Bank statement credit card payments (debits)
        assert!(is_cc_bill_settlement("debit", "UPI/CRED/123456789/payto"));
        assert!(is_cc_bill_settlement("debit", "ACH D- CRED CLUB PVT LTD"));
        assert!(is_cc_bill_settlement("debit", "CRED CLUB bill payment"));
        assert!(is_cc_bill_settlement("debit", "HDFC CREDIT CARD PAYMENT"));
        assert!(is_cc_bill_settlement("debit", "AUTOPAY HDFC CC"));
        assert!(is_cc_bill_settlement("debit", "BILLDESK CC PAYMENT"));
        assert!(!is_cc_bill_settlement("debit", "Credit card payment at AMAZON"));
        assert!(!is_cc_bill_settlement("debit", "Credit card annual fees"));
        assert!(!is_cc_bill_settlement("debit", "Swiggy food order"));
    }
}
