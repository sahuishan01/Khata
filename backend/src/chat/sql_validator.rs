use anyhow::{bail, Result};
use sqlparser::{dialect::PostgreSqlDialect, parser::Parser, ast::{Statement, SetExpr, TableFactor, Query}};

/// Tables the LLM is allowed to query (read-only role).
const ALLOWED_TABLES: &[&str] = &[
    "transactions",
    "statements",
    "chat_messages",
    "user_accounts",
];

pub fn validate_select_sql(sql: &str) -> Result<()> {
    let dialect = PostgreSqlDialect {};
    let stmts = Parser::parse_sql(&dialect, sql)
        .map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

    if stmts.len() != 1 {
        bail!("exactly one statement required, got {}", stmts.len());
    }

    match &stmts[0] {
        Statement::Query(q) => validate_query(q, &mut Vec::new())?,
        _ => bail!("only SELECT queries are allowed"),
    }

    // Keyword blocklist — catches DML/DDL that might slip inside CTEs or subqueries
    let lower = sql.to_lowercase();
    for kw in &[
        "insert ", "update ", "delete ", "drop ", "truncate ", "alter ",
        "create ", "grant ", "revoke ", "execute ", "call ", "copy ",
        "pg_sleep", "pg_read_file", "pg_write_file", "pg_terminate", "pg_cancel",
        "set_config", "current_setting",
    ] {
        if lower.contains(kw) {
            bail!("disallowed keyword: {}", kw.trim());
        }
    }

    // Block information_schema and pg_catalog table access
    if lower.contains("information_schema") || lower.contains("pg_catalog") {
        bail!("access to system schemas is not allowed");
    }

    // Block any semicolons inside string literals
    let mut in_string = false;
    let mut escape = false;
    for c in sql.chars() {
        if escape { escape = false; continue; }
        if c == '\\' { escape = true; continue; }
        if c == '\'' { in_string = !in_string; continue; }
        if in_string && c == ';' {
            bail!("semicolons inside string content are not allowed");
        }
    }

    Ok(())
}

/// Recursively validate that all tables in the query are in the allowlist.
/// `cte_names` tracks CTE aliases defined in the current scope (these are allowed).
fn validate_query(query: &Query, cte_names: &mut Vec<String>) -> Result<()> {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            // Register CTE alias before validating its body (allows self-referencing CTEs)
            cte_names.push(cte.alias.name.to_string());
            validate_query(&cte.query, cte_names)?;
        }
    }
    validate_set_expr(&query.body, cte_names)
}

fn validate_set_expr(expr: &SetExpr, cte_names: &[String]) -> Result<()> {
    match expr {
        SetExpr::Select(select) => {
            for table in &select.from {
                validate_table_factor(&table.relation, cte_names)?;
                for join in &table.joins {
                    validate_table_factor(&join.relation, cte_names)?;
                }
            }
        }
        SetExpr::Query(q) => validate_query(q, &mut cte_names.to_vec())?,
        SetExpr::SetOperation { left, right, .. } => {
            validate_set_expr(left, cte_names)?;
            validate_set_expr(right, cte_names)?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_table_factor(factor: &TableFactor, cte_names: &[String]) -> Result<()> {
    match factor {
        TableFactor::Table { name, .. } => {
            let table_name = name.to_string().to_lowercase();
            // Strip schema prefix if present (e.g. "public.transactions")
            let bare = table_name.split('.').last().unwrap_or(&table_name);
            // Skip CTE aliases — they're not real tables
            if cte_names.iter().any(|c| c.eq_ignore_ascii_case(bare)) {
                return Ok(());
            }
            if !ALLOWED_TABLES.contains(&bare) {
                bail!("table '{}' is not in the allowed list", bare);
            }
        }
        TableFactor::Derived { subquery, .. } => validate_query(subquery, &mut cte_names.to_vec())?,
        TableFactor::TableFunction { .. } => bail!("table functions are not allowed"),
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_select() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions WHERE direction = 'debit'"
        ).is_ok());
    }

    #[test]
    fn rejects_drop() {
        assert!(validate_select_sql("DROP TABLE transactions").is_err());
    }

    #[test]
    fn rejects_multi_statement() {
        assert!(validate_select_sql("SELECT 1; DROP TABLE users").is_err());
    }

    #[test]
    fn rejects_insert() {
        assert!(validate_select_sql("INSERT INTO users VALUES ('a','b','c')").is_err());
    }

    #[test]
    fn rejects_delete_keyword() {
        assert!(validate_select_sql(
            "SELECT * FROM transactions WHERE description LIKE '%delete %'"
        ).is_err());
    }

    #[test]
    fn accepts_aggregate() {
        assert!(validate_select_sql(
            "SELECT SUM(amount), direction FROM transactions GROUP BY direction"
        ).is_ok());
    }

    #[test]
    fn rejects_set_config() {
        assert!(validate_select_sql(
            "SELECT set_config('app.current_user_id', '11111111-1111-1111-1111-111111111111', true)"
        ).is_err());
    }

    #[test]
    fn rejects_disallowed_table() {
        assert!(validate_select_sql(
            "SELECT * FROM users"
        ).is_err());
    }

    #[test]
    fn rejects_disallowed_table_in_subquery() {
        assert!(validate_select_sql(
            "SELECT * FROM (SELECT * FROM users) u"
        ).is_err());
    }

    #[test]
    fn accepts_cte_with_allowed_table() {
        assert!(validate_select_sql(
            "WITH totals AS (SELECT SUM(amount) AS t FROM transactions) SELECT t FROM totals"
        ).is_ok());
    }

    #[test]
    fn rejects_cte_with_disallowed_table() {
        assert!(validate_select_sql(
            "WITH hacked AS (SELECT * FROM users) SELECT * FROM hacked"
        ).is_err());
    }

    #[test]
    fn accepts_user_accounts_query() {
        assert!(validate_select_sql(
            "SELECT label, identifier FROM user_accounts"
        ).is_ok());
    }

    #[test]
    fn rejects_join_to_disallowed_table() {
        assert!(validate_select_sql(
            "SELECT t.amount FROM transactions t JOIN users u ON t.user_id = u.id"
        ).is_err());
    }

    #[test]
    fn accepts_join_to_allowed_table() {
        assert!(validate_select_sql(
            "SELECT t.amount, a.label FROM transactions t JOIN user_accounts a ON t.bank = a.identifier"
        ).is_ok());
    }
}
