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
