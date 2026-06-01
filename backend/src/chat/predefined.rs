use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryKind {
    SpendPeriod,
    EarnPeriod,
    SavingsRate,
    NetBalance,
    TopExpenses,
    LargestExpense,
    TransactionCount,
    MonthlyBreakdown,
    CategorySpend,
    SameMonthYears,   // current month spending across past N years
}

pub struct Predefined {
    pub kind:  QueryKind,
    pub sql:   String,
    pub label: String,
}

fn any(q: &str, words: &[&str]) -> bool {
    words.iter().any(|w| q.contains(w))
}

/// Returns (WHERE date clause, human label like "this month")
fn time_clause(q: &str) -> (String, String) {
    if any(q, &["this month", "current month"]) {
        (
            "AND date_trunc('month', value_date) = date_trunc('month', CURRENT_DATE)".into(),
            "this month".into(),
        )
    } else if any(q, &["last month", "previous month", "past month"]) {
        (
            "AND date_trunc('month', value_date) = date_trunc('month', CURRENT_DATE - INTERVAL '1 month')".into(),
            "last month".into(),
        )
    } else if any(q, &["this year", "current year"]) {
        (
            "AND EXTRACT(year FROM value_date) = EXTRACT(year FROM CURRENT_DATE)".into(),
            "this year".into(),
        )
    } else if any(q, &["last year", "previous year"]) {
        (
            "AND EXTRACT(year FROM value_date) = EXTRACT(year FROM CURRENT_DATE) - 1".into(),
            "last year".into(),
        )
    } else if any(q, &["this week", "current week"]) {
        (
            "AND value_date >= date_trunc('week', CURRENT_DATE)".into(),
            "this week".into(),
        )
    } else if q.contains("today") {
        ("AND value_date = CURRENT_DATE".into(), "today".into())
    } else {
        (String::new(), "in total".into())
    }
}

/// Try to match a natural-language question to a predefined query.
/// `categories` is the user's actual category list (used for category-spend matching).
pub fn match_question(q: &str, categories: &[String]) -> Option<Predefined> {
    let lq = q.to_lowercase();
    let q  = lq.as_str();

    // ── Category spending ─────────────────────────────────────────────────────
    // Check before generic spend so "food spending" routes to CategorySpend.
    // Match if any significant word of the category appears in the question.
    for cat in categories {
        let cat_lower = cat.to_lowercase();
        let cat_words: Vec<&str> = cat_lower
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| w.len() > 3)
            .collect();
        let cat_matches = q.contains(cat_lower.as_str())
            || cat_words.iter().any(|w| q.contains(*w));

        if cat_matches && any(q, &["spend", "spent", "expense", "pay", "paid", "cost", "total", "how much"]) {
            let (date_clause, time_label) = time_clause(q);
            let safe = cat.replace('\'', "''");
            let label = if time_label == "in total" {
                format!("on {}", cat)
            } else {
                format!("on {} {}", cat, time_label)
            };
            return Some(Predefined {
                kind: QueryKind::CategorySpend,
                sql: format!(
                    r#"SELECT COALESCE(SUM(amount), 0)::float8 AS amount, COUNT(*)::bigint AS txn_count
                       FROM transactions
                       WHERE direction = 'debit' AND LOWER(category) = LOWER('{}') {}"#,
                    safe, date_clause
                ),
                label,
            });
        }
    }

    // ── Year-over-year same-month comparison ──────────────────────────────────
    // Must sit before generic SpendPeriod so "compared to previous financial year"
    // doesn't fall through to the all-time total.
    let is_yoy = any(q, &["year over year", "yoy", "year on year"])
        || (any(q, &["compare", "compared", "comparison", "versus", " vs "])
            && any(q, &["year", "financial year", "fy", "annual"]))
        || (any(q, &["same month"])
            && any(q, &["last year", "previous year", "financial year", "fy"]))
        || (any(q, &["previous financial year", "last financial year", "last fy", "prev fy"])
            && any(q, &["month", "spend", "spent", "expense"]));
    if is_yoy {
        return Some(Predefined {
            kind: QueryKind::SameMonthYears,
            sql:  r#"SELECT
                       to_char(value_date, 'Mon YYYY') AS period,
                       date_trunc('month', value_date) AS sort_key,
                       SUM(amount)::float8              AS spent,
                       COUNT(*)::bigint                 AS txn_count
                     FROM transactions
                     WHERE direction = 'debit'
                       AND EXTRACT(month FROM value_date) = EXTRACT(month FROM CURRENT_DATE)
                     GROUP BY to_char(value_date, 'Mon YYYY'), date_trunc('month', value_date)
                     ORDER BY sort_key DESC
                     LIMIT 5"#
                .to_string(),
            label: String::new(),
        });
    }

    // ── Savings rate ──────────────────────────────────────────────────────────
    if any(q, &["saving"]) && any(q, &["rate", "%", "percent", "ratio"]) {
        return Some(Predefined {
            kind: QueryKind::SavingsRate,
            sql:  r#"SELECT
                       COALESCE(SUM(amount) FILTER (WHERE direction='debit'),  0)::float8 AS spent,
                       COALESCE(SUM(amount) FILTER (WHERE direction='credit'), 0)::float8 AS earned
                     FROM transactions"#
                .to_string(),
            label: String::new(),
        });
    }

    // ── Net balance ───────────────────────────────────────────────────────────
    if (q.contains("net") && any(q, &["balance", "worth", "total", "amount", "money"]))
        || q.contains("overall balance")
    {
        return Some(Predefined {
            kind: QueryKind::NetBalance,
            sql:  r#"SELECT
                       COALESCE(SUM(amount) FILTER (WHERE direction='debit'),  0)::float8 AS spent,
                       COALESCE(SUM(amount) FILTER (WHERE direction='credit'), 0)::float8 AS earned
                     FROM transactions"#
                .to_string(),
            label: String::new(),
        });
    }

    // ── Monthly breakdown ─────────────────────────────────────────────────────
    if any(q, &["monthly", "each month", "month by month", "month wise", "per month", "by month"]) {
        return Some(Predefined {
            kind: QueryKind::MonthlyBreakdown,
            sql:  r#"SELECT to_char(value_date, 'Mon YYYY') AS month,
                            COALESCE(SUM(amount) FILTER (WHERE direction='debit'),  0)::float8 AS spent,
                            COALESCE(SUM(amount) FILTER (WHERE direction='credit'), 0)::float8 AS earned
                     FROM transactions
                     GROUP BY to_char(value_date, 'Mon YYYY'), date_trunc('month', value_date)
                     ORDER BY date_trunc('month', value_date) DESC
                     LIMIT 12"#
                .to_string(),
            label: String::new(),
        });
    }

    // ── Top expenses ──────────────────────────────────────────────────────────
    if any(q, &["top", "biggest", "highest", "most expensive"])
        && any(q, &["spend", "spent", "expense", "merchant", "transaction", "payment", "purchase"])
    {
        return Some(Predefined {
            kind: QueryKind::TopExpenses,
            sql:  r#"SELECT description, SUM(amount)::float8 AS total, COUNT(*)::bigint AS txn_count
                     FROM transactions WHERE direction = 'debit'
                     GROUP BY description ORDER BY total DESC LIMIT 10"#
                .to_string(),
            label: String::new(),
        });
    }

    // ── Largest single expense ────────────────────────────────────────────────
    if any(q, &["largest", "biggest", "maximum", "most expensive single"])
        && any(q, &["expense", "transaction", "payment", "purchase", "single"])
        && !any(q, &["top", "list", "merchants"])
    {
        return Some(Predefined {
            kind: QueryKind::LargestExpense,
            sql:  r#"SELECT value_date::text, description, amount::float8
                     FROM transactions WHERE direction = 'debit'
                     ORDER BY amount DESC LIMIT 1"#
                .to_string(),
            label: String::new(),
        });
    }

    // ── Transaction count ─────────────────────────────────────────────────────
    if any(q, &["how many", "count", "number of", "total number"])
        && any(q, &["transaction", "txn", "payment", "entry", "entries"])
    {
        return Some(Predefined {
            kind: QueryKind::TransactionCount,
            sql:  "SELECT COUNT(*)::bigint AS total_count FROM transactions".to_string(),
            label: String::new(),
        });
    }

    // ── Income / credits ──────────────────────────────────────────────────────
    if any(q, &["earn", "earned", "income", "receive", "received", "salary", "credit total", "total credit"]) {
        let (date_clause, label) = time_clause(q);
        return Some(Predefined {
            kind: QueryKind::EarnPeriod,
            sql:  format!(
                r#"SELECT COALESCE(SUM(amount), 0)::float8 AS amount, COUNT(*)::bigint AS txn_count
                   FROM transactions WHERE direction = 'credit' {}"#,
                date_clause
            ),
            label,
        });
    }

    // ── Spending / debits ─────────────────────────────────────────────────────
    if any(q, &["spend", "spent", "expense", "expend", "pay", "paid", "debit total", "total debit"]) {
        let (date_clause, label) = time_clause(q);
        return Some(Predefined {
            kind: QueryKind::SpendPeriod,
            sql:  format!(
                r#"SELECT COALESCE(SUM(amount), 0)::float8 AS amount, COUNT(*)::bigint AS txn_count
                   FROM transactions WHERE direction = 'debit' {}"#,
                date_clause
            ),
            label,
        });
    }

    None
}

// ── Answer formatters ─────────────────────────────────────────────────────────

pub fn format_predefined(kind: QueryKind, rows: &[Value], label: &str) -> String {
    match kind {
        QueryKind::SpendPeriod => {
            let amount = first_f64(rows, "amount");
            let count  = first_i64(rows, "txn_count");
            if amount == 0.0 {
                format!("No spending recorded {}.", label)
            } else {
                format!(
                    "You spent {} {} across {} transaction{}.",
                    inr(amount), label, count, s(count)
                )
            }
        }
        QueryKind::EarnPeriod => {
            let amount = first_f64(rows, "amount");
            let count  = first_i64(rows, "txn_count");
            if amount == 0.0 {
                format!("No income recorded {}.", label)
            } else {
                format!(
                    "You received {} {} across {} credit transaction{}.",
                    inr(amount), label, count, s(count)
                )
            }
        }
        QueryKind::SavingsRate => {
            let spent  = first_f64(rows, "spent");
            let earned = first_f64(rows, "earned");
            let rate   = if earned > 0.0 { (earned - spent).max(0.0) / earned * 100.0 } else { 0.0 };
            let saved  = (earned - spent).max(0.0);
            format!(
                "Your savings rate is {:.1}% — earned {}, spent {}, saved {}.",
                rate, inr(earned), inr(spent), inr(saved)
            )
        }
        QueryKind::NetBalance => {
            let spent  = first_f64(rows, "spent");
            let earned = first_f64(rows, "earned");
            let net    = earned - spent;
            if net >= 0.0 {
                format!("Net: +{} (earned {} − spent {}).", inr(net), inr(earned), inr(spent))
            } else {
                format!("Net: −{} (earned {} − spent {}).", inr(-net), inr(earned), inr(spent))
            }
        }
        QueryKind::TopExpenses => {
            if rows.is_empty() { return "No expense data found.".to_string(); }
            let mut lines = vec!["Your top spending merchants:".to_string()];
            for (i, row) in rows.iter().enumerate() {
                let desc  = row["description"].as_str().unwrap_or("Unknown");
                let total = row["total"].as_f64().unwrap_or(0.0);
                let count = row["txn_count"].as_i64().unwrap_or(0);
                let desc  = if desc.len() > 55 { &desc[..55] } else { desc };
                lines.push(format!("{}. {} — {} ({} txn{})", i + 1, desc, inr(total), count, s(count)));
            }
            lines.join("\n")
        }
        QueryKind::LargestExpense => {
            let date   = rows.first().and_then(|r| r["value_date"].as_str()).unwrap_or("?");
            let desc   = rows.first().and_then(|r| r["description"].as_str()).unwrap_or("Unknown");
            let amount = first_f64(rows, "amount");
            format!("Your largest single expense is {} — {} ({}).", inr(amount), desc, date)
        }
        QueryKind::TransactionCount => {
            let count = rows.first().and_then(|r| r["total_count"].as_i64()).unwrap_or(0);
            format!("You have {} transaction{} in total.", count, s(count))
        }
        QueryKind::MonthlyBreakdown => {
            if rows.is_empty() { return "No monthly data found.".to_string(); }
            let mut lines = vec!["Monthly breakdown:".to_string()];
            for row in rows {
                let month  = row["month"].as_str().unwrap_or("?");
                let spent  = row["spent"].as_f64().unwrap_or(0.0);
                let earned = row["earned"].as_f64().unwrap_or(0.0);
                lines.push(format!("{}: spent {} | earned {}", month, inr(spent), inr(earned)));
            }
            lines.join("\n")
        }
        QueryKind::CategorySpend => {
            let amount = first_f64(rows, "amount");
            let count  = first_i64(rows, "txn_count");
            if amount == 0.0 {
                format!("No spending recorded {}.", label)
            } else {
                format!(
                    "You spent {} {} across {} transaction{}.",
                    inr(amount), label, count, s(count)
                )
            }
        }
        QueryKind::SameMonthYears => {
            if rows.is_empty() {
                return "No spending data found for the current month across any year.".to_string();
            }
            let mut lines = Vec::new();
            let mut prev_spent: Option<f64> = None;
            for row in rows {
                let period = row["period"].as_str().unwrap_or("?");
                let spent  = row["spent"].as_f64().unwrap_or(0.0);
                let count  = row["txn_count"].as_i64().unwrap_or(0);
                let change = match prev_spent {
                    Some(p) if p > 0.0 => {
                        let pct = (spent - p) / p * 100.0;
                        if pct > 0.0 {
                            format!("  ↑ {:.1}% more than {}", pct, lines.last().map(|_| "previous year").unwrap_or(""))
                        } else {
                            format!("  ↓ {:.1}% less than previous year", pct.abs())
                        }
                    }
                    _ => String::new(),
                };
                lines.push(format!("{}: {} ({} txn{}){}", period, inr(spent), count, s(count), change));
                prev_spent = Some(spent);
            }
            format!("Same-month spending across years:\n{}", lines.join("\n"))
        }
    }
}

/// Generic formatter for Claude-generated query results.
pub fn format_generic(rows: &[Value]) -> String {
    if rows.is_empty() {
        return "No results found for your query.".to_string();
    }
    if rows.len() == 1 {
        if let Some(obj) = rows[0].as_object() {
            if obj.len() == 1 {
                let (k, v) = obj.iter().next().unwrap();
                return format!("{}: {}", human(k), fmtv(v));
            }
            return obj.iter()
                .map(|(k, v)| format!("{}: {}", human(k), fmtv(v)))
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
    let mut lines = Vec::new();
    for (i, row) in rows.iter().enumerate().take(25) {
        if let Some(obj) = row.as_object() {
            let parts: Vec<String> = obj.iter()
                .map(|(k, v)| format!("{}: {}", human(k), fmtv(v)))
                .collect();
            lines.push(format!("{}. {}", i + 1, parts.join("  ·  ")));
        } else {
            lines.push(format!("{}. {}", i + 1, row));
        }
    }
    if rows.len() > 25 {
        lines.push(format!("… and {} more", rows.len() - 25));
    }
    lines.join("\n")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn first_f64(rows: &[Value], key: &str) -> f64 {
    rows.first().and_then(|r| r[key].as_f64()).unwrap_or(0.0)
}
fn first_i64(rows: &[Value], key: &str) -> i64 {
    rows.first().and_then(|r| r[key].as_i64()).unwrap_or(0)
}
fn s(n: i64) -> &'static str { if n == 1 { "" } else { "s" } }

fn human(k: &str) -> String { k.replace('_', " ") }

fn fmtv(v: &Value) -> String {
    match v {
        Value::String(s)  => s.clone(),
        Value::Number(n)  => n.as_f64().map(|f| format!("{:.2}", f)).unwrap_or_else(|| n.to_string()),
        Value::Bool(b)    => b.to_string(),
        Value::Null       => "—".to_string(),
        _                 => v.to_string(),
    }
}

/// Format a float as Indian rupees: ₹12,34,567
pub fn inr(amount: f64) -> String {
    let n   = amount.round() as i64;
    let (sign, abs_n) = if n < 0 { ("−₹", (-n) as u64) } else { ("₹", n as u64) };
    let s   = abs_n.to_string();
    let len = s.len();

    let formatted = if len <= 3 {
        s
    } else {
        // Last 3 digits, then groups of 2 from the right
        let last3    = &s[len - 3..];
        let head     = &s[..len - 3];
        let head_len = head.len();
        let first_sz = if head_len % 2 == 0 { 2 } else { 1 };

        let mut out = String::from(&head[..first_sz]);
        let mut pos = first_sz;
        while pos < head_len {
            out.push(',');
            out.push_str(&head[pos..pos + 2]);
            pos += 2;
        }
        out.push(',');
        out.push_str(last3);
        out
    };

    format!("{}{}", sign, formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inr_formatting() {
        assert_eq!(inr(0.0),        "₹0");
        assert_eq!(inr(999.0),      "₹999");
        assert_eq!(inr(1000.0),     "₹1,000");
        assert_eq!(inr(12345.0),    "₹12,345");
        assert_eq!(inr(123456.0),   "₹1,23,456");
        assert_eq!(inr(1234567.0),  "₹12,34,567");
        assert_eq!(inr(12345678.0), "₹1,23,45,678");
        assert_eq!(inr(-500.0),     "−₹500");
    }

    #[test]
    fn matches_spend_this_month() {
        let m = match_question("How much did I spend this month?", &[]);
        assert!(matches!(m, Some(Predefined { kind: QueryKind::SpendPeriod, .. })));
        let m = m.unwrap();
        assert!(m.sql.contains("CURRENT_DATE"));
        assert_eq!(m.label, "this month");
    }

    #[test]
    fn matches_savings_rate() {
        let m = match_question("What is my savings rate?", &[]);
        assert!(matches!(m, Some(Predefined { kind: QueryKind::SavingsRate, .. })));
    }

    #[test]
    fn matches_top_expenses() {
        let m = match_question("Show me my top 5 expenses", &[]);
        assert!(matches!(m, Some(Predefined { kind: QueryKind::TopExpenses, .. })));
    }

    #[test]
    fn matches_category_by_word() {
        let cats = vec!["Food & Dining".to_string(), "Transport".to_string()];
        let m = match_question("Show me food spending this month", &cats);
        assert!(matches!(m, Some(Predefined { kind: QueryKind::CategorySpend, .. })));
        let m = m.unwrap();
        assert!(m.label.contains("Food & Dining"));
        assert!(m.label.contains("this month"));
    }

    #[test]
    fn matches_yoy_same_month() {
        let cases = [
            "How my spending as compared to previous financial years spending for the same month",
            "Compare my spending to last financial year same month",
            "year over year spending",
            "How does this month compare to last year",
        ];
        for q in cases {
            let m = match_question(q, &[]);
            assert!(
                matches!(m, Some(Predefined { kind: QueryKind::SameMonthYears, .. })),
                "expected SameMonthYears for: {q}"
            );
        }
    }

    #[test]
    fn no_match_returns_none() {
        let m = match_question("What is 2 + 2?", &[]);
        assert!(m.is_none());
    }
}
