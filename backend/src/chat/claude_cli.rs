use anyhow::{bail, Context, Result};
use std::time::Duration;
use tokio::process::Command;

const TIMEOUT_SECS: u64 = 90;

// Embedded directly in the user prompt so Claude cannot ignore it
const SQL_PROMPT_PREFIX: &str = r#"You are a SQL assistant for a personal finance app.

RULES (MUST follow):
1. Respond ONLY with a JSON object — no preamble, no markdown, no explanation outside JSON.
2. The JSON must have exactly two keys: "sql" and "explanation".
3. "sql" must be a single read-only SELECT query for PostgreSQL (no semicolon).
4. "explanation" is a one-sentence description of what the query does.
5. Row-Level Security is active: do NOT add WHERE user_id = ... — it is automatic.

Table: transactions
Columns: value_date DATE, txn_date DATE, description TEXT,
         amount NUMERIC (always positive), direction TEXT ('debit'|'credit'),
         balance NUMERIC nullable, bank TEXT, account_label TEXT, bank_ref TEXT nullable

Example response (output this exact format, no other text):
{"sql":"SELECT SUM(amount) FROM transactions WHERE direction='debit'","explanation":"Total amount debited"}

User question: "#;

pub async fn generate_sql(claude_bin: &str, question: &str) -> Result<String> {
    let prompt = format!("{SQL_PROMPT_PREFIX}{question}");
    let raw = run_claude(claude_bin, &prompt).await?;
    extract_json_object(&raw)
        .ok_or_else(|| anyhow::anyhow!("Claude did not return valid JSON.\nRaw response:\n{raw}"))
}

pub async fn phrase_answer(claude_bin: &str, question: &str, rows_json: &str) -> Result<String> {
    let prompt = format!(
        "The user asked: \"{question}\"\n\nQuery results (JSON array):\n{rows_json}\n\n\
         Answer in plain English, 1-3 sentences. Be specific with numbers and dates. \
         Use ₹ for rupee amounts (e.g. ₹1,500). If the results array is empty, say no matching data was found."
    );
    run_claude(claude_bin, &prompt).await
}

async fn run_claude(bin: &str, prompt: &str) -> Result<String> {
    let mut cmd = Command::new(bin);
    cmd.arg("-p")
        .arg("--output-format").arg("text")
        .arg("--disallowedTools")
        .arg("Bash,Edit,Write,Read,WebSearch,WebFetch,Agent")
        .kill_on_drop(true)
        .arg(prompt);

    let output = tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), cmd.output())
        .await
        .context("claude CLI timed out")?
        .context("failed to spawn claude CLI")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("claude CLI error: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Extract the first complete JSON object from text.
/// Handles: bare JSON, JSON inside ```json ... ```, JSON inside ``` ... ```.
fn extract_json_object(text: &str) -> Option<String> {
    // 1. Direct parse
    if serde_json::from_str::<serde_json::Value>(text).is_ok() {
        return Some(text.to_string());
    }

    // 2. Inside ```json ... ``` or ``` ... ```
    for fence in &["```json\n", "```json ", "```\n", "``` "] {
        if let Some(start) = text.find(fence) {
            let after = &text[start + fence.len()..];
            if let Some(end) = after.find("```") {
                let candidate = after[..end].trim();
                if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
                    return Some(candidate.to_string());
                }
            }
        }
    }

    // 3. Find first '{' and scan for matching '}'
    let start = text.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let chars: Vec<char> = text[start..].chars().collect();
    let mut end = None;
    for (i, &c) in chars.iter().enumerate() {
        if escape { escape = false; continue; }
        if c == '\\' && in_string { escape = true; continue; }
        if c == '"' { in_string = !in_string; continue; }
        if in_string { continue; }
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 { end = Some(i); break; }
            }
            _ => {}
        }
    }
    let end = end?;
    let candidate: String = chars[..=end].iter().collect();
    if serde_json::from_str::<serde_json::Value>(&candidate).is_ok() {
        return Some(candidate);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_bare_json() {
        let s = r#"{"sql":"SELECT 1","explanation":"test"}"#;
        assert!(extract_json_object(s).is_some());
    }

    #[test]
    fn extracts_json_from_fenced_block() {
        let s = "Here is the query:\n```json\n{\"sql\":\"SELECT 1\",\"explanation\":\"test\"}\n```\nDone.";
        assert!(extract_json_object(s).is_some());
    }

    #[test]
    fn extracts_json_embedded_in_prose() {
        let s = "Sure! {\"sql\":\"SELECT 1\",\"explanation\":\"test\"} is what you need.";
        assert!(extract_json_object(s).is_some());
    }

    #[test]
    fn returns_none_for_plain_text() {
        assert!(extract_json_object("No JSON here at all.").is_none());
    }
}
