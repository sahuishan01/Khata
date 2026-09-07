use anyhow::{bail, Result};
use core::ops::ControlFlow;
use sqlparser::{
    ast::{ObjectName, Query, SetExpr, Statement, TableFactor, Visit, Visitor},
    dialect::PostgreSqlDialect,
    parser::Parser,
};

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
        Statement::Query(q) => {
            let mut checker = QueryChecker::default();
            if let Err(msg) = checker.check_query(q) {
                bail!("{msg}");
            }
        }
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

/// Recursively enforces the [`ALLOWED_TABLES`] allowlist over the whole parsed
/// query while honouring SQL name-resolution scope.
///
/// Query *structure* is walked by hand — CTEs (`WITH`), set operations
/// (`UNION` / `EXCEPT` / `INTERSECT`), the `TABLE <name>` shorthand and
/// `VALUES` — so that:
/// * a non-recursive CTE's own name is **not** in scope while its own body is
///   checked (it binds to the real relation, exactly like PostgreSQL); a
///   `WITH RECURSIVE` CTE's name *is* in scope for its body;
/// * only previously-defined sibling CTEs (plus outer scopes) are visible to a
///   given CTE body;
/// * bare `TABLE <name>` / `INSERT` / `UPDATE` bodies are rejected outright.
///
/// Within a single query level every *expression* position — projection,
/// `WHERE`, `GROUP BY`, `HAVING`, `QUALIFY`, `PREWHERE`, `CLUSTER/DISTRIBUTE/
/// SORT BY`, `CONNECT BY`, window (`OVER` / `WINDOW`) definitions, `JOIN ... ON`
/// constraints, `CASE`, function arguments (incl. bare-subquery args), casts,
/// subscripts, array/tuple/struct literals, `ORDER BY` / `LIMIT` / `OFFSET` /
/// `FETCH` — is walked by `sqlparser`'s derived [`Visit`] implementation
/// (the `visitor` feature) via [`LevelCollector`], which reliably reaches every
/// nested subquery and table reference regardless of nesting depth. Each nested
/// subquery is then re-checked through [`QueryChecker::check_query`] so it gets
/// its own correct CTE scope.
#[derive(Default)]
struct QueryChecker {
    /// One frame of in-scope CTE names per enclosing query level.
    scopes: Vec<Vec<String>>,
}

impl QueryChecker {
    fn cte_in_scope(&self, name: &str) -> bool {
        self.scopes.iter().flatten().any(|c| c.eq_ignore_ascii_case(name))
    }

    /// Enforce the allowlist on a single (already schema-stripped, unquoted)
    /// table name, case-insensitively, unless it resolves to an in-scope CTE.
    fn check_relation(&self, raw: &str) -> Result<(), String> {
        let lname = raw.to_lowercase();
        // Defensive: strip a schema qualifier if one slipped through.
        let bare = lname.rsplit('.').next().unwrap_or(&lname);
        if self.cte_in_scope(bare) {
            return Ok(());
        }
        if !ALLOWED_TABLES.contains(&bare) {
            return Err(format!("table '{bare}' is not in the allowed list"));
        }
        Ok(())
    }

    fn check_query(&mut self, query: &Query) -> Result<(), String> {
        let recursive = query.with.as_ref().map(|w| w.recursive).unwrap_or(false);

        // CTEs: check each body under the scope actually visible to it, then
        // expose that CTE name to later siblings and to the outer body.
        let mut cte_names: Vec<String> = Vec::new();
        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                // `.value` is the identifier without any surrounding quotes.
                let alias = cte.alias.name.value.clone();
                // A CTE must not shadow a real base table in the allowlist:
                // that would let a reference to e.g. `transactions` silently
                // resolve to attacker-authored CTE SQL.
                if ALLOWED_TABLES.iter().any(|t| t.eq_ignore_ascii_case(&alias)) {
                    return Err(format!(
                        "CTE name '{alias}' collides with a base table in the allowlist"
                    ));
                }
                let mut body_scope = cte_names.clone();
                if recursive {
                    body_scope.push(alias.clone());
                }
                self.scopes.push(body_scope);
                let r = self.check_query(&cte.query);
                self.scopes.pop();
                r?;
                cte_names.push(alias);
            }
        }

        self.scopes.push(cte_names);
        let result = (|| {
            self.check_set_expr(&query.body)?;
            // Tail clauses that can embed scalar subqueries.
            self.scan(&query.order_by)?;
            self.scan(&query.limit)?;
            self.scan(&query.limit_by)?;
            self.scan(&query.offset)?;
            self.scan(&query.fetch)?;
            Ok(())
        })();
        self.scopes.pop();
        result
    }

    fn check_set_expr(&mut self, body: &SetExpr) -> Result<(), String> {
        match body {
            SetExpr::Select(select) => self.scan(select.as_ref()),
            SetExpr::Query(q) => self.check_query(q),
            SetExpr::SetOperation { left, right, .. } => {
                self.check_set_expr(left)?;
                self.check_set_expr(right)
            }
            SetExpr::Values(values) => self.scan(values),
            // The text-to-SQL path only ever needs SELECT. `TABLE <name>` reads
            // a relation without ever surfacing as a `TableFactor`, so the
            // allowlist visitor never sees it — reject it here.
            SetExpr::Table(_) => Err("TABLE <name> syntax is not allowed".to_string()),
            SetExpr::Insert(_) | SetExpr::Update(_) => {
                Err("only SELECT queries are allowed".to_string())
            }
        }
    }

    /// Walk one AST node with the derived visitor: enforce the allowlist on
    /// every table reference *at this query level*, reject table-function
    /// factors, and recurse into every directly-nested subquery under its own
    /// CTE scope.
    fn scan<T: Visit>(&mut self, node: &T) -> Result<(), String> {
        let mut collector = LevelCollector::default();
        // `LevelCollector` never breaks the walk.
        let _ = node.visit(&mut collector);

        if let Some(msg) = collector.bad_factor {
            return Err(msg);
        }
        for name in &collector.level_relations {
            self.check_relation(name)?;
        }
        for nested in &collector.nested {
            self.check_query(nested)?;
        }
        Ok(())
    }
}

/// Collects, for a single query level, the table references and directly-nested
/// subqueries reachable by `sqlparser`'s derived AST walk. Depth tracking keeps
/// deeper subqueries opaque so they are validated separately (with their own
/// scope) rather than against the current level's scope.
#[derive(Default)]
struct LevelCollector {
    /// 0 == current query level; >0 == inside a nested subquery.
    depth: i32,
    level_relations: Vec<String>,
    nested: Vec<Query>,
    bad_factor: Option<String>,
}

impl Visitor for LevelCollector {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.depth += 1;
        if self.depth == 1 {
            self.nested.push(query.clone());
        }
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _query: &Query) -> ControlFlow<()> {
        self.depth -= 1;
        ControlFlow::Continue(())
    }

    fn pre_visit_relation(&mut self, name: &ObjectName) -> ControlFlow<()> {
        if self.depth == 0 {
            // Last part = the table itself; `.value` drops any quoting.
            let bare = name
                .0
                .last()
                .and_then(|p| p.as_ident())
                .map(|i| i.value.clone())
                .unwrap_or_else(|| name.to_string());
            self.level_relations.push(bare);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        if self.depth == 0
            && matches!(
                factor,
                TableFactor::TableFunction { .. } | TableFactor::Function { .. }
            )
        {
            self.bad_factor
                .get_or_insert_with(|| "table functions are not allowed".to_string());
        }
        ControlFlow::Continue(())
    }
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

    // --- Subquery-position allowlist enforcement (finding F13) ---

    #[test]
    fn rejects_catalog_subquery_in_projection() {
        // Outer FROM is `transactions` (allowed); the leak rides in a scalar
        // subquery in the SELECT projection.
        assert!(validate_select_sql(
            "SELECT amount, (SELECT string_agg(name, ',') FROM pg_settings) AS diag \
             FROM transactions LIMIT 1"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_subquery_in_where() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions \
             WHERE amount > (SELECT reltuples FROM pg_class LIMIT 1)"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_subquery_in_in_clause() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions \
             WHERE direction IN (SELECT usename FROM pg_stat_activity)"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_subquery_in_having() {
        assert!(validate_select_sql(
            "SELECT direction, SUM(amount) FROM transactions GROUP BY direction \
             HAVING SUM(amount) > (SELECT count(*) FROM pg_stat_activity)"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_subquery_in_group_by() {
        assert!(validate_select_sql(
            "SELECT SUM(amount) FROM transactions \
             GROUP BY (SELECT count(*) FROM pg_class)"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_subquery_in_order_by() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions \
             ORDER BY (SELECT count(*) FROM pg_class)"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_exists_subquery() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions \
             WHERE EXISTS (SELECT 1 FROM pg_roles WHERE rolsuper)"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_subquery_in_function_arg() {
        assert!(validate_select_sql(
            "SELECT COALESCE((SELECT setting FROM pg_settings LIMIT 1), amount) \
             FROM transactions"
        ).is_err());
    }

    #[test]
    fn rejects_nested_cte_reading_disallowed_table() {
        assert!(validate_select_sql(
            "WITH a AS (SELECT * FROM (SELECT usename FROM pg_stat_activity) z) \
             SELECT * FROM a"
        ).is_err());
    }

    #[test]
    fn rejects_cte_alias_shadowing_transactions() {
        assert!(validate_select_sql(
            "WITH transactions AS (SELECT 1 AS amount) SELECT amount FROM transactions"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_in_nested_join() {
        assert!(validate_select_sql(
            "SELECT t.amount FROM (transactions t JOIN pg_class c ON true)"
        ).is_err());
    }

    #[test]
    fn rejects_catalog_subquery_in_set_operation() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions UNION SELECT reltuples FROM pg_class"
        ).is_err());
    }

    // --- Legitimate queries that must keep working ---

    #[test]
    fn accepts_scalar_subquery_over_allowed_table() {
        assert!(validate_select_sql(
            "SELECT amount, (SELECT SUM(amount) FROM transactions) AS total FROM transactions"
        ).is_ok());
    }

    #[test]
    fn accepts_where_subquery_over_allowed_table() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions \
             WHERE bank IN (SELECT identifier FROM user_accounts)"
        ).is_ok());
    }

    #[test]
    fn accepts_nested_cte_over_allowed_tables() {
        assert!(validate_select_sql(
            "WITH a AS (SELECT amount FROM transactions), \
                  b AS (SELECT SUM(amount) AS s FROM a) \
             SELECT s FROM b"
        ).is_ok());
    }

    // --- Revision round: `TABLE <name>` shorthand, CTE self-reference scope,
    //     and `TableFactor::Function` ---

    #[test]
    fn rejects_bare_table_statement() {
        assert!(validate_select_sql("(TABLE pg_stat_activity)").is_err());
    }

    #[test]
    fn rejects_bare_table_after_cte() {
        assert!(validate_select_sql(
            "WITH d AS (SELECT 1) TABLE pg_settings"
        ).is_err());
    }

    #[test]
    fn rejects_bare_table_in_derived() {
        assert!(validate_select_sql(
            "SELECT * FROM (TABLE pg_settings) x"
        ).is_err());
    }

    #[test]
    fn rejects_bare_table_in_set_operation() {
        assert!(validate_select_sql(
            "SELECT amount FROM transactions UNION TABLE pg_settings"
        ).is_err());
    }

    #[test]
    fn rejects_cte_named_after_catalog_reading_itself() {
        // Non-recursive CTE: `pg_settings` inside its own body binds to the real
        // relation, not the CTE, so this must be rejected.
        assert!(validate_select_sql(
            "WITH pg_settings AS (SELECT name FROM pg_settings) SELECT * FROM pg_settings"
        ).is_err());
    }

    #[test]
    fn rejects_cte_named_pg_class_reading_itself() {
        assert!(validate_select_sql(
            "WITH pg_class AS (SELECT relname FROM pg_class) SELECT * FROM pg_class"
        ).is_err());
    }

    #[test]
    fn rejects_cte_reading_later_sibling() {
        // `a` references `b`, defined after it: not in scope for `a`'s body.
        assert!(validate_select_sql(
            "WITH a AS (SELECT * FROM b), b AS (SELECT 1) SELECT * FROM a"
        ).is_err());
    }

    #[test]
    fn rejects_lateral_function_table_factor() {
        assert!(validate_select_sql("SELECT * FROM LATERAL foo()").is_err());
    }

    #[test]
    fn rejects_quoted_cte_named_after_catalog_reading_itself() {
        assert!(validate_select_sql(
            "WITH \"pg_settings\" AS (SELECT name FROM \"pg_settings\") SELECT * FROM \"pg_settings\""
        ).is_err());
    }

    #[test]
    fn rejects_quoted_cte_alias_shadowing_transactions() {
        assert!(validate_select_sql(
            "WITH \"transactions\" AS (SELECT amount FROM transactions) SELECT * FROM \"transactions\""
        ).is_err());
    }

    #[test]
    fn accepts_recursive_cte_self_reference() {
        // `WITH RECURSIVE`: the CTE's own name IS in scope for its body.
        assert!(validate_select_sql(
            "WITH RECURSIVE t AS (\
                 SELECT 1 AS n \
                 UNION ALL \
                 SELECT n + 1 FROM t WHERE n < 5\
             ) SELECT * FROM t"
        ).is_ok());
    }

    #[test]
    fn accepts_recursive_cte_over_allowed_base_table() {
        assert!(validate_select_sql(
            "WITH RECURSIVE chain AS (\
                 SELECT id, amount FROM transactions WHERE amount > 0 \
                 UNION ALL \
                 SELECT t.id, t.amount FROM transactions t JOIN chain c ON t.id = c.id\
             ) SELECT * FROM chain"
        ).is_ok());
    }
}
