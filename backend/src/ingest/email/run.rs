//! One email-sync run: connect, search since the watermark, pull statement
//! attachments, push each through the existing parse → normalize → store
//! pipeline, and record progress in `email_sync_runs`.

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ingest::{
    categorize::categorize,
    detect::{detect_bank, detect_file_kind},
    email::{
        imap::{Mailbox, RustlsImap, SearchCriteria},
        mime, pdf_decrypt, txn_email,
    },
    models::NormalizedTxn,
    normalize::normalize,
    profiles::registry,
    store::store_transactions,
};
use crate::AppState;

const STALE_RUN_MINUTES: i64 = 30;
const FETCH_BATCH: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Poll,
    Manual,
}

impl Trigger {
    fn as_str(self) -> &'static str {
        match self {
            Trigger::Poll => "poll",
            Trigger::Manual => "manual",
        }
    }
}

#[derive(Default)]
struct Counters {
    messages_scanned: i32,
    attachments_seen: i32,
    attachments_parsed: i32,
    txns_imported: i32,
    txns_skipped: i32,
    errors: Vec<serde_json::Value>,
}

impl Counters {
    fn push_error(&mut self, stage: &str, item: &str, detail: &str) {
        self.errors.push(serde_json::json!({
            "stage": stage, "item": item, "detail": detail
        }));
    }
}

struct EmailConfig {
    email_address: String,
    enc_app_password: String,
    enc_pdf_password: Option<String>,
    imap_server: String,
    imap_folder: String,
    parse_txn_emails: bool,
    sender_allowlist: Vec<String>,
    subject_patterns: Vec<String>,
    imap_uid_validity: Option<i64>,
    last_uid: Option<i64>,
}

/// Public entry point. Spawned detached; never panics. Returns the run id when a
/// run was started (so a manual caller can report it), or an `AppError`-shaped
/// message when one was already in flight.
pub async fn sync_user(state: AppState, user_id: Uuid, trigger: Trigger) -> Result<Uuid> {
    let cfg = load_config(&state.db, user_id)
        .await?
        .ok_or_else(|| anyhow!("No active Gmail configuration. Connect Gmail first."))?;

    let full_scan = cfg.last_uid.is_none();
    let run_id = start_run(&state.db, user_id, trigger, full_scan).await?;

    // The actual work runs detached so a manual trigger returns immediately.
    // catch_unwind so a panic in a dependency (pdf-extract, mail-parser…) marks
    // the run errored instead of leaving it stuck 'running'.
    let st = state.clone();
    tokio::spawn(async move {
        use futures::FutureExt;
        let work = async {
            tokio::time::timeout(
                std::time::Duration::from_secs(25 * 60),
                execute(&st, user_id, run_id, cfg),
            ).await.map_err(|_| anyhow!("Email sync timed out. Retry to continue from the last saved batch."))?
        };
        let outcome = std::panic::AssertUnwindSafe(work)
            .catch_unwind()
            .await;
        let err = match outcome {
            Ok(Ok(())) => None,
            Ok(Err(e)) => Some(format!("{e:#}")),
            Err(p) => Some(format!(
                "internal error: {}",
                p.downcast_ref::<&str>().copied().unwrap_or("panic")
            )),
        };
        if let Some(msg) = err {
            let _ = finish_run_error(&st.db, user_id, run_id, &msg).await;
            tracing::warn!(%user_id, %run_id, "email sync run failed: {msg}");
        }
    });

    Ok(run_id)
}

/// Same as `sync_user` but drives a caller-supplied mailbox and runs inline —
/// used by tests.
#[cfg(test)]
pub async fn run_inline<M: Mailbox>(
    state: &AppState,
    user_id: Uuid,
    trigger: Trigger,
    mailbox: M,
) -> Result<(Uuid, i32, i32)> {
    let cfg = load_config(&state.db, user_id).await?.expect("config");
    let full_scan = cfg.last_uid.is_none();
    let run_id = start_run(&state.db, user_id, trigger, full_scan).await?;
    let counters = drive(state, user_id, run_id, &cfg, mailbox).await?;
    finish_run_ok(&state.db, user_id, run_id, &cfg, &counters, None).await?;
    Ok((run_id, counters.txns_imported, counters.txns_skipped))
}

async fn execute(state: &AppState, user_id: Uuid, run_id: Uuid, cfg: EmailConfig) -> Result<()> {
    let secret = &state.config.jwt_secret;
    let uid_str = user_id.to_string();
    let app_password = crate::ingest::crypto::decrypt_credential(
        &cfg.enc_app_password,
        secret,
        &uid_str,
    )
    .context("decrypt app password")?;

    let mailbox = tokio::time::timeout(
        std::time::Duration::from_secs(45),
        RustlsImap::connect(&cfg.imap_server, &cfg.email_address, &app_password, &cfg.imap_folder),
    )
    .await
    .map_err(|_| anyhow!("IMAP connect timed out"))?
    .context("IMAP connect")?;
    // app_password dropped here

    let counters = drive(state, user_id, run_id, &cfg, mailbox).await?;
    finish_run_ok(&state.db, user_id, run_id, &cfg, &counters, None).await?;
    Ok(())
}

async fn drive<M: Mailbox>(
    state: &AppState,
    user_id: Uuid,
    run_id: Uuid,
    cfg: &EmailConfig,
    mut mailbox: M,
) -> Result<Counters> {
    let mut counters = Counters::default();

    set_progress(&state.db, user_id, run_id, "searching", None, false).await?;
    let server_validity = mailbox.uid_validity().await.context("read mailbox identity")? as i64;
    let validity_changed = cfg
        .imap_uid_validity
        .map(|v| v != server_validity)
        .unwrap_or(false);
    let since_uid = if validity_changed || cfg.last_uid.is_none() {
        None
    } else {
        cfg.last_uid.map(|u| u as u32)
    };

    let criteria = SearchCriteria {
        from_any: cfg.sender_allowlist.clone(),
        subject_any: cfg.subject_patterns.clone(),
    };
    let max_messages = state.config.email_sync_max_messages.max(1);
    let mut uids = tokio::time::timeout(
        std::time::Duration::from_secs(90),
        mailbox.search(since_uid, &criteria, max_messages.saturating_add(1)),
    )
        .await.map_err(|_| anyhow!("Mailbox search timed out. Please retry."))?
        .context("IMAP search")?;
    let has_more = uids.len() > max_messages;
    uids.truncate(max_messages);
    set_progress(&state.db, user_id, run_id, "fetching", Some(uids.len() as i32), has_more).await?;

    let secret = &state.config.jwt_secret;
    let uid_str = user_id.to_string();
    let pdf_password = cfg
        .enc_pdf_password
        .as_deref()
        .and_then(|enc| {
            crate::ingest::crypto::decrypt_credential(enc, secret, &uid_str).ok()
        });

    let profiles = registry();
    let mut max_uid_seen = since_uid.unwrap_or(0) as i64;

    for chunk in uids.chunks(FETCH_BATCH) {
        set_progress(&state.db, user_id, run_id, "fetching", None, has_more).await?;
        let mut messages = tokio::time::timeout(
            std::time::Duration::from_secs(90), mailbox.fetch(chunk),
        ).await.map_err(|_| anyhow!("Fetching email timed out. Retry to continue."))?
            .context("IMAP fetch")?;
        messages.sort_by_key(|msg| msg.uid);
        set_progress(&state.db, user_id, run_id, "processing", None, has_more).await?;
        for msg in messages {
            counters.messages_scanned += 1;
            max_uid_seen = max_uid_seen.max(msg.uid as i64);

            let attachments = mime::statement_attachments(&msg.raw);

            if attachments.is_empty() {
                // No statement file — try to read a transaction out of the body.
                if cfg.parse_txn_emails {
                    if let Err(e) =
                        process_body_txn(state, user_id, &profiles, &msg.raw, &mut counters).await
                    {
                        counters.push_error("txn-email", "message body", &format!("{e:#}"));
                    }
                }
                flush_counters(&state.db, user_id, run_id, &counters).await?;
                continue;
            }

            for att in attachments {
                counters.attachments_seen += 1;

                if att.bytes.len() > state.config.email_sync_max_attach_bytes {
                    counters.push_error("attachment", &att.filename, "attachment exceeds size cap");
                    continue;
                }

                if let Err(e) = process_attachment(
                    state,
                    user_id,
                    &profiles,
                    &att.filename,
                    att.bytes,
                    pdf_password.as_deref(),
                    &mut counters,
                )
                .await
                {
                    counters.push_error("import", &att.filename, &format!("{e:#}"));
                }
            }
            flush_counters(&state.db, user_id, run_id, &counters).await?;
        }
        flush_counters(&state.db, user_id, run_id, &counters).await?;
        persist_watermark(&state.db, user_id, max_uid_seen, server_validity).await?;
    }

    // Persist watermark.
    persist_watermark(&state.db, user_id, max_uid_seen, server_validity).await?;
    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), mailbox.logout()).await;
    Ok(counters)
}

async fn process_attachment(
    state: &AppState,
    user_id: Uuid,
    profiles: &[crate::ingest::profiles::BankProfile],
    filename: &str,
    bytes: Vec<u8>,
    pdf_password: Option<&str>,
    counters: &mut Counters,
) -> Result<()> {
    let sha = hex::encode(Sha256::digest(&bytes));

    // Attachment already imported?
    let seen: Option<(Uuid,)> = {
        let mut tx = state.db.begin().await?;
        crate::db::set_current_user(&mut *tx, user_id).await?;
        let r = sqlx::query_as(
            "SELECT id FROM statements WHERE user_id = $1 AND file_sha256 = $2",
        )
        .bind(user_id)
        .bind(&sha)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        r
    };
    if seen.is_some() {
        return Ok(()); // attachment already imported in a prior run
    }

    let kind = detect_file_kind(filename);

    // Decrypt encrypted PDFs before parsing.
    let bytes = if kind == crate::ingest::detect::FileKind::Pdf && pdf_decrypt::is_encrypted(&bytes) {
        match pdf_password {
            Some(pw) => pdf_decrypt::decrypt_pdf(&bytes, pw)
                .context("decrypt statement PDF")?,
            None => {
                counters.push_error("decrypt", filename, "encrypted PDF but no statement password stored");
                return Ok(());
            }
        }
    } else {
        bytes
    };

    // Two-pass parse: generic profile for the bank hint, then the real profile.
    // pdf-extract can panic on malformed PDFs — contain it per-attachment.
    let hint = if kind == crate::ingest::detect::FileKind::Pdf {
        // `extract_text` can't panic — no catch_unwind needed.
        crate::ingest::pdf::extract_text(&bytes).to_lowercase()
    } else {
        safe_parse(&bytes, kind.clone(), profiles.last().unwrap())?.2
    };
    let profile = detect_bank(profiles, &hint);
    let (raw_rows, _, _, low_conf) = safe_parse(&bytes, kind, profile)?;

    // Guard against false positives: a real statement has several rows, and a
    // GENERIC match means the bank fingerprint never hit.
    if raw_rows.len() < 2 || profile.name == "GENERIC" {
        counters.push_error("parse", filename, "not recognised as a supported bank statement");
        return Ok(());
    }
    let rows_parsed = raw_rows.len() as i32;

    let stmt_id: Option<(Uuid,)> = {
        let mut tx = state.db.begin().await?;
        crate::db::set_current_user(&mut *tx, user_id).await?;
        let r = sqlx::query_as(
            "INSERT INTO statements (user_id, bank, file_name, file_sha256, row_count) \
             VALUES ($1,$2,$3,$4,$5) ON CONFLICT (user_id, file_sha256) DO NOTHING RETURNING id",
        )
        .bind(user_id)
        .bind(profile.name)
        .bind(filename)
        .bind(&sha)
        .bind(rows_parsed)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        r
    };
    let Some((stmt_id,)) = stmt_id else {
        return Ok(()); // race: another run inserted it
    };

    let txns = normalize(raw_rows, user_id, stmt_id, profile, "email");
    if txns.is_empty() {
        counters.attachments_parsed += 1;
        return Ok(());
    }
    let (inserted, skipped) = store_transactions(&state.db, user_id, &txns).await?;
    counters.attachments_parsed += 1;
    counters.txns_imported += inserted as i32;
    counters.txns_skipped += skipped as i32;
    if let Some(reason) = low_conf {
        counters.push_error("parse-confidence", filename, &reason);
    }
    Ok(())
}

/// Parse one transaction out of an alert email's body (no attachment). Keyed by
/// the raw message hash so re-runs dedupe; the txn itself also dedupes on its
/// fingerprint against everything imported from statements.
async fn process_body_txn(
    state: &AppState,
    user_id: Uuid,
    profiles: &[crate::ingest::profiles::BankProfile],
    raw: &[u8],
    counters: &mut Counters,
) -> Result<()> {
    let Some(meta) = mime::message_meta(raw) else {
        return Ok(());
    };
    let Some(bt) = txn_email::extract(&meta.subject, &meta.body, &meta.from) else {
        return Ok(());
    };

    let sha = hex::encode(Sha256::digest(raw));

    let seen: Option<(Uuid,)> = {
        let mut tx = state.db.begin().await?;
        crate::db::set_current_user(&mut *tx, user_id).await?;
        let r = sqlx::query_as("SELECT id FROM statements WHERE user_id = $1 AND file_sha256 = $2")
            .bind(user_id)
            .bind(&sha)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        r
    };
    if seen.is_some() {
        return Ok(());
    }

    let hint = format!("{} {} {}", meta.from, meta.subject, meta.body).to_lowercase();
    let profile = detect_bank(profiles, &hint);
    let bank = if profile.name == "GENERIC" {
        "Email".to_string()
    } else {
        profile.name.to_string()
    };

    let date = bt
        .txn_date
        .or_else(|| {
            meta.date_epoch
                .and_then(|e| chrono::DateTime::from_timestamp(e, 0))
                .map(|d| d.date_naive())
        })
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let account_label = bt.account_label.clone().unwrap_or_else(|| bank.clone());

    let stmt_id: Option<(Uuid,)> = {
        let mut tx = state.db.begin().await?;
        crate::db::set_current_user(&mut *tx, user_id).await?;
        let subject = if meta.subject.is_empty() { "transaction alert" } else { &meta.subject };
        let r = sqlx::query_as(
            "INSERT INTO statements (user_id, bank, file_name, file_sha256, row_count) \
             VALUES ($1,$2,$3,$4,1) ON CONFLICT (user_id, file_sha256) DO NOTHING RETURNING id",
        )
        .bind(user_id)
        .bind(&bank)
        .bind(subject)
        .bind(&sha)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        r
    };
    let Some((stmt_id,)) = stmt_id else {
        return Ok(());
    };

    // Classify using the original alert before the payee extractor discards its context.
    let repayment: bool = sqlx::query_scalar("SELECT is_card_repayment($1, $2, false)")
        .bind(format!("{} {}", meta.subject, meta.body))
        .bind(&bt.direction)
        .fetch_one(&state.db).await?;
    let description = if repayment {
        if bt.direction == "credit" { "Credit card payment received" } else { "Credit card bill payment" }.to_string()
    } else { bt.description.clone() };
    let txn = NormalizedTxn {
        user_id,
        statement_id: stmt_id,
        bank,
        account_label,
        txn_date: date,
        value_date: date,
        description,
        raw_description: bt.description.clone(),
        amount: bt.amount,
        direction: bt.direction.clone(),
        balance: None,
        bank_ref: bt.bank_ref.clone(),
        category: categorize(&bt.description, &bt.direction).to_string(),
    };

    let (inserted, skipped) = store_transactions(&state.db, user_id, &[txn]).await?;
    counters.txns_imported += inserted as i32;
    counters.txns_skipped += skipped as i32;
    Ok(())
}

type ParseOut = (Vec<crate::ingest::models::RawRow>, Vec<String>, String, Option<String>);

/// `parse_file` but a panic (pdf-extract on a malformed PDF) becomes an `Err`.
fn safe_parse(
    bytes: &[u8],
    kind: crate::ingest::detect::FileKind,
    profile: &crate::ingest::profiles::BankProfile,
) -> Result<ParseOut> {
    crate::ingest::parse::parse_file_safe(bytes, kind, profile).map_err(|e| anyhow!("parse: {e}"))
}

// ── DB helpers ────────────────────────────────────────────────────────────────

async fn load_config(db: &PgPool, user_id: Uuid) -> Result<Option<EmailConfig>> {
    let mut tx = db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    let row: Option<(
        String, String, Option<String>, String, String, bool,
        Option<Vec<String>>, Option<Vec<String>>, Option<i64>, Option<i64>,
    )> = sqlx::query_as(
        "SELECT email_address, encrypted_app_password, \
                NULLIF(encrypted_pdf_password, ''), imap_server, imap_folder, parse_txn_emails, \
                sender_allowlist, subject_patterns, imap_uid_validity, last_uid \
         FROM user_email_configs WHERE user_id = $1 AND sync_enabled = true",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(row.map(|(email, app, pdf, imap, folder, parse_txn, senders, subjects, validity, last_uid)| EmailConfig {
        email_address: email,
        enc_app_password: app,
        enc_pdf_password: pdf,
        imap_server: imap,
        imap_folder: folder,
        parse_txn_emails: parse_txn,
        sender_allowlist: senders.unwrap_or_default(),
        subject_patterns: subjects.unwrap_or_default(),
        imap_uid_validity: validity,
        last_uid,
    }))
}

async fn start_run(db: &PgPool, user_id: Uuid, trigger: Trigger, full_scan: bool) -> Result<Uuid> {
    let mut tx = db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    // Serialize starts for this user so double-clicks/background polling cannot
    // both pass the running-run check.
    sqlx::query("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(user_id).execute(&mut *tx).await?;

    // Live run? (allow superseding stale ones)
    let live: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM email_sync_runs \
         WHERE user_id = $1 AND status = 'running' \
           AND updated_at > now() - ($2 || ' minutes')::interval",
    )
    .bind(user_id)
    .bind(STALE_RUN_MINUTES.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    if live.is_some() {
        tx.rollback().await?;
        return Err(anyhow!("A sync is already running"));
    }

    sqlx::query(
        "UPDATE email_sync_runs SET status = 'error', phase = 'failed', \
         error = 'Previous sync stopped responding. A new run has started.', finished_at = now(), updated_at = now() \
         WHERE user_id = $1 AND status = 'running'",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO email_sync_runs (user_id, trigger, full_scan) VALUES ($1,$2,$3) RETURNING id",
    )
    .bind(user_id)
    .bind(trigger.as_str())
    .bind(full_scan)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(id)
}

async fn flush_counters(db: &PgPool, user_id: Uuid, run_id: Uuid, c: &Counters) -> Result<()> {
    let mut tx = db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    sqlx::query(
        "UPDATE email_sync_runs SET messages_scanned=$1, attachments_seen=$2, \
             attachments_parsed=$3, txns_imported=$4, txns_skipped=$5, errors=$6, updated_at=now() \
         WHERE id = $7",
    )
    .bind(c.messages_scanned)
    .bind(c.attachments_seen)
    .bind(c.attachments_parsed)
    .bind(c.txns_imported)
    .bind(c.txns_skipped)
    .bind(serde_json::Value::Array(c.errors.clone()))
    .bind(run_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn set_progress(
    db: &PgPool, user_id: Uuid, run_id: Uuid, phase: &str,
    total: Option<i32>, has_more: bool,
) -> Result<()> {
    let mut tx = db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    sqlx::query(
        "UPDATE email_sync_runs SET phase=$1, messages_total=COALESCE($2,messages_total), \
         has_more=$3, updated_at=now() WHERE id=$4 AND user_id=$5",
    ).bind(phase).bind(total).bind(has_more).bind(run_id).bind(user_id)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn persist_watermark(db: &PgPool, user_id: Uuid, last_uid: i64, validity: i64) -> Result<()> {
    let mut tx = db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    sqlx::query(
        "UPDATE user_email_configs \
         SET last_uid = $1, imap_uid_validity = $2, last_synced_at = now(), last_error = NULL \
         WHERE user_id = $3",
    )
    .bind(last_uid)
    .bind(validity)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn finish_run_ok(
    db: &PgPool,
    user_id: Uuid,
    run_id: Uuid,
    _cfg: &EmailConfig,
    c: &Counters,
    _validity: Option<i64>,
) -> Result<()> {
    let mut tx = db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    sqlx::query(
        "UPDATE email_sync_runs SET status='ok', phase='complete', finished_at=now(), updated_at=now(), \
             messages_scanned=$1, attachments_seen=$2, attachments_parsed=$3, \
             txns_imported=$4, txns_skipped=$5, errors=$6 WHERE id=$7",
    )
    .bind(c.messages_scanned)
    .bind(c.attachments_seen)
    .bind(c.attachments_parsed)
    .bind(c.txns_imported)
    .bind(c.txns_skipped)
    .bind(serde_json::Value::Array(c.errors.clone()))
    .bind(run_id)
    .execute(&mut *tx)
    .await?;

    // Surface "needs a PDF password" prominently: it's a fixable config gap, not
    // a transient per-item glitch.
    let needs_pw = c.errors.iter().any(|e| {
        e.get("detail")
            .and_then(|d| d.as_str())
            .is_some_and(|d| d.contains("password"))
    });
    if needs_pw {
        let msg = "Some statement PDFs are password-protected — add your statement password under Gmail Sync.";
        sqlx::query("UPDATE email_sync_runs SET error = $1 WHERE id = $2")
            .bind(msg)
            .bind(run_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE user_email_configs SET last_error = $1 WHERE user_id = $2")
            .bind(msg)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::email::imap::{FetchedMessage, Mailbox, SearchCriteria};
    use async_trait::async_trait;
    use sqlx::PgPool;
    use std::sync::{Arc, Mutex};

    fn app_state(pool: &PgPool) -> AppState {
        AppState {
            db: pool.clone(),
            db_ro: pool.clone(),
            config: Arc::new(crate::config::Config {
                database_url: String::new(),
                ro_database_url: String::new(),
                jwt_secret: "test-secret-32-chars-min-aaaaaaaaaa".into(),
                claude_bin: "claude".into(),
                bind_addr: "127.0.0.1:0".into(),
                cors_origins: vec![],
                cookie_secure: true,
                allow_remote_setup: false,
                email_sync_poll_secs: 0,
                email_sync_max_messages: 200,
                email_sync_max_attach_bytes: 15 * 1024 * 1024,
                pdfium_lib_path: None,
            }),
            chat_ratelimit: Arc::new(Mutex::new(std::collections::HashMap::new())),
            login_attempts: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    fn eml_with_attachment(name: &str, mime: &str, body_b64: &str) -> Vec<u8> {
        format!(
            "From: alerts@hdfcbank.net\r\nTo: me@example.com\r\nSubject: Statement\r\n\
             MIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"b\"\r\n\r\n\
             --b\r\nContent-Type: text/plain\r\n\r\nStatement attached.\r\n\
             --b\r\nContent-Type: {mime}; name=\"{name}\"\r\n\
             Content-Disposition: attachment; filename=\"{name}\"\r\n\
             Content-Transfer-Encoding: base64\r\n\r\n{body_b64}\r\n--b--\r\n"
        )
        .into_bytes()
    }

    fn eml_plain(subject: &str, body: &str) -> Vec<u8> {
        format!(
            "From: alerts@hdfcbank.net\r\nTo: me@example.com\r\nSubject: {subject}\r\n\
             Date: Thu, 05 Sep 2024 10:00:00 +0530\r\n\
             MIME-Version: 1.0\r\nContent-Type: text/plain\r\n\r\n{body}\r\n"
        )
        .into_bytes()
    }

    fn hdfc_csv_b64() -> String {
        use base64::Engine;
        let csv = "HDFC Bank Ltd Statement\r\n\
                   Date,Narration,Withdrawal Amt,Deposit Amt,Closing Balance\r\n\
                   01/03/2024,UPI-AMAZON,500.00,,10000.00\r\n\
                   02/03/2024,SALARY CREDIT,,50000.00,60000.00\r\n";
        base64::engine::general_purpose::STANDARD.encode(csv)
    }

    struct FakeMailbox {
        messages: Vec<FetchedMessage>,
        validity: u64,
    }

    #[async_trait]
    impl Mailbox for FakeMailbox {
        async fn uid_validity(&mut self) -> Result<u64> {
            Ok(self.validity)
        }
        async fn search(
            &mut self,
            since_uid: Option<u32>,
            _c: &SearchCriteria,
            limit: usize,
        ) -> Result<Vec<u32>> {
            let lo = since_uid.map(|u| u + 1).unwrap_or(0);
            let mut uids: Vec<u32> = self
                .messages
                .iter()
                .map(|m| m.uid)
                .filter(|u| *u >= lo)
                .collect();
            uids.sort_unstable();
            uids.truncate(limit);
            Ok(uids)
        }
        async fn fetch(&mut self, uids: &[u32]) -> Result<Vec<FetchedMessage>> {
            Ok(self
                .messages
                .iter()
                .filter(|m| uids.contains(&m.uid))
                .map(|m| FetchedMessage { uid: m.uid, raw: m.raw.clone() })
                .collect())
        }
        async fn logout(&mut self) -> Result<()> {
            Ok(())
        }
    }

    /// Run a scalar query with the RLS user context set (mirrors what a real
    /// request handler does). Direct pooled queries against forced-RLS tables
    /// would hit `''::uuid` once a connection has served any prior transaction.
    async fn scalar_i64(pool: &PgPool, user_id: Uuid, sql: &str) -> i64 {
        let mut tx = pool.begin().await.unwrap();
        crate::db::set_current_user(&mut *tx, user_id).await.unwrap();
        let (n,): (i64,) = sqlx::query_as(sql).bind(user_id).fetch_one(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
        n
    }

    async fn seed_user_and_config(pool: &PgPool, state: &AppState) -> Uuid {
        let (user_id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash) VALUES ('u@e.com','x') RETURNING id",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let enc = crate::ingest::crypto::encrypt_credential(
            "app-pw-1234",
            &state.config.jwt_secret,
            &user_id.to_string(),
        )
        .unwrap();
        sqlx::query(
            "INSERT INTO user_email_configs (user_id, email_address, encrypted_app_password) \
             VALUES ($1, 'u@e.com', $2)",
        )
        .bind(user_id)
        .bind(enc)
        .execute(pool)
        .await
        .unwrap();
        user_id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn progress_and_watermark_continue_without_skipping_old_mail(pool: PgPool) {
        let mut state = app_state(&pool);
        std::sync::Arc::get_mut(&mut state.config).unwrap().email_sync_max_messages = 2;
        let user_id = seed_user_and_config(&pool, &state).await;
        struct ProgressMailbox { inner: FakeMailbox, db: PgPool, user_id: Uuid }
        #[async_trait]
        impl Mailbox for ProgressMailbox {
            async fn uid_validity(&mut self) -> Result<u64> { self.inner.uid_validity().await }
            async fn search(&mut self, since: Option<u32>, c: &SearchCriteria, limit: usize) -> Result<Vec<u32>> {
                let phase: String = sqlx::query_scalar("SELECT phase FROM email_sync_runs WHERE user_id=$1 ORDER BY started_at DESC LIMIT 1")
                    .bind(self.user_id).fetch_one(&self.db).await?;
                assert_eq!(phase, "searching");
                self.inner.search(since, c, limit).await
            }
            async fn fetch(&mut self, ids: &[u32]) -> Result<Vec<FetchedMessage>> {
                let progress: (String, Option<i32>) = sqlx::query_as("SELECT phase,messages_total FROM email_sync_runs WHERE user_id=$1 ORDER BY started_at DESC LIMIT 1")
                    .bind(self.user_id).fetch_one(&self.db).await?;
                assert_eq!(progress, ("fetching".into(), Some(2)));
                self.inner.fetch(ids).await
            }
            async fn logout(&mut self) -> Result<()> { Ok(()) }
        }
        for expected_uid in [2_i64, 4] {
            let mailbox = ProgressMailbox {
                inner: FakeMailbox { validity: 1, messages: (1..=5).map(|uid| FetchedMessage { uid, raw: b"From: bank@example.com\r\nSubject: Notice\r\n\r\nNo transaction.".to_vec() }).collect() },
                db: pool.clone(), user_id,
            };
            let (id, _, _) = run_inline(&state, user_id, Trigger::Manual, mailbox).await.unwrap();
            let progress: (String, String, Option<i32>, i32, bool) = sqlx::query_as(
                "SELECT status,phase,messages_total,messages_scanned,has_more FROM email_sync_runs WHERE id=$1",
            ).bind(id).fetch_one(&pool).await.unwrap();
            assert_eq!(progress, ("ok".into(), "complete".into(), Some(2), 2, true));
            let uid: i64 = sqlx::query_scalar("SELECT last_uid FROM user_email_configs WHERE user_id=$1")
                .bind(user_id).fetch_one(&pool).await.unwrap();
            assert_eq!(uid, expected_uid);
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn only_one_run_can_start_per_user(pool: PgPool) {
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;
        let (a, b) = tokio::join!(start_run(&pool, user_id, Trigger::Manual, true), start_run(&pool, user_id, Trigger::Poll, true));
        assert_ne!(a.is_ok(), b.is_ok());
        let id = a.or(b).unwrap();
        finish_run_error(&pool, user_id, id, "Test connection failure").await.unwrap();
        let row: (String, String, bool) = sqlx::query_as("SELECT status,phase,finished_at IS NOT NULL FROM email_sync_runs WHERE id=$1")
            .bind(id).fetch_one(&pool).await.unwrap();
        assert_eq!(row, ("error".into(), "failed".into(), true));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn device_import_is_idempotent_and_scoped_to_its_user(pool: PgPool) {
        use crate::auth::middleware::CurrentUser;
        use crate::txns::{handlers::sync_push, models::SyncPushOp};
        use axum::{extract::State, Json};
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;
        let client_id = Uuid::new_v4();
        let operation = || SyncPushOp {
            client_id, base_rev: 0, amount: Some(250.0), direction: Some("debit".into()), description: Some("SMS test".into()),
            category: None, bank: Some("HDFC".into()), bank_ref: Some("1234567890".into()), txn_date: None, value_date: None, notes: None, is_transfer: None, deleted: None, sms_source: None,
        };
        let first = sync_push(State(state.clone()), CurrentUser(user_id), Json(vec![operation()])).await.unwrap().0;
        assert_eq!(first.accepted.len(), 1);
        let retry = sync_push(State(state.clone()), CurrentUser(user_id), Json(vec![operation()])).await.unwrap().0;
        assert_eq!(retry.conflicts.len(), 1);
        let other: Uuid = sqlx::query_scalar("INSERT INTO users (email,password_hash) VALUES ('other@example.com','x') RETURNING id")
            .fetch_one(&pool).await.unwrap();
        let second = sync_push(State(state), CurrentUser(other), Json(vec![operation()])).await.unwrap().0;
        assert_eq!(second.accepted.len(), 1);
        let count = scalar_i64(&pool, user_id, "SELECT count(*) FROM transactions").await;
        assert_eq!(count, 1, "the retry must not duplicate the first user's transaction");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn device_download_pages_all_imports_with_the_same_row_revision(pool: PgPool) {
        use crate::auth::middleware::CurrentUser;
        use crate::txns::{handlers::{sync_push, sync_pull}, models::{SyncPushOp, SyncPullQuery}};
        use axum::{extract::{State, Query}, Json};
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;
        let ops = (0..205).map(|_| SyncPushOp {
            client_id: Uuid::new_v4(), base_rev: 0, amount: Some(250.0), direction: Some("debit".into()),
            description: Some("SMS backfill".into()), category: None, bank: Some("HDFC".into()), bank_ref: None,
            txn_date: None, value_date: None, notes: None, is_transfer: None, deleted: None, sms_source: None,
        }).collect();
        assert_eq!(sync_push(State(state.clone()), CurrentUser(user_id), Json(ops)).await.unwrap().0.accepted.len(), 205);
        let first = sync_pull(State(state.clone()), CurrentUser(user_id), Query(SyncPullQuery {
            since_rev: Some(0), limit: Some(200), cursor_version: Some(2),
        })).await.unwrap().0;
        assert_eq!(first.txns.len(), 200);
        assert!(first.has_more);
        assert!(first.txns.iter().all(|row| row.rev == 1 && row.bank == "HDFC"));
        let next = sync_pull(State(state), CurrentUser(user_id), Query(SyncPullQuery {
            since_rev: Some(first.new_rev), limit: Some(200), cursor_version: Some(2),
        })).await.unwrap().0;
        assert_eq!(next.txns.len(), 5);
        assert!(!next.has_more);
        assert!(next.txns.iter().all(|row| !first.txns.iter().any(|old| old.id == row.id)));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn cursor_upgrade_backfills_existing_transactions_under_forced_rls(pool: PgPool) {
        use crate::auth::middleware::CurrentUser;
        use crate::txns::{handlers::{sync_push, sync_pull}, models::{SyncPushOp, SyncPullQuery}};
        use axum::{extract::{State, Query}, Json};
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;
        // Reproduce the schema immediately before the cursor migration.
        sqlx::raw_sql("DROP TRIGGER transaction_sync_cursor_changed ON transactions; DROP FUNCTION assign_transaction_sync_cursor(); ALTER TABLE transactions DROP COLUMN sync_rev; DROP SEQUENCE transaction_sync_cursor;")
            .execute(&pool).await.unwrap();
        let op = SyncPushOp {
            client_id: Uuid::new_v4(), base_rev: 0, amount: Some(100.0), direction: Some("credit".into()),
            description: Some("Existing transaction".into()), category: None, bank: None, bank_ref: None,
            txn_date: None, value_date: None, notes: None, is_transfer: None, deleted: None, sms_source: None,
        };
        sync_push(State(state.clone()), CurrentUser(user_id), Json(vec![op])).await.unwrap();
        sqlx::raw_sql(include_str!("../../../migrations/0033_sync_change_cursor.sql"))
            .execute(&pool).await.unwrap();
        let response = sync_pull(State(state), CurrentUser(user_id), Query(SyncPullQuery {
            since_rev: Some(0), limit: Some(200), cursor_version: Some(2),
        })).await.unwrap().0;
        assert_eq!(response.txns.len(), 1, "the upgrade must preserve visibility of existing records");
        assert!(response.new_rev > 0);
        let forced: bool = sqlx::query_scalar("SELECT relforcerowsecurity FROM pg_class WHERE oid='transactions'::regclass")
            .fetch_one(&pool).await.unwrap();
        assert!(forced, "row-level security must be forced again after backfill");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn setting_pdf_password_retries_previously_scanned_messages(pool: PgPool) {
        use super::super::config::{SaveEmailConfigReq, save_email_config_handler};
        use crate::auth::middleware::CurrentUser;
        use axum::{extract::State, Json};
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;
        sqlx::query("UPDATE user_email_configs SET last_uid=50 WHERE user_id=$1").bind(user_id).execute(&pool).await.unwrap();
        let response = save_email_config_handler(State(state), CurrentUser(user_id), Json(SaveEmailConfigReq {
            email_address: "u@e.com".into(), app_password: None, pdf_password: Some("statement-password".into()),
            imap_server: None, imap_folder: None, parse_txn_emails: None, sync_enabled: None,
            sender_allowlist: None, subject_patterns: None,
        })).await.unwrap().0;
        assert!(response.full_rescan_queued);
        assert!(response.config.has_pdf_password);
        let uid: Option<i64> = sqlx::query_scalar("SELECT last_uid FROM user_email_configs WHERE user_id=$1").bind(user_id).fetch_one(&pool).await.unwrap();
        assert!(uid.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn imports_statement_attachments_and_records_errors(pool: PgPool) {
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;

        let mailbox = FakeMailbox {
            validity: 42,
            messages: vec![
                FetchedMessage {
                    uid: 10,
                    raw: eml_with_attachment("stmt.csv", "text/csv", &hdfc_csv_b64()),
                },
                FetchedMessage {
                    uid: 11,
                    raw: eml_with_attachment("broken.pdf", "application/pdf", "bm90IGEgcGRm"), // "not a pdf"
                },
            ],
        };

        let (run_id, imported, _skipped) =
            run_inline(&state, user_id, Trigger::Manual, mailbox).await.unwrap();
        assert_eq!(imported, 2, "two HDFC rows imported");

        let (status, msgs, seen, parsed, errs): (String, i32, i32, i32, serde_json::Value) =
            sqlx::query_as(
                "SELECT status, messages_scanned, attachments_seen, attachments_parsed, errors \
                 FROM email_sync_runs WHERE id = $1",
            )
            .bind(run_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "ok");
        assert_eq!(msgs, 2);
        assert_eq!(seen, 2);
        assert_eq!(parsed, 1);
        assert_eq!(errs.as_array().unwrap().len(), 1, "broken.pdf recorded");

        let (last_uid, validity): (Option<i64>, Option<i64>) = sqlx::query_as(
            "SELECT last_uid, imap_uid_validity FROM user_email_configs WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(last_uid, Some(11));
        assert_eq!(validity, Some(42));

        let txn_count = scalar_i64(&pool, user_id, "SELECT count(*) FROM transactions WHERE user_id = $1").await;
        assert_eq!(txn_count, 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn imports_transaction_alert_emails(pool: PgPool) {
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;

        let mailbox = FakeMailbox {
            validity: 1,
            messages: vec![
                FetchedMessage {
                    uid: 20,
                    raw: eml_plain(
                        "Txn alert",
                        "Dear Customer, Rs. 450.00 has been debited from account **1234 \
                         to VPA swiggy@ybl on 05-09-2024. UPI Ref 512345678901. - HDFC Bank",
                    ),
                },
                FetchedMessage {
                    uid: 21,
                    raw: eml_plain("OTP", "Your OTP for a txn of Rs 999 is 445566. - HDFC Bank"),
                },
            ],
        };

        let (_run_id, imported, _skipped) =
            run_inline(&state, user_id, Trigger::Manual, mailbox).await.unwrap();
        assert_eq!(imported, 1, "one alert-email txn imported, OTP ignored");

        let (amount, direction, acct): (f64, String, String) = {
            let mut tx = pool.begin().await.unwrap();
            crate::db::set_current_user(&mut *tx, user_id).await.unwrap();
            let r = sqlx::query_as(
                "SELECT amount::float8, direction, account_label FROM transactions WHERE user_id = $1",
            )
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
            tx.commit().await.unwrap();
            r
        };
        assert_eq!(amount, 450.0);
        assert_eq!(direction, "debit");
        assert_eq!(acct, "XX1234");

        // Re-scan the same message → deduped, no second row.
        sqlx::query("UPDATE user_email_configs SET last_uid = NULL WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();
        let rescan = FakeMailbox {
            validity: 1,
            messages: vec![FetchedMessage {
                uid: 20,
                raw: eml_plain(
                    "Txn alert",
                    "Dear Customer, Rs. 450.00 has been debited from account **1234 \
                     to VPA swiggy@ybl on 05-09-2024. UPI Ref 512345678901. - HDFC Bank",
                ),
            }],
        };
        let (_, imported2, _) =
            run_inline(&state, user_id, Trigger::Manual, rescan).await.unwrap();
        assert_eq!(imported2, 0, "same alert email must not import twice");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn second_run_dedupes_same_attachment(pool: PgPool) {
        let state = app_state(&pool);
        let user_id = seed_user_and_config(&pool, &state).await;

        let make_box = || FakeMailbox {
            validity: 1,
            messages: vec![FetchedMessage {
                uid: 5,
                raw: eml_with_attachment("s.csv", "text/csv", &hdfc_csv_b64()),
            }],
        };

        let (_, first, _) =
            run_inline(&state, user_id, Trigger::Manual, make_box()).await.unwrap();
        assert_eq!(first, 2);

        // watermark now at 5; a re-scan (reset last_uid) sees the same sha → 0 new statements
        sqlx::query("UPDATE user_email_configs SET last_uid = NULL WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

        let (_, second, _) =
            run_inline(&state, user_id, Trigger::Manual, make_box()).await.unwrap();
        assert_eq!(second, 0, "same attachment sha → skipped");

        let stmt_count = scalar_i64(&pool, user_id, "SELECT count(*) FROM statements WHERE user_id = $1").await;
        assert_eq!(stmt_count, 1);
    }
}

async fn finish_run_error(db: &PgPool, user_id: Uuid, run_id: Uuid, msg: &str) -> Result<()> {
    let mut tx = db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    sqlx::query(
        "UPDATE email_sync_runs SET status='error', phase='failed', error=$1, finished_at=now(), updated_at=now() WHERE id=$2",
    )
    .bind(msg)
    .bind(run_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE user_email_configs SET last_error = $1 WHERE user_id = $2")
        .bind(msg)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
