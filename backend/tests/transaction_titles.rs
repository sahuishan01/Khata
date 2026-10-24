use axum::{extract::{Path, Query, State}, Json};
use khata::{
    auth::middleware::CurrentUser,
    txns::{handlers, models::{SyncPushOp, SyncPullQuery, UpdateDescriptionReq}},
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
async fn title_edit_preserves_identity_source_and_money_and_reaches_device_sync(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let operation = op("Transaction", "debit", 123.45);
    let client = operation.client_id;
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![operation])).await.unwrap();
    let pulled = handlers::sync_pull(State(app.clone()), CurrentUser(owner), Query(SyncPullQuery { since_rev: Some(0), limit: Some(50), cursor_version: Some(2) })).await.unwrap().0;
    let old = &pulled.txns[0];
    let before = snapshot(&pool, owner, old.id).await;
    handlers::update_description(State(app.clone()), CurrentUser(owner), Path(old.id), Json(UpdateDescriptionReq {
        description: "  Groceries & café <receipt>  ".into(),
    })).await.unwrap();
    let after = snapshot(&pool, owner, old.id).await;
    assert_eq!(after["description"], "Groceries & café <receipt>");
    assert_eq!(after["rev"].as_i64().unwrap(), old.rev + 1);
    for field in ["description", "rev", "version", "updated_at", "sync_rev"] {
        // Only display text and change-tracking fields may change.
        assert!(before.get(field).is_some());
    }
    let mut original = before.as_object().unwrap().clone();
    let mut changed = after.as_object().unwrap().clone();
    for field in ["description", "rev", "version", "updated_at", "sync_rev"] {
        original.remove(field); changed.remove(field);
    }
    assert_eq!(original, changed);
    let delta = handlers::sync_pull(State(app.clone()), CurrentUser(owner), Query(SyncPullQuery {
        since_rev: Some(pulled.new_rev), limit: Some(50), cursor_version: Some(2),
    })).await.unwrap().0;
    assert_eq!(delta.txns.len(), 1);
    assert_eq!(delta.txns[0].client_id, Some(client));
    assert_eq!(delta.txns[0].description, "Groceries & café <receipt>");
    // A stale device must not restore the automatically parsed title.
    let mut stale = op("Transaction", "debit", 123.45);
    stale.client_id = client; stale.base_rev = old.rev;
    let reply = handlers::sync_push(State(app), CurrentUser(owner), Json(vec![stale])).await.unwrap().0;
    assert!(reply.accepted.is_empty()); assert_eq!(reply.conflicts.len(), 1);
}

async fn snapshot(pool: &PgPool, owner: Uuid, id: Uuid) -> serde_json::Value {
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let value = sqlx::query_scalar("SELECT to_jsonb(t) FROM transactions t WHERE id=$1 AND user_id=$2")
        .bind(id).bind(owner).fetch_one(&mut *tx).await.unwrap();
    tx.commit().await.unwrap(); value
}

#[sqlx::test(migrations = "./migrations")]
async fn title_edit_rejects_other_users_missing_deleted_and_invalid_titles(pool: PgPool) {
    let app = state(&pool); let owner = user(&pool).await; let other = user(&pool).await;
    let operation = op("Original", "credit", 500.0);
    handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![operation])).await.unwrap();
    let pulled = handlers::sync_pull(State(app.clone()), CurrentUser(owner), Query(SyncPullQuery { since_rev: Some(0), limit: Some(50), cursor_version: Some(2) })).await.unwrap().0;
    let id = pulled.txns[0].id; let original = snapshot(&pool, owner, id).await;
    for (actor, target, text, bad_request) in [
        (other, id, "Other user's edit".to_string(), false),
        (owner, Uuid::new_v4(), "Missing".to_string(), false),
        (owner, id, " \n\t ".to_string(), true),
        (owner, id, "a".repeat(501), true),
    ] {
        let result = handlers::update_description(State(app.clone()), CurrentUser(actor), Path(target), Json(UpdateDescriptionReq { description: text })).await;
        if bad_request { assert!(matches!(result, Err(khata::error::AppError::BadRequest(_)))); }
        else { assert!(matches!(result, Err(khata::error::AppError::NotFound))); }
        assert_eq!(original, snapshot(&pool, owner, id).await);
    }
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    sqlx::query("UPDATE transactions SET deleted=true WHERE id=$1").bind(id).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    assert!(matches!(handlers::update_description(State(app), CurrentUser(owner), Path(id), Json(UpdateDescriptionReq { description: "Restore by rename".into() })).await, Err(khata::error::AppError::NotFound)));
}

#[test]
fn title_edit_payload_cannot_include_financial_changes() {
    assert!(serde_json::from_value::<UpdateDescriptionReq>(serde_json::json!({"description":"Title", "amount":0})).is_err());
    assert!(serde_json::from_value::<UpdateDescriptionReq>(serde_json::json!({"description":null})).is_err());
}
