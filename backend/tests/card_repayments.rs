use axum::{extract::{Query, State}, Json};
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
    let detail = handlers::get_analytics_detail(State(app.clone()), CurrentUser(owner), Query(
        khata::txns::models::DetailQuery { category: None, month: None, payee: None }
    )).await.unwrap().0;
    assert_eq!(detail.txn_count, 4);
    assert_eq!(detail.total_spent, 110.0);
    assert_eq!(detail.total_earned, 2040.0);
    let highlights = handlers::get_analytics_highlights(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    let expected_rate = (2040.0 - 110.0) / 2040.0 * 100.0;
    assert!((highlights.best_savings_month.unwrap().1 - expected_rate).abs() < 0.0001);
    for (question, field, expected) in [("What is my total spending?", "amount", 110.0),
        ("What is my total income?", "amount", 2040.0), ("How many transactions?", "total_count", 4.0)] {
        let matched = khata::chat::predefined::match_question(question, &[]).unwrap_or_else(|| panic!("Unsupported question fixture: {question}"));
        let mut tx = pool.begin().await.unwrap();
        khata::db::set_current_user(&mut *tx, owner).await.unwrap();
        let query = format!("SELECT ({})::float8 FROM ({}) AS result", field, matched.sql);
        let actual: f64 = sqlx::query_scalar(&query).fetch_one(&mut *tx).await.unwrap();
        assert_eq!(actual, expected, "{question}");
        tx.commit().await.unwrap();
    }
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

fn bank_op(bank: &str, reference: Option<&str>, direction: &str, amount: f64) -> SyncPushOp {
    let mut row = op("Bank alert", direction, amount);
    row.bank = Some(bank.into());
    row.bank_ref = reference.map(str::to_string);
    row
}

async fn identity_rows(pool: &PgPool, owner: Uuid) -> Vec<(Uuid, bool, bool, i64)> {
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let rows = sqlx::query_as("SELECT client_id,deleted,is_transfer,rev FROM transactions ORDER BY sync_rev")
        .fetch_all(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    rows
}

#[sqlx::test(migrations = "./migrations")]
async fn separate_sms_ids_with_same_bank_reference_count_once_and_cannot_resurrect(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let first = bank_op("KOTAK", Some("123456789012"), "debit", 10000.0);
    let second = bank_op("KOTAK", Some("123456789012"), "debit", 10000.0);
    let second_id = second.client_id;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![first,second])).await.unwrap();
    let rows = identity_rows(&pool, owner).await;
    assert_eq!(rows.len(), 2, "Retain both source identities");
    assert_eq!(rows.iter().filter(|r| r.1).count(), 1);
    let dashboard = handlers::get_dashboard(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent, 10000.0);
    let params = serde_json::from_value(serde_json::json!({})).unwrap();
    let listed = handlers::list_txns(State(app.clone()), CurrentUser(owner), Query(params)).await.unwrap().0;
    assert_eq!(listed.total, 1);
    assert_eq!(listed.data.len(), 1);

    let mut retry = bank_op("KOTAK", Some("123456789012"), "debit", 10000.0);
    retry.client_id = second_id;
    retry.base_rev = 1;
    retry.deleted = Some(false);
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![retry])).await.unwrap();
    let mut resolution = bank_op("KOTAK", Some("123456789012"), "debit", 10000.0);
    resolution.client_id = second_id;
    resolution.deleted = Some(false);
    handlers::sync_resolve(State(app.clone()), CurrentUser(owner), Json(vec![resolution])).await.unwrap();
    let pull = handlers::sync_pull(State(app), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(0), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0;
    assert_eq!(pull.txns.len(), 2, "Sync must include tombstones");
    assert!(pull.txns.iter().find(|r| r.client_id == Some(second_id)).unwrap().deleted);
}

#[sqlx::test(migrations = "./migrations")]
async fn no_reference_repeated_payments_and_other_tenants_are_never_merged(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let other = user(&pool).await;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![
        bank_op("KOTAK", None, "debit", 10000.0), bank_op("KOTAK", None, "debit", 10000.0),
        bank_op("Manual", Some("123456789012"), "debit", 10000.0),
        bank_op("Manual", Some("123456789012"), "debit", 10000.0),
        bank_op("HDFC", Some("123456789012"), "debit", 10000.0),
        bank_op("HDFC", Some("123456789012"), "debit", 20000.0),
        bank_op("HDFC", Some("123456"), "debit", 10000.0),
        bank_op("HDFC", Some("123456"), "debit", 10000.0),
    ])).await.unwrap();
    handlers::sync_push(State(app.clone()), CurrentUser(other), Json(vec![
        bank_op("HDFC", Some("123456789012"), "debit", 10000.0),
        bank_op("KOTAK", Some("123456789012"), "credit", 20000.0),
    ])).await.unwrap();
    for tenant in [owner, other] {
        assert!(identity_rows(&pool, tenant).await.iter().all(|r| !r.1 && !r.2));
    }
    let dashboard = handlers::get_dashboard(State(app), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent, 90000.0);
}

#[sqlx::test(migrations = "./migrations")]
async fn reference_backed_bank_transfer_updates_both_sides_and_preserves_manual_override(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let debit = bank_op("HDFC", Some("123456789012"), "debit", 500.0);
    let debit_client = debit.client_id;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![debit])).await.unwrap();
    let cursor = handlers::sync_pull(State(app.clone()), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(0), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0.new_rev;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![
        bank_op("KOTAK", Some("123456789012"), "credit", 500.0)
    ])).await.unwrap();
    assert!(identity_rows(&pool, owner).await.iter().all(|r| !r.1 && r.2));
    let dashboard = handlers::get_dashboard(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!((dashboard.total_spent, dashboard.total_earned), (0.0, 0.0));
    let pull = handlers::sync_pull(State(app.clone()), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(cursor), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0;
    assert_eq!(pull.txns.len(), 2, "Existing counterpart must advance its cursor");
    let debit_id = pull.txns.iter().find(|r| r.client_id == Some(debit_client)).unwrap().id;
    handlers::toggle_transfer(State(app.clone()), CurrentUser(owner), axum::extract::Path(debit_id),
        Json(handlers::ToggleTransferReq { is_transfer: false })).await.unwrap();
    let mut retry = bank_op("HDFC", Some("123456789012"), "debit", 500.0);
    retry.client_id = debit_client;
    retry.base_rev = 3;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![retry])).await.unwrap();
    let dashboard = handlers::get_dashboard(State(app), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent, 500.0);
}

#[sqlx::test(migrations = "./migrations")]
async fn ambiguous_references_and_distant_dates_do_not_trigger_transfer_matching(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let mut late = bank_op("KOTAK", Some("123456789012"), "credit", 500.0);
    late.value_date = Some(NaiveDate::from_ymd_opt(2026, 10, 24).unwrap());
    handlers::sync_push(State(app), CurrentUser(owner), Json(vec![
        bank_op("HDFC", Some("123456789012"), "debit", 500.0), late,
        bank_op("AXIS", Some("998877665544"), "debit", 500.0),
        bank_op("CANARA", Some("998877665544"), "debit", 500.0),
        bank_op("KOTAK", Some("998877665544"), "credit", 500.0),
    ])).await.unwrap();
    assert!(identity_rows(&pool, owner).await.iter().all(|r| !r.1 && !r.2));
}

#[sqlx::test(migrations = "./migrations")]
async fn excluded_no_reference_duplicate_leaves_all_financial_queries_but_remains_in_sync(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let mut originals = vec![op("Reported payment", "debit", 10000.0), op("Reported payment", "debit", 10000.0),
        op("Salary", "credit", 25000.0), op("Bad imported credit", "credit", 999999.0)];
    let date = chrono::Utc::now().date_naive();
    for row in &mut originals { row.txn_date = Some(date); row.value_date = Some(date); }
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(originals)).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM transactions WHERE description='Reported payment' ORDER BY id")
        .fetch_all(&mut *tx).await.unwrap();
    sqlx::query("UPDATE transactions SET duplicate_of=$1, deleted=true, rev=rev+1 WHERE id=$2")
        .bind(ids[0]).bind(ids[1]).execute(&mut *tx).await.unwrap();
    sqlx::query("UPDATE transactions SET deleted=true, rev=rev+1 WHERE description='Bad imported credit'")
        .execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO budgets (user_id,category,monthly_limit) VALUES ($1,'Uncategorized',20000)")
        .bind(owner).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let dashboard = handlers::get_dashboard(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!((dashboard.total_spent, dashboard.total_earned), (10000.0, 25000.0));
    assert_eq!(dashboard.monthly[0].earned, 25000.0);
    assert_eq!(dashboard.top_debits[0].total, 10000.0);
    let analysis = handlers::get_analysis(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(analysis.total_transactions, 2);
    let detail = handlers::get_analytics_detail(State(app.clone()), CurrentUser(owner), Query(
        khata::txns::models::DetailQuery { category: None, month: None, payee: None }
    )).await.unwrap().0;
    assert_eq!((detail.total_spent, detail.total_earned, detail.txn_count), (10000.0, 25000.0, 2));
    let highlights = handlers::get_analytics_highlights(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(highlights.biggest_income.unwrap().amount, 25000.0);
    let balances = handlers::get_account_balances(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(balances[0].total_earned, 25000.0);
    let budget = khata::budgets::handlers::budget_status(State(app.clone()), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(budget[0].spent, 10000.0);
    for (question, expected) in [("What is my total spending?", 10000.0), ("What is my total income?", 25000.0)] {
        let matched = khata::chat::predefined::match_question(question, &[]).unwrap();
        let mut tx = pool.begin().await.unwrap();
        khata::db::set_current_user(&mut *tx, owner).await.unwrap();
        let actual: f64 = sqlx::query_scalar(&format!("SELECT amount::float8 FROM ({}) AS answer", matched.sql))
            .fetch_one(&mut *tx).await.unwrap();
        assert_eq!(actual, expected);
        tx.commit().await.unwrap();
    }
    let pull = handlers::sync_pull(State(app), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(0), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0;
    assert_eq!(pull.txns.iter().filter(|r| r.deleted).count(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn identity_backfill_is_atomic_tenant_scoped_and_advances_revisions(pool: PgPool) {
    let owner = user(&pool).await;
    let other = user(&pool).await;
    let app = state(&pool);
    sqlx::query("ALTER TABLE transactions DISABLE TRIGGER transaction_identity").execute(&pool).await.unwrap();
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![
        bank_op("HDFC", Some("123456789012"), "debit", 500.0),
        bank_op("KOTAK", Some("123456789012"), "credit", 500.0),
        bank_op("HDFC", None, "debit", 10000.0), bank_op("HDFC", None, "debit", 10000.0),
    ])).await.unwrap();
    handlers::sync_push(State(app), CurrentUser(other), Json(vec![
        bank_op("HDFC", Some("123456789012"), "debit", 500.0),
    ])).await.unwrap();
    sqlx::query("ALTER TABLE transactions ENABLE TRIGGER transaction_identity").execute(&pool).await.unwrap();
    let migration = include_str!("../migrations/0035_transaction_identity.sql");
    let backfill = &migration[migration.find("ALTER TABLE transactions NO FORCE").unwrap()..];
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(backfill).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let rows = identity_rows(&pool, owner).await;
    assert_eq!(rows.iter().filter(|r| r.2 && r.3 == 2).count(), 2);
    assert!(rows.iter().all(|r| !r.1));
    assert!(!identity_rows(&pool, other).await[0].2);
    let forced: bool = sqlx::query_scalar("SELECT relforcerowsecurity FROM pg_class WHERE oid='transactions'::regclass")
        .fetch_one(&pool).await.unwrap();
    assert!(forced);
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_review_requires_confirmation_is_tenant_scoped_and_can_be_undone(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let other = user(&pool).await;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![
        op("Repeated payment", "debit", 10000.0), op("Repeated payment", "debit", 10000.0),
        op("Repeated payment", "debit", 10000.0), op("A separate payee", "debit", 10000.0),
    ])).await.unwrap();
    let list = handlers::list_txns(State(app.clone()), CurrentUser(owner), Query(
        serde_json::from_value(serde_json::json!({"duplicates_only":true})).unwrap()
    )).await.unwrap().0;
    assert_eq!(list.total, 3);
    assert!(list.data.iter().all(|r| r.possible_duplicate));
    let first = list.data[0].id;
    assert!(handlers::exclude_duplicate(State(app.clone()), CurrentUser(other), axum::extract::Path(first)).await.is_err());
    handlers::exclude_duplicate(State(app.clone()), CurrentUser(owner), axum::extract::Path(first)).await.unwrap();
    assert!(handlers::restore_duplicate(State(app.clone()), CurrentUser(other), axum::extract::Path(first)).await.is_err());
    // Excluding the chosen root of another alias must reparent that alias.
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let root: Uuid = sqlx::query_scalar("SELECT duplicate_of FROM transactions WHERE id=$1")
        .bind(first).fetch_one(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    handlers::exclude_duplicate(State(app.clone()), CurrentUser(owner), axum::extract::Path(root)).await.unwrap();
    let list = handlers::list_txns(State(app.clone()), CurrentUser(owner), Query(
        serde_json::from_value(serde_json::json!({})).unwrap()
    )).await.unwrap().0;
    assert_eq!(list.total, 2);
    assert!(list.data.iter().all(|r| !r.possible_duplicate));
    let last = list.data.iter().find(|r| r.description == "Repeated payment").unwrap().id;
    assert!(handlers::exclude_duplicate(State(app.clone()), CurrentUser(owner), axum::extract::Path(last)).await.is_err());
    handlers::restore_duplicate(State(app.clone()), CurrentUser(owner), axum::extract::Path(first)).await.unwrap();
    handlers::restore_duplicate(State(app.clone()), CurrentUser(owner), axum::extract::Path(root)).await.unwrap();
    let dashboard = handlers::get_dashboard(State(app), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent, 40000.0);
}

#[sqlx::test(migrations = "./migrations")]
async fn long_imported_text_does_not_break_identity_indexes(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let mut rows = vec![op("Long description", "debit", 100.0), op("Long description", "debit", 100.0)];
    for row in &mut rows {
        row.description = Some("Long merchant description ".repeat(1000));
        row.bank_ref = Some("1234567890".repeat(1000));
    }
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(rows)).await.unwrap();
    let list = handlers::list_txns(State(app), CurrentUser(owner), Query(
        serde_json::from_value(serde_json::json!({"duplicates_only":true})).unwrap()
    )).await.unwrap().0;
    assert_eq!(list.total, 2, "Unrecognized references require review, not automatic exclusion");
}

fn sms_source(received: i64, anchored: bool) -> sqlx::types::Json<khata::txns::models::SmsSource> {
    sqlx::types::Json(serde_json::from_value(serde_json::json!({
        "parser_version": 1, "received_at": received, "body_hash": "a".repeat(64),
        "event_key": if anchored { Some("a".repeat(64)) } else { None },
        "date_source": "message", "parsed_date": "2026-10-20"
    })).unwrap())
}

#[sqlx::test(migrations = "./migrations")]
async fn future_sms_dates_use_message_or_queue_dates_but_manual_entries_are_preserved(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let received = chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    let mut sms = op("Bank alert", "credit", 500.0);
    sms.notes = Some("Auto-captured via SMS (CANARA)".into());
    sms.txn_date = Some(NaiveDate::from_ymd_opt(2099, 12, 2).unwrap());
    sms.value_date = sms.txn_date;
    sms.sms_source = Some(sms_source(received, false));
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![sms])).await.unwrap();
    let mut manual = op("Future manual entry", "credit", 600.0);
    manual.txn_date = Some(NaiveDate::from_ymd_opt(2099, 12, 2).unwrap()); manual.value_date = manual.txn_date;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![manual])).await.unwrap();
    let pull = handlers::sync_pull(State(app), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(0), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0;
    let sms = pull.txns.iter().find(|t| t.description == "Bank alert").unwrap();
    assert_eq!(sms.value_date.to_string(), "2026-10-20");
    let source = sms.sms_source.as_ref().unwrap();
    assert_eq!(source.date_source, "message");
    assert_eq!(source.previous_value_date.unwrap().to_string(), "2099-12-02");
    assert_eq!(pull.txns.iter().find(|t| t.description == "Future manual entry").unwrap().value_date.to_string(), "2099-12-02");
}

#[sqlx::test(migrations = "./migrations")]
async fn rescan_reference_updates_are_saved_and_exclude_the_second_identity(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let first = bank_op("KOTAK", Some("123456789012"), "debit", 10000.0);
    let second = bank_op("KOTAK", None, "debit", 10000.0);
    let second_id = second.client_id;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![first, second])).await.unwrap();
    let mut update = bank_op("KOTAK", Some("123456789012"), "debit", 10000.0);
    update.client_id = second_id; update.base_rev = 1;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![update])).await.unwrap();
    let rows = identity_rows(&pool, owner).await;
    assert_eq!(rows.iter().filter(|r| !r.1).count(), 1);
    assert!(rows.iter().find(|r| r.0 == second_id).unwrap().1);
    let pull = handlers::sync_pull(State(app), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(0), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0;
    assert_eq!(pull.txns.iter().find(|r| r.client_id == Some(second_id)).unwrap().bank_ref.as_deref(), Some("123456789012"));
}

#[sqlx::test(migrations = "./migrations")]
async fn anchored_repeat_alerts_are_excluded_without_merging_distinct_payments_or_tenants(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let other = user(&pool).await;
    let received = chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    let make = |offset, anchored| {
        let mut row = bank_op("HDFC", None, "debit", 500.0);
        row.sms_source = Some(sms_source(received + offset, anchored)); row
    };
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![
        make(0, true), make(30_000, true), make(600_000, true), make(0, false), make(30_000, false),
    ])).await.unwrap();
    let rows = identity_rows(&pool, owner).await;
    assert_eq!(rows.len(), 5);
    assert_eq!(rows.iter().filter(|r| r.1).count(), 1);
    handlers::sync_push(State(app.clone()), CurrentUser(other), Json(vec![make(0, true)])).await.unwrap();
    assert!(!identity_rows(&pool, other).await[0].1);
    let dashboard = handlers::get_dashboard(State(app), CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent, 2000.0);
}

#[sqlx::test(migrations = "./migrations")]
async fn source_added_by_rescan_deduplicates_and_syncs_tombstones(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let received = chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    let mut first = bank_op("CANARA", None, "credit", 500.0);
    let mut second = bank_op("CANARA", None, "credit", 500.0);
    let ids = [first.client_id, second.client_id];
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![first, second])).await.unwrap();
    first = bank_op("CANARA", None, "credit", 500.0); first.client_id = ids[0]; first.base_rev = 1;
    first.sms_source = Some(sms_source(received, true));
    second = bank_op("CANARA", None, "credit", 500.0); second.client_id = ids[1]; second.base_rev = 1;
    second.sms_source = Some(sms_source(received + 60_000, true));
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![first, second])).await.unwrap();
    let pull = handlers::sync_pull(State(app), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(0), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0;
    assert_eq!(pull.txns.len(), 2);
    assert_eq!(pull.txns.iter().filter(|r| r.deleted).count(), 1);
    assert!(pull.txns.iter().all(|r| r.sms_source.as_ref().unwrap().body_hash.as_ref().unwrap().len() == 64));
}

#[sqlx::test(migrations = "./migrations")]
async fn future_sms_backfill_preserves_old_dates_and_advances_the_sync_cursor(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    sqlx::query("ALTER TABLE transactions DISABLE TRIGGER transaction_evidence").execute(&pool).await.unwrap();
    let mut sms = op("Bank alert", "credit", 500.0);
    sms.notes = Some("Auto-captured via SMS (CANARA)".into());
    sms.txn_date = Some(NaiveDate::from_ymd_opt(2099,12,2).unwrap()); sms.value_date = sms.txn_date;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![sms])).await.unwrap();
    sqlx::query("ALTER TABLE transactions ENABLE TRIGGER transaction_evidence").execute(&pool).await.unwrap();
    let old_cursor = identity_rows(&pool, owner).await[0].3;
    let migration = include_str!("../migrations/0036_sms_provenance.sql");
    let backfill = &migration[migration.find("ALTER TABLE transactions NO FORCE").unwrap()..];
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(backfill).execute(&mut *tx).await.unwrap(); tx.commit().await.unwrap();
    let pull = handlers::sync_pull(State(app), CurrentUser(owner), Query(
        khata::txns::models::SyncPullQuery { since_rev: Some(0), limit: Some(100), cursor_version: Some(2) }
    )).await.unwrap().0;
    let row = &pull.txns[0];
    assert!(row.rev > old_cursor);
    assert!(row.value_date <= chrono::Utc::now().date_naive() + chrono::Duration::days(1));
    assert_eq!(row.sms_source.as_ref().unwrap().previous_value_date.unwrap().to_string(), "2099-12-02");
    let forced: bool = sqlx::query_scalar("SELECT relforcerowsecurity FROM pg_class WHERE oid='transactions'::regclass")
        .fetch_one(&pool).await.unwrap();
    assert!(forced);
}

#[sqlx::test(migrations = "./migrations")]
async fn sms_hash_does_not_override_distinct_references_and_invalid_provenance_is_rejected(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let received = chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    let mut first = bank_op("HDFC", Some("123456789012"), "debit", 500.0);
    first.sms_source = Some(sms_source(received, true));
    let mut second = bank_op("HDFC", Some("223456789012"), "debit", 500.0);
    second.sms_source = Some(sms_source(received, true));
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![first, second])).await.unwrap();
    assert!(identity_rows(&pool, owner).await.iter().all(|r| !r.1));
    let mut invalid = bank_op("HDFC", None, "debit", 500.0);
    let mut source = sms_source(received, true);
    source.event_key = Some("unbounded invalid key".repeat(1000)); invalid.sms_source = Some(source);
    assert!(handlers::sync_push(State(app), CurrentUser(owner), Json(vec![invalid])).await.is_err());
    assert_eq!(identity_rows(&pool, owner).await.len(), 2);
}

fn overlap_op(description: &str, direction: &str, amount: f64, time: i64, role: Option<&str>, key: Option<&str>) -> SyncPushOp {
    let mut row = bank_op("HDFC", None, direction, amount);
    row.description = Some(description.into());
    let mut source = sms_source(time, false);
    source.date_source = "body".into();
    source.payment_role = role.map(str::to_string);
    source.card_account_key = key.map(str::to_string);
    row.sms_source = Some(source);
    row
}

#[sqlx::test(migrations = "./migrations")]
async fn existing_card_backfill_finishes_deferred_events_before_restoring_rls(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    sqlx::query("ALTER TABLE transactions DISABLE TRIGGER transaction_card_evidence").execute(&pool).await.unwrap();
    handlers::sync_push(State(app),CurrentUser(owner),Json(vec![
        overlap_op("CRED Club","debit",7777.0,time,None,None),
        overlap_op("Credit card payment received","credit",7777.0,time+3000,None,None),
    ])).await.unwrap();
    sqlx::query("ALTER TABLE transactions ENABLE TRIGGER transaction_card_evidence").execute(&pool).await.unwrap();
    assert_eq!(identity_rows(&pool,owner).await.iter().filter(|r|r.2).count(),1);
    let migration=include_str!("../migrations/0037_card_confirmations.sql");
    let backfill=&migration[migration.rfind("ALTER TABLE transactions NO FORCE").unwrap()..];
    let mut tx=pool.begin().await.unwrap();
    sqlx::raw_sql(backfill).execute(&mut *tx).await.unwrap();tx.commit().await.unwrap();
    let rows=identity_rows(&pool,owner).await;
    assert_eq!(rows.len(),2);assert!(rows.iter().all(|r|!r.1 && r.2));
    let forced:bool=sqlx::query_scalar("SELECT relforcerowsecurity FROM pg_class WHERE oid='transactions'::regclass")
        .fetch_one(&pool).await.unwrap();
    assert!(forced);
}

#[sqlx::test(migrations = "./migrations")]
async fn cred_triplets_reconcile_without_per_record_corrections(pool: PgPool) {
    let app = state(&pool); let owner = user(&pool).await;
    let time = chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    for amount in [7777.0,3151.0] {
        let funding = overlap_op("CRED Club", "debit", amount, time, None, None);
        let receipt = overlap_op("Credit card payment received", "credit", amount, time+3000, None, None);
        let later = overlap_op("Transaction", "credit", amount, time+86_400_000, None, None);
        handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![later,receipt,funding])).await.unwrap();
    }
    let rows=identity_rows(&pool,owner).await;
    assert_eq!(rows.len(),6);
    assert_eq!(rows.iter().filter(|r| r.1).count(),2);
    assert_eq!(rows.iter().filter(|r| !r.1 && r.2).count(),4);
    let dashboard=handlers::get_dashboard(State(app.clone()),CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent,0.0); assert_eq!(dashboard.total_earned,0.0);
    let other=user(&pool).await;
    let params=serde_json::from_value(serde_json::json!({"duplicates_only":true})).unwrap();
    assert_eq!(handlers::list_txns(State(app),CurrentUser(other),Query(params)).await.unwrap().0.total,0);
}

#[sqlx::test(migrations = "./migrations")]
async fn card_confirmation_keys_require_one_funding_and_distinct_receipt_roles(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    let key="b".repeat(64);
    let funding=overlap_op("CRED Club","debit",7777.0,time,None,None);
    let receipt=overlap_op("Credit card payment received","credit",7777.0,time+3000,Some("card_receipt"),Some(&key));
    let confirmation=overlap_op("Credit card payment received","credit",7777.0,time+86_400_000,Some("card_confirmation"),Some(&key));
    handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![confirmation,receipt,funding])).await.unwrap();
    let rows=identity_rows(&pool,owner).await;
    assert_eq!(rows.iter().filter(|r| r.1).count(),1);
    assert_eq!(rows.iter().filter(|r| !r.1 && r.2).count(),2);
    // Another card and another received-payment alert must remain separate.
    let other=overlap_op("Credit card payment received","credit",7777.0,time+5000,Some("card_receipt"),Some(&"c".repeat(64)));
    handlers::sync_push(State(app),CurrentUser(owner),Json(vec![other])).await.unwrap();
    assert_eq!(identity_rows(&pool,owner).await.iter().filter(|r| r.1).count(),1);
}

#[sqlx::test(migrations = "./migrations")]
async fn ambiguous_cred_funding_is_not_merged_even_when_it_arrives_last_in_the_batch(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();let key="b".repeat(64);
    let receipt=overlap_op("Credit card payment received","credit",7777.0,time,Some("card_receipt"),Some(&key));
    let confirmation=overlap_op("Credit card payment received","credit",7777.0,time+86_400_000,Some("card_confirmation"),Some(&key));
    let first=overlap_op("CRED Club","debit",7777.0,time,None,None);
    let second=overlap_op("CRED Club","debit",7777.0,time+3000,None,None);
    handlers::sync_push(State(app),CurrentUser(owner),Json(vec![receipt,confirmation,first,second])).await.unwrap();
    let rows=identity_rows(&pool,owner).await;
    assert!(rows.iter().all(|r| !r.1));
    assert_eq!(rows.iter().filter(|r| r.2).count(),2,"Only explicit card receipts are transfers");
}

#[sqlx::test(migrations = "./migrations")]
async fn cred_purchases_distant_receipts_and_manual_transfer_choices_are_preserved(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    let funding=overlap_op("CRED Club","debit",500.0,time,None,None);let funding_id=funding.client_id;
    let distant=overlap_op("Credit card payment received","credit",500.0,time+86_400_000,None,None);
    handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![funding,distant])).await.unwrap();
    assert!(!identity_rows(&pool,owner).await.iter().find(|r| r.0==funding_id).unwrap().2);
    let query=serde_json::from_value(serde_json::json!({})).unwrap();
    let listed=handlers::list_txns(State(app.clone()),CurrentUser(owner),Query(query)).await.unwrap().0;
    let id=listed.data.iter().find(|r| r.client_id==Some(funding_id)).unwrap().id;
    handlers::toggle_transfer(State(app.clone()),CurrentUser(owner),axum::extract::Path(id),Json(handlers::ToggleTransferReq{is_transfer:false})).await.unwrap();
    let receipt=overlap_op("Credit card payment received","credit",500.0,time+3000,None,None);
    handlers::sync_push(State(app),CurrentUser(owner),Json(vec![receipt])).await.unwrap();
    assert!(!identity_rows(&pool,owner).await.iter().find(|r| r.0==funding_id).unwrap().2);
}

#[sqlx::test(migrations = "./migrations")]
async fn transaction_pages_have_a_stable_order_with_identical_sort_and_creation_values(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let operations=(0..125).map(|_| bank_op("HDFC",None,"debit",1.0)).collect();
    handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(operations)).await.unwrap();
    for sort in ["date","amount","description","category"] {
        let mut ids=Vec::new();
        for page in 1..=3 {
            let params=serde_json::from_value(serde_json::json!({"page":page,"per_page":50,"sort_by":sort})).unwrap();
            let result=handlers::list_txns(State(app.clone()),CurrentUser(owner),Query(params)).await.unwrap().0;
            assert_eq!(result.total,125);ids.extend(result.data.into_iter().map(|r|r.id));
        }
        ids.sort();ids.dedup();assert_eq!(ids.len(),125,"{sort} must neither repeat nor skip tied records");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn general_reconciliation_handles_many_banks_direct_bills_and_separate_uploads(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    for (i,bank) in ["HDFC","KOTAK","AXIS","CANARA"].iter().enumerate() {
        let amount=701.0+i as f64;
        let mut funding=overlap_op(if i%2==0 {"CRED Club"} else {"Credit card bill payment"},"debit",amount,time,None,None);
        funding.bank=Some("HDFC".into());
        let mut receipt=overlap_op("Credit card payment received","credit",amount,time+3000,None,None);
        receipt.bank=Some((*bank).into());
        let mut later=overlap_op("Transaction","credit",amount,time+86_400_000,None,None);
        later.bank=Some((*bank).into());later.sms_source.as_mut().unwrap().parsed_date=NaiveDate::from_ymd_opt(2026,10,21).unwrap();
        if i%2==0 {
            handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![funding,receipt])).await.unwrap();
            handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![later])).await.unwrap();
        } else {
            handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![later])).await.unwrap();
            handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![receipt,funding])).await.unwrap();
        }
    }
    let rows=identity_rows(&pool,owner).await;
    assert_eq!(rows.len(),12);assert_eq!(rows.iter().filter(|r|r.1).count(),4);
    assert_eq!(rows.iter().filter(|r|!r.1 && r.2).count(),8);
    let dashboard=handlers::get_dashboard(State(app.clone()),CurrentUser(owner)).await.unwrap().0;
    assert_eq!(dashboard.total_spent,0.0);assert_eq!(dashboard.total_earned,0.0);
    let pull=handlers::sync_pull(State(app),CurrentUser(owner),Query(
        khata::txns::models::SyncPullQuery{since_rev:Some(0),limit:Some(100),cursor_version:Some(2)})).await.unwrap().0;
    assert_eq!(pull.txns.len(),12);assert_eq!(pull.txns.iter().filter(|r|r.deleted).count(),4);
}

#[sqlx::test(migrations = "./migrations")]
async fn missing_funding_needs_card_identity_and_legacy_candidates_remain_reviewable(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    for keyed in [false,true] {
        let amount=if keyed {702.0} else {701.0};let key="d".repeat(64);
        let receipt=overlap_op("Credit card payment received","credit",amount,time,
            keyed.then_some("card_receipt"),keyed.then_some(key.as_str()));
        let mut later=overlap_op(if keyed {"Credit card payment received"} else {"Transaction"},"credit",amount,time+86_400_000,
            keyed.then_some("card_confirmation"),keyed.then_some(key.as_str()));
        later.sms_source.as_mut().unwrap().parsed_date=NaiveDate::from_ymd_opt(2026,10,21).unwrap();
        handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![receipt,later])).await.unwrap();
    }
    let rows=identity_rows(&pool,owner).await;
    assert_eq!(rows.len(),4);assert_eq!(rows.iter().filter(|r|r.1).count(),1);
    let params=serde_json::from_value(serde_json::json!({"duplicates_only":true})).unwrap();
    let review=handlers::list_txns(State(app.clone()),CurrentUser(owner),Query(params)).await.unwrap().0;
    assert_eq!(review.total,1);assert_eq!(review.data[0].description,"Transaction");
    let other=user(&pool).await;
    assert!(handlers::exclude_duplicate(State(app.clone()),CurrentUser(other),axum::extract::Path(review.data[0].id)).await.is_err());
    handlers::exclude_duplicate(State(app),CurrentUser(owner),axum::extract::Path(review.data[0].id)).await.unwrap();
    assert_eq!(identity_rows(&pool,owner).await.iter().filter(|r|r.1).count(),2);
}

#[sqlx::test(migrations = "./migrations")]
async fn restoration_stays_authoritative_when_later_rescans_reconcile_the_group(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    let funding=overlap_op("CRED Club","debit",701.0,time,None,None);
    let receipt=overlap_op("Credit card payment received","credit",701.0,time+3000,None,None);
    let later=overlap_op("Transaction","credit",701.0,time+86_400_000,None,None);let id=later.client_id;
    handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![funding,receipt,later])).await.unwrap();
    let pull=handlers::sync_pull(State(app.clone()),CurrentUser(owner),Query(
        khata::txns::models::SyncPullQuery{since_rev:Some(0),limit:Some(100),cursor_version:Some(2)})).await.unwrap().0;
    let excluded=pull.txns.iter().find(|r|r.client_id==Some(id)).unwrap();assert!(excluded.deleted);
    handlers::restore_duplicate(State(app.clone()),CurrentUser(owner),axum::extract::Path(excluded.id)).await.unwrap();
    let pull=handlers::sync_pull(State(app.clone()),CurrentUser(owner),Query(
        khata::txns::models::SyncPullQuery{since_rev:Some(0),limit:Some(100),cursor_version:Some(2)})).await.unwrap().0;
    let restored=pull.txns.iter().find(|r|r.client_id==Some(id)).unwrap();
    let mut rescan=overlap_op("Transaction","credit",701.0,time+86_400_000,None,None);
    rescan.client_id=id;rescan.base_rev=restored.rev;rescan.sms_source.as_mut().unwrap().body_hash=Some("f".repeat(64));
    handlers::sync_push(State(app),CurrentUser(owner),Json(vec![rescan])).await.unwrap();
    assert!(identity_rows(&pool,owner).await.iter().all(|r|!r.1));
}

#[sqlx::test(migrations = "./migrations")]
async fn ambiguous_generic_credits_different_cards_references_and_income_stay_separate(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    for scenario in 0..4 {
        let amount=701.0+scenario as f64;
        let funding=overlap_op("CRED Club","debit",amount,time,None,None);
        let mut receipt=overlap_op("Credit card payment received","credit",amount,time+3000,None,None);
        let mut later=overlap_op(if scenario==3 {"Salary"} else {"Transaction"},"credit",amount,time+86_400_000,None,None);
        if scenario==1 {
            receipt.sms_source.as_mut().unwrap().card_account_key=Some("b".repeat(64));
            later.sms_source.as_mut().unwrap().card_account_key=Some("c".repeat(64));
        }
        if scenario==2 {receipt.bank_ref=Some("123456789012".into());later.bank_ref=Some("223456789012".into());}
        let mut ops=vec![funding,receipt,later];
        if scenario==0 {ops.push(overlap_op("Transaction","credit",amount,time+90_000_000,None,None));}
        handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(ops)).await.unwrap();
    }
    assert!(identity_rows(&pool,owner).await.iter().all(|r|!r.1));
}

#[sqlx::test(migrations = "./migrations")]
async fn general_backfill_reconciles_existing_groups_and_restores_forced_rls(pool: PgPool) {
    let app=state(&pool);let owner=user(&pool).await;
    let time=chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z").unwrap().timestamp_millis();
    sqlx::query("ALTER TABLE transactions DISABLE TRIGGER transaction_card_evidence").execute(&pool).await.unwrap();
    for amount in [701.0,702.0,703.0] {
        handlers::sync_push(State(app.clone()),CurrentUser(owner),Json(vec![
            overlap_op("CRED Club","debit",amount,time,None,None),
            overlap_op("Credit card payment received","credit",amount,time+3000,None,None),
            overlap_op("Transaction","credit",amount,time+86_400_000,None,None),
        ])).await.unwrap();
    }
    sqlx::query("ALTER TABLE transactions ENABLE TRIGGER transaction_card_evidence").execute(&pool).await.unwrap();
    assert!(identity_rows(&pool,owner).await.iter().all(|r|!r.1));
    let migration=include_str!("../migrations/0038_general_card_reconciliation.sql");
    let backfill=&migration[migration.rfind("ALTER TABLE transactions NO FORCE").unwrap()..];
    let mut tx=pool.begin().await.unwrap();sqlx::raw_sql(backfill).execute(&mut *tx).await.unwrap();tx.commit().await.unwrap();
    let rows=identity_rows(&pool,owner).await;assert_eq!(rows.len(),9);assert_eq!(rows.iter().filter(|r|r.1).count(),3);
    let forced:bool=sqlx::query_scalar("SELECT relforcerowsecurity FROM pg_class WHERE oid='transactions'::regclass").fetch_one(&pool).await.unwrap();
    assert!(forced);
}
