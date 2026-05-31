use anyhow::Result;
use sqlx::PgPool;

use super::{fingerprint::compute_fingerprint, models::NormalizedTxn};

pub async fn store_transactions(pool: &PgPool, txns: &[NormalizedTxn]) -> Result<(usize, usize)> {
    let mut inserted = 0usize;
    let mut skipped = 0usize;

    for t in txns {
        let fp = compute_fingerprint(t);
        let result = sqlx::query(
            r#"INSERT INTO transactions
               (user_id, statement_id, bank, account_label, txn_date, value_date,
                description, raw_description, amount, direction, balance, bank_ref, fingerprint)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
               ON CONFLICT (user_id, fingerprint) DO NOTHING"#,
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
        .execute(pool)
        .await?;

        if result.rows_affected() == 1 {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }
    Ok((inserted, skipped))
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
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn dedup_same_txn(pool: PgPool) {
        let (user_id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash) VALUES ('a@b.com','x') RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let (stmt_id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO statements (user_id, bank, file_name, file_sha256, row_count) VALUES ($1,'HDFC','f.csv','abc123',1) RETURNING id",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let txns = vec![make_txn(user_id, stmt_id)];

        let (ins1, skip1) = store_transactions(&pool, &txns).await.unwrap();
        assert_eq!(ins1, 1);
        assert_eq!(skip1, 0);

        let (ins2, skip2) = store_transactions(&pool, &txns).await.unwrap();
        assert_eq!(ins2, 0);
        assert_eq!(skip2, 1);
    }
}
