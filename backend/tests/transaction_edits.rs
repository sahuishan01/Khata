use axum::{
    extract::{Path, Query, State},
    Json,
};
use khata::{
    auth::middleware::CurrentUser,
    txns::{
        handlers,
        models::{SyncPullQuery, SyncPushOp, UpdateTxnReq},
    },
    AppState,
};
use sha2::Digest;
use sqlx::PgPool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

fn state(pool: &PgPool) -> AppState {
    AppState {
        db: pool.clone(),
        db_ro: pool.clone(),
        config: Arc::new(khata::config::Config {
            database_url: String::new(),
            ro_database_url: String::new(),
            jwt_secret: "test-secret-at-least-32-characters".into(),
            claude_bin: "claude".into(),
            bind_addr: "127.0.0.1:0".into(),
            cors_origins: vec![],
            cookie_secure: true,
            allow_remote_setup: false,
            email_sync_poll_secs: 0,
            email_sync_max_messages: 200,
            email_sync_max_attach_bytes: 1024,
            pdfium_lib_path: None,
        }),
        chat_ratelimit: Arc::new(Mutex::new(HashMap::new())),
        login_attempts: Arc::new(Mutex::new(HashMap::new())),
    }
}

async fn user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users (email,password_hash) VALUES ($1,'test') RETURNING id")
        .bind(format!("{}@example.test", Uuid::new_v4()))
        .fetch_one(pool)
        .await
        .unwrap()
}

fn op(description: &str, direction: &str, amount: f64) -> SyncPushOp {
    serde_json::from_value(serde_json::json!({
        "client_id": Uuid::new_v4(), "base_rev": 0, "description": description,
        "direction": direction, "amount": amount, "txn_date": "2026-10-20",
        "value_date": "2026-10-20", "bank": "HDFC", "category": "Uncategorized",
        "is_transfer": false,
    }))
    .unwrap()
}

fn edit(row: &khata::txns::models::TxnRow) -> UpdateTxnReq {
    serde_json::from_value(serde_json::json!({
        "base_rev":row.rev, "txn_date":"2026-10-22", "value_date":"2026-10-23",
        "description":"  Exact SMS\nwith café and <markup>  ", "amount":987.65,
        "direction":"credit", "category":"Refunds", "bank":"AXIS", "bank_ref":"",
        "balance": -123.45, "is_transfer":false, "notes":"My corrections",
    }))
    .unwrap()
}

async fn snapshot(pool: &PgPool, owner: Uuid, id: Uuid) -> serde_json::Value {
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    let value =
        sqlx::query_scalar("SELECT to_jsonb(t) FROM transactions t WHERE id=$1 AND user_id=$2")
            .bind(id)
            .bind(owner)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    value
}

#[sqlx::test(migrations = "./migrations")]
async fn full_edit_is_atomic_preserves_identity_and_survives_sms_rules_and_sync(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let mut original = op("Original SMS\nexact text", "debit", 123.45);
    original.sms_source = Some(sqlx::types::Json(
        serde_json::from_value(serde_json::json!({
            "parser_version":1, "date_source":"body", "parsed_date":"2026-10-20",
            "received_at":1792477800000_i64, "body_hash":"a".repeat(64), "parsed_payee":"CRED CLUB",
        }))
        .unwrap(),
    ));
    original.notes = Some("Auto-captured via SMS (HDFC)".into());
    let client = original.client_id;
    let _ = handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![original]))
        .await
        .unwrap();
    let pulled = handlers::sync_pull(
        State(app.clone()),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0;
    let old = &pulled.txns[0];
    let before = snapshot(&pool, owner, old.id).await;
    let mut request = edit(old);
    request.notes = "Auto-captured via SMS (HDFC)".into();
    let saved = handlers::update_txn(
        State(app.clone()),
        CurrentUser(owner),
        Path(old.id),
        Json(request),
    )
    .await
    .unwrap()
    .0;
    assert_eq!(saved.description, "  Exact SMS\nwith café and <markup>  ");
    assert_eq!(saved.amount, 987.65);
    assert_eq!(saved.direction, "credit");
    assert_eq!(saved.txn_date.to_string(), "2026-10-22"); // after receipt day: manual choice wins
    assert_eq!(saved.value_date.to_string(), "2026-10-23");
    assert_eq!(saved.bank, "AXIS");
    assert_eq!(saved.category, "Refunds");
    assert_eq!(saved.bank_ref.as_deref(), Some(""));
    assert_eq!(saved.balance, Some(-123.45));
    assert!(!saved.is_transfer);
    assert_eq!(saved.rev, old.rev + 1);
    assert_eq!(saved.sms_source.as_ref().unwrap().user_edited, Some(true));
    let after = snapshot(&pool, owner, old.id).await;
    for field in [
        "id",
        "client_id",
        "user_id",
        "statement_id",
        "raw_description",
        "fingerprint",
        "created_at",
        "duplicate_of",
        "deleted",
    ] {
        assert_eq!(before[field], after[field], "{field}");
    }
    let delta = handlers::sync_pull(
        State(app.clone()),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(pulled.new_rev),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0;
    assert_eq!(delta.txns.len(), 1);
    assert_eq!(delta.txns[0].client_id, Some(client));
    assert_eq!(delta.txns[0].balance, Some(-123.45));
    // A second edit can reverse a previous manual transfer choice and clear a balance.
    let mut next = edit(&saved);
    next.is_transfer = true;
    next.balance = None;
    let saved = handlers::update_txn(
        State(app.clone()),
        CurrentUser(owner),
        Path(old.id),
        Json(next),
    )
    .await
    .unwrap()
    .0;
    assert!(saved.is_transfer);
    assert_eq!(saved.balance, None);
    let mut next = edit(&saved);
    next.is_transfer = false;
    let saved = handlers::update_txn(
        State(app.clone()),
        CurrentUser(owner),
        Path(old.id),
        Json(next),
    )
    .await
    .unwrap()
    .0;
    assert!(!saved.is_transfer);
    let mut stale = op("Original SMS\nexact text", "debit", 123.45);
    stale.client_id = client;
    stale.base_rev = old.rev;
    let result = handlers::sync_push(State(app), CurrentUser(owner), Json(vec![stale]))
        .await
        .unwrap()
        .0;
    assert!(result.accepted.is_empty());
    assert_eq!(result.conflicts.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn full_edit_rejects_invalid_values_other_owners_stale_and_excluded_rows(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let other = user(&pool).await;
    let _ = handlers::sync_push(
        State(app.clone()),
        CurrentUser(owner),
        Json(vec![op("Original", "debit", 10.0)]),
    )
    .await
    .unwrap();
    let pulled = handlers::sync_pull(
        State(app.clone()),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0;
    let old = &pulled.txns[0];
    let before = snapshot(&pool, owner, old.id).await;
    for (actor, id) in [(other, old.id), (owner, Uuid::new_v4())] {
        assert!(matches!(
            handlers::update_txn(
                State(app.clone()),
                CurrentUser(actor),
                Path(id),
                Json(edit(old))
            )
            .await,
            Err(khata::error::AppError::NotFound)
        ));
    }
    let mut stale = edit(old);
    stale.base_rev += 1;
    assert!(matches!(
        handlers::update_txn(
            State(app.clone()),
            CurrentUser(owner),
            Path(old.id),
            Json(stale)
        )
        .await,
        Err(khata::error::AppError::Conflict(_))
    ));
    for invalid in 0..11 {
        let mut req = edit(old);
        match invalid {
            0 => req.description = " \n ".into(),
            1 => req.description = "x".repeat(16001),
            2 => req.amount = -1.0,
            3 => req.amount = f64::NAN,
            4 => req.amount = 10.001,
            5 => req.balance = Some(f64::INFINITY),
            6 => req.amount = 1_000_000_000_000.0,
            7 => req.bank = " ".into(),
            8 => req.direction = "refund".into(),
            9 => req.category = "x".repeat(101),
            _ => req.notes = "x".repeat(10001),
        }
        assert!(
            matches!(
                handlers::update_txn(
                    State(app.clone()),
                    CurrentUser(owner),
                    Path(old.id),
                    Json(req)
                )
                .await,
                Err(khata::error::AppError::BadRequest(_))
            ),
            "case {invalid}"
        );
        assert_eq!(before, snapshot(&pool, owner, old.id).await);
    }
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, owner).await.unwrap();
    sqlx::query("UPDATE transactions SET deleted=true WHERE id=$1")
        .bind(old.id)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(matches!(
        handlers::update_txn(
            State(app),
            CurrentUser(owner),
            Path(old.id),
            Json(edit(old))
        )
        .await,
        Err(khata::error::AppError::NotFound)
    ));
}

#[test]
fn full_edit_rejects_unknown_fields_and_invalid_dates() {
    let mut value = serde_json::json!({"base_rev":1,"txn_date":"2026-10-20","value_date":"2026-10-20",
        "description":"SMS","amount":10,"direction":"debit","category":"Food","bank":"HDFC",
        "balance":null,"is_transfer":false,"notes":"","user_id":Uuid::new_v4()});
    assert!(serde_json::from_value::<UpdateTxnReq>(value.clone()).is_err());
    value.as_object_mut().unwrap().remove("user_id");
    value["txn_date"] = serde_json::json!("2026-02-30");
    assert!(serde_json::from_value::<UpdateTxnReq>(value).is_err());
}

#[sqlx::test(migrations = "./migrations")]
async fn non_sms_transactions_can_reverse_existing_manual_transfer_choices(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let _ = handlers::sync_push(
        State(app.clone()),
        CurrentUser(owner),
        Json(vec![op("Manual payment", "debit", 10.0)]),
    )
    .await
    .unwrap();
    let mut row = handlers::sync_pull(
        State(app.clone()),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0
    .txns
    .remove(0);
    assert!(row.sms_source.is_none());
    for transfer in [true, false, true, false] {
        let mut request = edit(&row);
        request.is_transfer = transfer;
        row = handlers::update_txn(
            State(app.clone()),
            CurrentUser(owner),
            Path(row.id),
            Json(request),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(row.is_transfer, transfer);
        assert!(row.sms_source.is_none());
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn inline_edits_invalidate_an_open_full_transaction_editor(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let _ = handlers::sync_push(
        State(app.clone()),
        CurrentUser(owner),
        Json(vec![op("Original", "debit", 10.0)]),
    )
    .await
    .unwrap();
    let original = handlers::sync_pull(
        State(app.clone()),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0
    .txns
    .remove(0);
    let _ = handlers::update_notes(
        State(app.clone()),
        CurrentUser(owner),
        Path(original.id),
        Json(khata::txns::models::UpdateNotesReq {
            notes: "Changed on another device".into(),
        }),
    )
    .await
    .unwrap();
    let after_notes = handlers::get_txn(State(app.clone()), CurrentUser(owner), Path(original.id))
        .await
        .unwrap()
        .0;
    assert_eq!(after_notes.rev, original.rev + 1);
    assert!(matches!(
        handlers::update_txn(
            State(app.clone()),
            CurrentUser(owner),
            Path(original.id),
            Json(edit(&original))
        )
        .await,
        Err(khata::error::AppError::Conflict(_))
    ));
    let _ = handlers::update_category(
        State(app.clone()),
        CurrentUser(owner),
        Path(original.id),
        Json(
            serde_json::from_value(serde_json::json!({"category":"Food", "scope":"single"}))
                .unwrap(),
        ),
    )
    .await
    .unwrap();
    let after_category =
        handlers::get_txn(State(app.clone()), CurrentUser(owner), Path(original.id))
            .await
            .unwrap()
            .0;
    assert_eq!(after_category.rev, after_notes.rev + 1);
    assert!(matches!(
        handlers::update_txn(
            State(app),
            CurrentUser(owner),
            Path(original.id),
            Json(edit(&after_notes))
        )
        .await,
        Err(khata::error::AppError::Conflict(_))
    ));
    assert_eq!(after_category.notes, "Changed on another device");
    assert_eq!(after_category.category, "Food");
}

#[sqlx::test(migrations = "./migrations")]
async fn older_sms_scanners_cannot_replace_full_edits_and_unchanged_replays_still_work(
    pool: PgPool,
) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let mut imported = op("Original SMS", "debit", 500.0);
    imported.notes = Some("Auto-captured via SMS (HDFC)".into());
    imported.sms_source = Some(sqlx::types::Json(
        serde_json::from_value(serde_json::json!({
            "parser_version":1,"date_source":"body","parsed_date":"2026-10-20",
            "body_hash":"a".repeat(64),"received_at":1792477800000_i64,"parsed_payee":"SHOP"
        }))
        .unwrap(),
    ));
    let client = imported.client_id;
    let _ = handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![imported]))
        .await
        .unwrap();
    let row = handlers::sync_pull(
        State(app.clone()),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0
    .txns
    .remove(0);
    let mut request = edit(&row);
    request.notes = row.notes.clone();
    let saved = handlers::update_txn(
        State(app.clone()),
        CurrentUser(owner),
        Path(row.id),
        Json(request),
    )
    .await
    .unwrap()
    .0;
    let mut old_metadata = saved.sms_source.as_ref().unwrap().clone();
    old_metadata.user_edited = None;
    old_metadata.parsed_payee = None;
    // An older app can pull the latest revision but drop unknown metadata keys.
    let mut legacy = op(&saved.description, &saved.direction, saved.amount);
    legacy.client_id = client;
    legacy.base_rev = saved.rev;
    legacy.bank = Some(saved.bank.clone());
    legacy.category = Some(saved.category.clone());
    legacy.notes = Some(saved.notes.clone());
    legacy.is_transfer = Some(saved.is_transfer);
    legacy.sms_source = Some(old_metadata);
    legacy.bank_ref = Some("123456789012".into());
    let before = snapshot(&pool, owner, saved.id).await;
    let result = handlers::sync_push(
        State(app.clone()),
        CurrentUser(owner),
        Json(vec![serde_json::from_value(
            serde_json::to_value(&legacy).unwrap(),
        )
        .unwrap()]),
    )
    .await
    .unwrap()
    .0;
    assert!(result.accepted.is_empty());
    assert_eq!(result.conflicts.len(), 1);
    assert_eq!(before, snapshot(&pool, owner, saved.id).await);
    // Omitting additive metadata alone must not revive the old lost-ACK conflict.
    legacy.bank_ref = saved.bank_ref.clone();
    legacy.txn_date = Some(saved.txn_date);
    legacy.value_date = Some(saved.value_date);
    let resolution = serde_json::from_value(serde_json::to_value(&legacy).unwrap()).unwrap();
    let result = handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(vec![legacy]))
        .await
        .unwrap()
        .0;
    assert_eq!(result.accepted.len(), 1);
    assert!(result.conflicts.is_empty());
    let after = snapshot(&pool, owner, saved.id).await;
    assert_eq!(after["sms_source"]["user_edited"], true);
    assert_eq!(after["sms_source"]["parsed_payee"], "SHOP");
    assert_eq!(after["rev"], before["rev"]);
    let resolved = handlers::sync_resolve(State(app), CurrentUser(owner), Json(vec![resolution]))
        .await
        .unwrap()
        .0;
    assert_eq!(resolved.len(), 1);
    let after_resolution = snapshot(&pool, owner, saved.id).await;
    assert_eq!(after_resolution["sms_source"]["user_edited"], true);
    assert_eq!(after_resolution["sms_source"]["parsed_payee"], "SHOP");
}

#[sqlx::test(migrations = "./migrations")]
async fn exact_sms_descriptions_retain_cred_card_reconciliation(pool: PgPool) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let time = chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z")
        .unwrap()
        .timestamp_millis();
    let mut operations = vec![];
    for (text, direction, offset, payee, role) in [
        (
            "INR 701 debited from HDFC A/c XX1234 to CRED Club.",
            "debit",
            0,
            "CRED Club",
            None,
        ),
        (
            "Payment of INR 701 received towards HDFC credit card XX4321.",
            "credit",
            3000,
            "Transaction",
            Some("card_receipt"),
        ),
        (
            "INR 701 credited to HDFC credit card XX4321.",
            "credit",
            86_400_000,
            "Transaction",
            Some("card_confirmation"),
        ),
    ] {
        let mut operation = op(text, direction, 701.0);
        operation.is_transfer = Some(direction == "credit");
        operation.sms_source=Some(sqlx::types::Json(serde_json::from_value(serde_json::json!({
            "parser_version":1,"date_source":"body","parsed_date":"2026-10-20","received_at":time+offset,
            "parsed_payee":payee,"body_hash":format!("{:x}", sha2::Sha256::digest(text.to_lowercase().as_bytes())),
            "payment_role":role,"card_account_key":role.map(|_| "b".repeat(64)),
        })).unwrap()));
        operations.push(operation);
    }
    let result = handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(operations))
        .await
        .unwrap()
        .0;
    assert_eq!(result.accepted.len(), 3);
    let rows = handlers::sync_pull(
        State(app),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0
    .txns;
    assert_eq!(rows.iter().filter(|row| row.deleted).count(), 1);
    assert!(
        rows.iter()
            .find(|row| row.direction == "debit")
            .unwrap()
            .is_transfer
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn generic_confirmations_keep_legacy_proof_requirements_with_exact_sms_descriptions(
    pool: PgPool,
) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let time = chrono::DateTime::parse_from_rfc3339("2026-10-20T04:00:00Z")
        .unwrap()
        .timestamp_millis();
    let mut operations = vec![];
    let mut confirmations = vec![];
    for (amount, role, transfer) in [
        (701.0, None, false),
        (702.0, Some("card_confirmation"), true),
        (703.0, None, false),
        (704.0, None, false),
    ] {
        for (index, text, direction, payee, payment_role, is_transfer) in [
            (
                0,
                "INR debited from HDFC A/c XX1234 to CRED Club.",
                "debit",
                "CRED Club",
                None,
                false,
            ),
            (
                1,
                "Payment received towards HDFC credit card.",
                "credit",
                "Transaction",
                Some("card_receipt"),
                true,
            ),
            (
                2,
                "INR credited to HDFC account on 20/10/26.",
                "credit",
                "Transaction",
                role,
                transfer,
            ),
        ] {
            let original_message = format!("{text} Amount {amount}");
            let description = if amount == 703.0 && index == 2 {
                "My salary"
            } else if amount == 704.0 && index == 0 {
                "My store purchase"
            } else {
                &original_message
            };
            let mut operation = op(description, direction, amount);
            operation.is_transfer = Some(is_transfer);
            operation.sms_source = Some(sqlx::types::Json(
                serde_json::from_value(serde_json::json!({
                    "parser_version":1,"date_source":"body","parsed_date":"2026-10-20",
                    "received_at":time + if index == 2 { 86_400_000 } else { index * 3000 },
                    "parsed_payee":payee,"body_hash":format!("{:x}", sha2::Sha256::digest(original_message.to_lowercase().as_bytes())),"payment_role":payment_role,
                }))
                .unwrap(),
            ));
            if index == 2 {
                confirmations.push(operation.client_id);
            }
            operations.push(operation);
        }
    }
    let _ = handlers::sync_push(State(app.clone()), CurrentUser(owner), Json(operations))
        .await
        .unwrap();
    let rows = handlers::sync_pull(
        State(app),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0
    .txns;
    assert!(
        rows.iter()
            .find(|row| row.client_id == Some(confirmations[0]))
            .unwrap()
            .deleted
    );
    // Parsed payee alone must not loosen the stricter policy for known card confirmations without a card key.
    assert!(
        !rows
            .iter()
            .find(|row| row.client_id == Some(confirmations[1]))
            .unwrap()
            .deleted
    );
    assert!(
        !rows
            .iter()
            .find(|row| row.client_id == Some(confirmations[2]))
            .unwrap()
            .deleted
    );
    assert!(
        !rows
            .iter()
            .find(|row| row.client_id == Some(confirmations[3]))
            .unwrap()
            .deleted
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn editing_legacy_sms_creates_rescan_protection_and_keeps_matching_reference_visible(
    pool: PgPool,
) {
    let app = state(&pool);
    let owner = user(&pool).await;
    let mut first = op("First", "debit", 701.0);
    first.bank_ref = Some("123456789012".into());
    let mut legacy = op("Legacy SMS", "debit", 500.0);
    legacy.notes = Some("Auto-captured via SMS (HDFC)".into());
    let _ = handlers::sync_push(
        State(app.clone()),
        CurrentUser(owner),
        Json(vec![first, legacy]),
    )
    .await
    .unwrap();
    let rows = handlers::sync_pull(
        State(app.clone()),
        CurrentUser(owner),
        Query(SyncPullQuery {
            since_rev: Some(0),
            limit: Some(50),
            cursor_version: Some(2),
        }),
    )
    .await
    .unwrap()
    .0
    .txns;
    let row = rows
        .iter()
        .find(|row| row.description == "Legacy SMS")
        .unwrap();
    let mut request = edit(row);
    request.bank = "HDFC".into();
    request.bank_ref = Some("123456789012".into());
    request.amount = 701.0;
    request.direction = "debit".into();
    request.value_date = "2026-10-20".parse().unwrap();
    let saved = handlers::update_txn(State(app), CurrentUser(owner), Path(row.id), Json(request))
        .await
        .unwrap()
        .0;
    assert!(!saved.deleted);
    assert_eq!(saved.bank_ref.as_deref(), Some("123456789012"));
    let source = saved.sms_source.unwrap();
    assert_eq!(source.user_edited, Some(true));
    source.validate().unwrap();
}
