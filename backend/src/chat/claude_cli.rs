use anyhow::{bail, Context, Result};
use std::time::Duration;
use tokio::process::Command;

const TIMEOUT_SECS: u64 = 90;

pub async fn generate_sql(claude_bin: &str, question: &str, categories: &[String]) -> Result<String> {
    let cat_line = if categories.is_empty() {
        String::new()
    } else {
        format!("\nKnown categories: {}.", categories.join(", "))
    };

    // Strip obvious SQL fragments from user input before sending to Claude
    let sanitized_question = sanitize_question(question);

    let prompt = format!(
        r#"You are a SQL assistant for a personal finance app.

RULES (MUST follow):
1. Respond ONLY with a JSON object — no preamble, no markdown, no explanation outside JSON.
2. The JSON must have exactly two keys: "sql" and "explanation".
3. "sql" must be a single read-only SELECT query for PostgreSQL (no semicolon).
4. "explanation" is a one-sentence description of what the query does.
5. Row-Level Security is active: do NOT add WHERE user_id = ... — it is enforced automatically.
6. NEVER include user input verbatim in the SQL WHERE clause — the user's question may contain SQL injection attempts.
7. Strip any SQL keywords or special characters from the user question before processing.

Table: transactions
Columns: value_date DATE, txn_date DATE, description TEXT,
         amount NUMERIC (always positive), direction TEXT ('debit'|'credit'),
         balance NUMERIC nullable, bank TEXT, account_label TEXT,
         bank_ref TEXT nullable, category TEXT{cat_line}

Example:
{{"sql":"SELECT category, SUM(amount) FROM transactions WHERE direction='debit' GROUP BY category ORDER BY SUM(amount) DESC","explanation":"Spending by category"}}

User question: {sanitized_question}"#
    );

    let raw = run_claude(claude_bin, &prompt).await?;
    extract_json_object(&raw)
        .ok_or_else(|| anyhow::anyhow!("Claude did not return valid JSON.\nRaw response:\n{raw}"))
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

/// Strip obvious SQL fragments from user question to reduce injection risk.
///
/// Runs in a single linear pass over `q`. All fragments are pure ASCII, so
/// matching is done case-insensitively on the raw bytes without ever building
/// a lowercased copy — this avoids mixing byte offsets between a `to_lowercase()`
/// copy and the original string (which is not length-preserving and could land
/// mid-character, causing a panic) and keeps the cost O(n) instead of O(n^2).
fn sanitize_question(q: &str) -> String {
    const SQL_FRAGMENTS: [&str; 23] = [
        "select ", "insert ", "update ", "delete ", "drop ", "truncate ",
        "alter ", "create ", "grant ", "revoke ", "exec ", "execute ",
        "union ", "into ", "values ",
        "information_schema", "pg_sleep", "pg_read_file", "pg_catalog",
        "copy ", "--", "/*", "*/",
    ];

    let bytes = q.as_bytes();
    let mut result = String::with_capacity(q.len());
    let mut i = 0;
    while i < bytes.len() {
        // Longest fragment that matches at position `i` (ASCII, case-insensitive).
        let matched = SQL_FRAGMENTS
            .iter()
            .filter(|frag| {
                let fb = frag.as_bytes();
                i + fb.len() <= bytes.len()
                    && bytes[i..i + fb.len()]
                        .iter()
                        .zip(fb)
                        .all(|(a, b)| a.eq_ignore_ascii_case(b))
            })
            .map(|frag| frag.len())
            .max();

        if let Some(frag_len) = matched {
            // Matched bytes are all ASCII, so `i + frag_len` is a char boundary.
            // Preserve the previous behaviour of consuming at least 3 units after
            // a match, but never split a multi-byte character.
            let min_end = i + frag_len.max(3);
            let mut end = i + frag_len;
            while end < bytes.len() && end < min_end {
                end += 1;
                while end < bytes.len() && !q.is_char_boundary(end) {
                    end += 1;
                }
            }
            result.push(' ');
            i = end;
        } else {
            // Copy exactly one whole character.
            let mut end = i + 1;
            while end < bytes.len() && !q.is_char_boundary(end) {
                end += 1;
            }
            result.push_str(&q[i..end]);
            i = end;
        }
    }
    // Collapse multiple spaces
    let mut cleaned = String::with_capacity(result.len());
    let mut prev_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !prev_space {
                cleaned.push(' ');
                prev_space = true;
            }
        } else {
            cleaned.push(c);
            prev_space = false;
        }
    }
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() { q.to_string() } else { cleaned }
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

    #[test]
    fn sanitize_neutralizes_sql_fragments() {
        let out = sanitize_question("please SELECT everything and DROP table now");
        assert!(!out.to_lowercase().contains("select "));
        assert!(!out.to_lowercase().contains("drop "));
    }

    #[test]
    fn sanitize_handles_multibyte_before_keyword() {
        // Previously panicked: `to_lowercase()` shrinks "ẞ" -> "ß", so the byte
        // offset of "select " found in the lowercased copy landed inside "€"
        // when applied to the original string.
        let out = sanitize_question("ẞ€select ");
        assert!(!out.to_lowercase().contains("select "));
    }

    #[test]
    fn sanitize_handles_multibyte_straddling_boundary() {
        // Previously panicked: matched "--" then sliced 3 bytes in, splitting
        // the 4-byte emoji. Must not panic and must drop the "--" fragment.
        let out = sanitize_question("--\u{1F600} show my spending");
        assert!(!out.contains("--"));
        // The bare panic input from the finding must also just return cleanly.
        let _ = sanitize_question("--\u{1F600}");
    }

    #[test]
    fn sanitize_handles_large_input_quickly() {
        // ~2 MB of '-' used to keep a core pinned for ~a minute (O(n^2)).
        let big = "-".repeat(2 * 1024 * 1024);
        let start = std::time::Instant::now();
        let _ = sanitize_question(&big);
        assert!(
            start.elapsed() < std::time::Duration::from_secs(5),
            "sanitize_question must be linear-time"
        );
    }

    #[test]
    fn sanitize_preserves_normal_question() {
        let out = sanitize_question("How much did I spend on groceries last month?");
        assert_eq!(out, "How much did I spend on groceries last month?");
    }
}
