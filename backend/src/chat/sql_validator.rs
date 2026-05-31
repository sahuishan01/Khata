use anyhow::{bail, Result};
use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};

pub fn validate_select_sql(sql: &str) -> Result<()> {
    let dialect = PostgreSqlDialect {};
    let stmts = Parser::parse_sql(&dialect, sql)
        .map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

    if stmts.len() != 1 {
        bail!("exactly one statement required, got {}", stmts.len());
    }

    match &stmts[0] {
        sqlparser::ast::Statement::Query(_) => {}
        _ => bail!("only SELECT queries are allowed"),
    }

    // Keyword blocklist — catches DML/DDL that might slip inside CTEs or subqueries
    let lower = sql.to_lowercase();
    for kw in &[
        "insert ", "update ", "delete ", "drop ", "truncate ", "alter ",
        "create ", "grant ", "revoke ", "execute ", "call ", "copy ",
    ] {
        if lower.contains(kw) {
            bail!("disallowed keyword: {}", kw.trim());
        }
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
        ).is_err()); // keyword in string still blocked — conservative
    }

    #[test]
    fn accepts_aggregate() {
        assert!(validate_select_sql(
            "SELECT SUM(amount), direction FROM transactions GROUP BY direction"
        ).is_ok());
    }
}
