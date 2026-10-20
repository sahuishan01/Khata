use axum::{extract::State, Json};
use chrono::NaiveDate;
use khata::{
    auth::middleware::CurrentUser,
    txns::{handlers, models::SyncPushOp},
    AppState,
};
use sqlx::PgPool;
use std::{collections::HashMap, sync::{Arc, Mutex}};
use uuid::Uuid;

fn state(pool: &PgPool) -> AppState {
    AppState {
        db: pool.clone(), db_ro: pool.clone(),
        config: Arc::new(khata::config::Config {
            database_url: String::new(), ro_database_url: String::new(),
            jwt_secret: "test-secret-at-least-32-characters".into(), claude_bin: "claude".into(),
            bind_addr: "127.0.0.1:0".into(), cors_origins: vec![], cookie_secure: true,
            allow_remote_setup: false, email_sync_poll_secs: 0, email_sync_max_messages: 200,
            email_sync_max_attach_bytes: 1024, pdfium_lib_path: None,
        }),
        chat_ratelimit: Arc::new(Mutex::new(HashMap::new())),
        login_attempts: Arc::new(Mutex::new(HashMap::new())),
    }
}

async fn user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users (email,password_hash) VALUES ($1,'test') RETURNING id")
        .bind(format!("{}@example.test", Uuid::new_v4())).fetch_one(pool).await.unwrap()
}

fn op(description: &str, direction: &str, amount: f64) -> SyncPushOp {
    serde_json::from_value(serde_json::json!({
        "client_id": Uuid::new_v4(), "base_rev": 0, "description": description,
        "direction": direction, "amount": amount, "txn_date": "2026-10-20",
        "value_date": "2026-10-20", "bank": "HDFC", "category": "Uncategorized",
        "is_transfer": false,
    })).unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn repayment_policy_preserves_purchases_refunds_fees_and_income(pool: PgPool) {
    let cases = [
        ("Credit card bill payment", "debit", false, true),
        ("CC PAYMENT", "debit", false, true),
        ("Rs 500 debited towards HDFC Credit Card XX1234", "debit", false, true),
        ("Payment of INR 500 received towards Kotak credit card", "credit", false, true),
        ("Payment credited to Axis credit card", "credit", false, true),
        ("Canara credit card payment settlement credited", "credit", false, true),
        ("PAYMENT RECEIVED THANK YOU", "credit", true, true),
        ("PAYMENT RECEIVED THANK YOU", "credit", false, false),
        ("Rs 500 spent using HDFC credit card at AMAZON", "debit", false, false),
        ("Credit card payment at AMAZON", "debit", false, false),
        ("Credit card payment fee", "debit", false, false),
        ("Credit card annual charges", "debit", false, false),
        ("Credit card refund credited", "credit", true, false),
        ("CASHBACK credit card payment", "credit", true, false),
        ("Credit card payment reversal", "credit", true, false),
        ("Credit card payment due on 01/11/26", "debit", false, false),
        ("Credit card payment failed", "debit", false, false),
        ("Credit card payment will be debited", "debit", false, false),
        ("Insurance settlement", "credit", false, false),
        ("SALARY NEFT", "credit", false, false),
        ("UPI payment to friend", "debit", false, false),
    ];
    for (description, direction, card, expected) in cases {
        let actual: bool = sqlx::query_scalar("SELECT is_card_repayment($1,$2,$3)")
            .bind(description).bind(direction).bind(card).fetch_one(&pool).await.unwrap();
        assert_eq!(actual, expected, "{description}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn repayments_stay_visible_but_leave_totals_and_date_repairs_sync(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let other = user(&pool).await;
    let purchase = op("AMAZON credit card purchase", "debit", 100.0);
    let purchase_id = purchase.client_id;
    let ops = vec![purchase, op("CC PAYMENT", "debit", 500.0),
        op("Credit card payment received", "credit", 500.0),
        op("Salary", "credit", 2000.0), op("Merchant refund", "credit", 40.0),
        op("Credit card payment fee", "debit", 10.0)];
    let pushed = handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(ops)).await.unwrap().0;
    assert_eq!(pushed.accepted.len(), 6);
    let dashboard = handlers::get_dashboard(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent, 110.0);
    assert_eq!(dashboard.total_earned, 2040.0);
    let analysis = handlers::get_analysis(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(analysis.total_transactions, 4);
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let counts: (i64, i64) = sqlx::query_as("SELECT count(*),count(*) FILTER (WHERE is_transfer) FROM transactions")
        .fetch_one(&mut *tx).await.unwrap();
    assert_eq!(counts, (6, 2));
    tx.commit().await.unwrap();
    let hidden = handlers::get_dashboard(State(app.clone()), CurrentUser(other)).await.unwrap().0;
    assert_eq!(hidden.total_spent, 0.0);
    assert_eq!(hidden.total_earned, 0.0);

    let mut repair = op("AMAZON credit card purchase", "debit", 100.0);
    repair.client_id = purchase_id;
    repair.base_rev = 1;
    repair.txn_date = Some(NaiveDate::from_ymd_opt(2026, 10, 18).unwrap());
    repair.value_date = repair.txn_date;
    let result = handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![repair])).await.unwrap().0;
    assert!(result.conflicts.is_empty());
    assert_eq!(result.accepted[0].server_rev, 2);
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let dates: (NaiveDate, NaiveDate) = sqlx::query_as("SELECT txn_date,value_date FROM transactions WHERE client_id=$1")
        .bind(purchase_id).fetch_one(&mut *tx).await.unwrap();
    assert_eq!(dates.0.to_string(), "2026-10-18");
    assert_eq!(dates.0, dates.1);
    tx.commit().await.unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn backfill_advances_sync_cursor_and_later_manual_override_wins(pool: PgPool) {
    let owner = user(&pool).await;
    // Simulate rows that existed before 0034's insertion trigger.
    sqlx::query("ALTER TABLE transactions DISABLE TRIGGER transaction_card_repayment").execute(&pool).await.unwrap();
    let app = state(&pool);
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![op("CC PAYMENT", "debit", 500.0)]))
        .await.unwrap();
    sqlx::query("ALTER TABLE transactions ENABLE TRIGGER transaction_card_repayment").execute(&pool).await.unwrap();
    let migration = include_str!("../migrations/0034_card_repayments.sql");
    let backfill = &migration[migration.find("ALTER TABLE transactions NO FORCE").unwrap()..];
    // Migrations run atomically, so RLS is never left relaxed on failure.
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(backfill).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let row: (Uuid, bool, i64, i64) = sqlx::query_as("SELECT id,is_transfer,rev,sync_rev FROM transactions")
        .fetch_one(&mut *tx).await.unwrap();
    assert!(row.1);
    assert_eq!(row.2, 2);
    tx.commit().await.unwrap();
    handlers::toggle_transfer(State(app), CurrentUser(owner), axum::extract::Path(row.0),
        Json(handlers::ToggleTransferReq { is_transfer: false })).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let overridden: (bool, i64, i64) = sqlx::query_as("SELECT is_transfer,rev,sync_rev FROM transactions")
        .fetch_one(&mut *tx).await.unwrap();
    assert!(!overridden.0);
    assert_eq!(overridden.1, 3);
    assert!(overridden.2 > row.3);
    tx.commit().await.unwrap();
    let forced: bool = sqlx::query_scalar("SELECT relforcerowsecurity FROM pg_class WHERE oid='transactions'::regclass")
        .fetch_one(&pool).await.unwrap();
    assert!(forced);
}
