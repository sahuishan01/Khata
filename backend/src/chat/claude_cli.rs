use anyhow::{bail, Context, Result};
use std::time::Duration;
use tokio::process::Command;

const TIMEOUT_SECS: u64 = 90;

const SCHEMA_SYSTEM_PROMPT: &str = r#"
You are a SQL assistant for a personal finance app.
The user wants to query their own bank transactions.
Database: PostgreSQL.
You can ONLY query the `transactions` table.
Row-Level Security is active: you NEVER need to add WHERE user_id = ... — it is enforced automatically.
Table `transactions` columns:
  value_date DATE, txn_date DATE, description TEXT,
  amount NUMERIC (always positive), direction TEXT ('debit'|'credit'),
  balance NUMERIC (nullable), bank TEXT, account_label TEXT, bank_ref TEXT (nullable)
Return ONLY a single read-only SELECT with no trailing semicolon.
No DML, no DDL, no subqueries that modify data.
"#;

const SQL_SCHEMA: &str = r#"{"type":"object","properties":{"sql":{"type":"string"},"explanation":{"type":"string"}},"required":["sql","explanation"]}"#;

pub async fn generate_sql(claude_bin: &str, question: &str) -> Result<String> {
    invoke_claude(claude_bin, question, Some(SCHEMA_SYSTEM_PROMPT), Some(SQL_SCHEMA), true).await
}

pub async fn phrase_answer(claude_bin: &str, question: &str, rows_json: &str) -> Result<String> {
    let prompt = format!(
        "The user asked: \"{question}\"\n\nQuery results (JSON):\n{rows_json}\n\n\
         Answer in plain English, 1-3 sentences. Be specific with numbers. \
         Use Indian Rupee symbol ₹ for amounts. If no results, say so."
    );
    invoke_claude(claude_bin, &prompt, None, None, false).await
}

async fn invoke_claude(
    bin: &str,
    prompt: &str,
    system_prompt: Option<&str>,
    json_schema: Option<&str>,
    json_output: bool,
) -> Result<String> {
    let mut cmd = Command::new(bin);
    cmd.arg("-p")
        .arg("--disallowedTools")
        .arg("Bash,Edit,Write,Read,WebSearch,WebFetch,Agent")
        .kill_on_drop(true);

    if json_output {
        cmd.arg("--output-format").arg("json");
    }
    if let Some(sp) = system_prompt {
        cmd.arg("--append-system-prompt").arg(sp);
    }
    if let Some(schema) = json_schema {
        cmd.arg("--json-schema").arg(schema);
    }
    cmd.arg(prompt);

    let output = tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), cmd.output())
        .await
        .context("claude CLI timed out")?
        .context("failed to spawn claude CLI")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("claude CLI exited with error: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if json_output {
        // claude -p --output-format json wraps in {"type":"result","result":"..."}
        // Try to extract the inner result string
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(r) = v.get("result").and_then(|r| r.as_str()) {
                return Ok(r.to_string());
            }
            // Maybe the whole object IS the schema output (some CLI versions)
            return Ok(stdout);
        }
    }
    Ok(stdout)
}
