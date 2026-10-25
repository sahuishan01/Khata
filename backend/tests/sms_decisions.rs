use axum::{routing::post, Json, Router};
use khata::{
    sms::{classify_at, DecisionRequest},
    AppState,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
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

async fn server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (url, handle)
}
fn jwt(owner: Uuid) -> String {
    jsonwebtoken::encode(&jsonwebtoken::Header::default(),&json!({"sub":owner,"exp":chrono::Utc::now().timestamp()+3600,"ver":0,"iss":"khata","aud":"khata-api"}),
        &jsonwebtoken::EncodingKey::from_secret(b"test-secret-at-least-32-characters")).unwrap()
}
fn http() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

#[tokio::test]
async fn gateway_binds_decisions_to_input_and_rejects_invalid_or_unavailable_model() {
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let (url,task)=server(Router::new().route("/decide",post(move |Json(request):Json<Value>| {let seen=seen.clone();async move {
        seen.fetch_add(1,Ordering::SeqCst);
        let body=request["body"].as_str().unwrap();
        Json(json!({"status":"decision","is_transaction":true,"is_debit":true,
            "transaction_probability":0.95,"debit_probability":0.8,
            "source_hash":if body=="wrong hash" {"invalid".into()} else {format!("{:x}",Sha256::digest(body.as_bytes()))},
            "model_version":"laya:test:sms-choice-v1"}))
    }}))).await;
    let url = format!("{url}/decide");
    let owner = Uuid::new_v4();
    let decide = |body: &str| {
        classify_at(
            owner,
            DecisionRequest {
                owner_id: owner,
                body: body.into(),
            },
            &url,
        )
    };
    assert!(decide("INR 500 sent").await.unwrap().is_debit.unwrap());
    assert!(decide("wrong hash").await.is_err());
    assert!(decide("Your OTP 123456 for INR 500 sent").await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    task.abort();
    let unavailable = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/decide", unavailable.local_addr().unwrap());
    drop(unavailable);
    assert!(classify_at(
        owner,
        DecisionRequest {
            owner_id: owner,
            body: "INR 500 sent".into()
        },
        &endpoint
    )
    .await
    .is_err());
}

#[sqlx::test(migrations = "./migrations")]
async fn sms_endpoints_authenticate_scope_imports_and_preserve_manual_edits(pool: PgPool) {
    let app = state(&pool);
    let a = user(&pool).await;
    let b = user(&pool).await;
    let (url, task) = server(
        Router::new()
            .nest("/api/sms", khata::sms::router())
            .with_state(app.clone()),
    )
    .await;
    let client = http();
    let token = jwt(a);
    let source = json!({"parser_version":1,"received_at":1792477800000_i64,"body_hash":"a".repeat(64),"date_source":"body","parsed_date":"2026-10-20","payment_method":"IMPS","classifier_version":"laya:test:sms-choice-v1"});
    let mut operation = json!({"client_id":Uuid::new_v4(),"base_rev":0,"description":"Auto-captured via SMS (HDFC)","notes":"IMPS INR 500 sent from HDFC Bank A/c XX1234 on 20-10-26", "amount":500,"direction":"debit","category":"Uncategorized","bank":"HDFC","bank_ref":"", "txn_date":"2026-10-20","value_date":"2026-10-20","is_transfer":false,"deleted":false,"sms_source":source});
    for path in ["decide", "import", "lookup"] {
        assert_eq!(
            client
                .post(format!("{url}/api/sms/{path}"))
                .json(&json!({}))
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
    }
    assert_eq!(
        client
            .post(format!("{url}/api/sms/decide"))
            .bearer_auth(&token)
            .json(&json!({"owner_id":a,"body":"OTP 123456"}))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        client
            .post(format!("{url}/api/sms/import"))
            .bearer_auth(&token)
            .json(&json!({"owner_id":b,"operation":operation}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let post = |op: Value| {
        client
            .post(format!("{url}/api/sms/import"))
            .bearer_auth(&token)
            .json(&json!({"owner_id":a,"operation":op}))
            .send()
    };
    let first = post(operation.clone()).await.unwrap();
    assert!(first.status().is_success());
    let row: Value = first.json().await.unwrap();
    assert_eq!(row["amount"], 500.0);
    assert_eq!(row["sms_source"]["payment_method"], "IMPS");
    let retry: Value = post(operation.clone()).await.unwrap().json().await.unwrap();
    assert_eq!(row["id"], retry["id"]);
    assert_eq!(row["rev"], retry["rev"]);
    let found: Value = client
        .post(format!("{url}/api/sms/lookup"))
        .bearer_auth(jwt(b))
        .json(&json!({"owner_id":b,"client_id":operation["client_id"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(found.is_null());
    let own: Value = client
        .post(format!("{url}/api/sms/lookup"))
        .bearer_auth(&token)
        .json(&json!({"owner_id":a,"client_id":operation["client_id"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(own["id"], row["id"]);
    let mut tx = pool.begin().await.unwrap();
    khata::db::set_current_user(&mut *tx, a).await.unwrap();
    sqlx::query("UPDATE transactions SET description='My edited title', rev=rev+1, sms_source=jsonb_set(sms_source,'{user_edited}','true') WHERE id=$1").bind(row["id"].as_str().unwrap().parse::<Uuid>().unwrap()).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(post(operation.clone()).await.unwrap().status(), 409);
    operation["notes"] = json!("OTP 123456 for INR 500");
    assert_eq!(post(operation.clone()).await.unwrap().status(), 400);
    operation["notes"] = json!("INR 500 sent");
    operation["amount"] = json!(500.123);
    assert_eq!(post(operation).await.unwrap().status(), 400);
    task.abort();
}
