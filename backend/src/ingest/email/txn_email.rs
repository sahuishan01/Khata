//! Best-effort extraction of a single transaction from a bank *alert* email —
//! the "Rs. 450 debited from a/c XX1234 to swiggy@ybl" kind that carries no
//! statement attachment. Deliberately conservative: when anything is
//! ambiguous we return `None` and skip the mail rather than invent a txn.
//!
//! The patterns mirror `android/.../sms/SmsParser.kt`; keep the two in sync.

use chrono::NaiveDate;
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub struct BodyTxn {
    pub amount: f64,
    /// "debit" | "credit"
    pub direction: String,
    pub description: String,
    pub bank_ref: Option<String>,
    /// Masked account / card tail, e.g. "XX1234".
    pub account_label: Option<String>,
    /// Date parsed from the body, if any; the caller falls back to the mail date.
    pub txn_date: Option<NaiveDate>,
}

struct Patterns {
    debit: Regex,
    credit: Regex,
    acct: Regex,
    reference: Regex,
    payee: Regex,
    date: Regex,
    bank_kw: Regex,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        // "<verb> ... Rs 1,234.50"  or  "Rs 1,234.50 ... <verb>"
        debit: Regex::new(r"(?i)(?:debited|spent|paid|sent|withdrawn|purchase of|txn of|transferred|using .{0,40}? for|used .{0,40}? for)\D{0,40}?(?:rs\.?|inr|₹)\s*([0-9][0-9,]*(?:\.[0-9]{1,2})?)|(?:rs\.?|inr|₹)\s*([0-9][0-9,]*(?:\.[0-9]{1,2})?)\s+(?:at|towards|to)\s|(?:rs\.?|inr|₹)\s*([0-9][0-9,]*(?:\.[0-9]{1,2})?)\D{0,20}?(?:debited|spent|paid|withdrawn|transferred)").unwrap(),
        credit: Regex::new(r"(?i)(?:credited|received|deposited|refund(?:ed)? of|added)\D{0,40}?(?:rs\.?|inr|₹)\s*([0-9][0-9,]*(?:\.[0-9]{1,2})?)|(?:rs\.?|inr|₹)\s*([0-9][0-9,]*(?:\.[0-9]{1,2})?)\D{0,20}?(?:credited|received|deposited)").unwrap(),
        acct: Regex::new(r"(?i)(?:a/?c|account|card)(?:\s*(?:no\.?|number|ending|ending in|xx+|x+))?[:\s#\-x\*]*([0-9]{3,6})").unwrap(),
        reference: Regex::new(r"(?i)(?:upi(?:\s+transaction)?\s+ref(?:erence)?(?:\s+(?:no|number))?|txn\s*id|transaction\s+id|rrn|ref(?:erence)?\s+(?:no|number))\D{0,6}([A-Za-z0-9]{6,})").unwrap(),
        payee: Regex::new(r"(?i)(?:to(?:\s+vpa)?|at|towards|in\s+favou?r\s+of|paid\s+to)\s+([A-Za-z0-9@._][A-Za-z0-9@._\- ]{1,39}?)(?:\s+on\b|\s+ref\b|\s+dated\b|\s+txn\b|[.,;\n]|$)").unwrap(),
        date: Regex::new(r"(?i)\b(\d{1,2}[-/ ](?:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*[-/ ]\d{2,4}|\d{1,2}[-/]\d{1,2}[-/]\d{2,4})\b").unwrap(),
        bank_kw: Regex::new(r"(?i)\b(hdfc|icici|sbi|state bank|axis|kotak|pnb|punjab national|bob|bank of baroda|canara|idfc|yes bank|indusind|federal|rbl|au small)\b").unwrap(),
    })
}

/// Non-transactional / not-yet-happened mails we must never turn into a txn.
fn is_non_txn(lc: &str) -> bool {
    const BLOCK: &[&str] = &[
        "otp", "one time password", "one-time password", "login code",
        "will be debited", "will be deducted", "has been scheduled", "is scheduled",
        "e-statement", "estatement", "statement is ready", "statement of account",
        "account statement", "monthly statement", "credit card statement",
        "minimum amount due", "total amount due", "payment due", "due on", "due date",
        "mandate", "e-mandate", "auto pay", "autopay", "standing instruction",
        "failed", "declined", "unsuccessful", "could not be processed", "reversed",
        "reversal", "request for", "you requested", "otp for",
    ];
    BLOCK.iter().any(|k| lc.contains(k))
}

fn parse_amount(s: &str) -> Option<f64> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    cleaned.parse().ok().filter(|v: &f64| *v > 0.0)
}

fn first_group(caps: &regex::Captures) -> Option<f64> {
    caps.iter()
        .skip(1)
        .flatten()
        .find_map(|m| parse_amount(m.as_str()))
}

fn parse_body_date(s: &str) -> Option<NaiveDate> {
    let s = s.replace(['/', ' '], "-");
    for fmt in ["%d-%m-%Y", "%d-%m-%y", "%d-%b-%Y", "%d-%b-%y", "%d-%B-%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(&s, fmt) {
            return Some(d);
        }
    }
    None
}

/// `subject` + `body` are the mail's text; `sender` is the From address. Returns
/// a txn only when an amount, a direction and a bank signal are all present.
pub fn extract(subject: &str, body: &str, sender: &str) -> Option<BodyTxn> {
    let text = format!("{subject}\n{body}");
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let lc = flat.to_lowercase();

    if is_non_txn(&lc) {
        return None;
    }

    let p = patterns();

    let (amount, direction) = p
        .debit
        .captures(&flat)
        .and_then(|c| first_group(&c))
        .map(|a| (a, "debit"))
        .or_else(|| {
            p.credit
                .captures(&flat)
                .and_then(|c| first_group(&c))
                .map(|a| (a, "credit"))
        })?;

    // Require a bank signal so newsletters that happen to quote a rupee figure
    // don't slip through.
    let sender_lc = sender.to_lowercase();
    let bank_in_sender = ["hdfc", "icici", "sbi", "axis", "kotak", "pnb", "idfc", "yesbank", "indusind", "federal", "rbl", "aubank", "bankofbaroda", "canara"]
        .iter()
        .any(|b| sender_lc.contains(b));
    if !bank_in_sender && !p.bank_kw.is_match(&flat) {
        return None;
    }

    let account_label = p
        .acct
        .captures(&flat)
        .and_then(|c| c.get(1))
        .map(|m| format!("XX{}", m.as_str()));

    let bank_ref = p
        .reference
        .captures(&flat)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let description = p
        .payee
        .captures(&flat)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().trim_end_matches('.').to_string())
        .filter(|s| s.len() > 2 && !s.eq_ignore_ascii_case("your"))
        .unwrap_or_else(|| {
            if direction == "credit" { "Credit".to_string() } else { "Debit".to_string() }
        });

    let txn_date = p
        .date
        .captures(&flat)
        .and_then(|c| c.get(1))
        .and_then(|m| parse_body_date(m.as_str()));

    Some(BodyTxn {
        amount,
        direction: direction.to_string(),
        description,
        bank_ref,
        account_label,
        txn_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hdfc_upi_debit() {
        let b = "Dear Customer, Rs. 450.00 has been debited from account **1234 to VPA swiggy@ybl on 05-09-2025. Your UPI transaction reference number is 512345678901. - HDFC Bank";
        let t = extract("You have done a UPI txn", b, "alerts@hdfcbank.net").unwrap();
        assert_eq!(t.amount, 450.0);
        assert_eq!(t.direction, "debit");
        assert_eq!(t.account_label.as_deref(), Some("XX1234"));
        assert_eq!(t.bank_ref.as_deref(), Some("512345678901"));
        assert!(t.description.to_lowercase().contains("swiggy"));
        assert_eq!(t.txn_date, NaiveDate::from_ymd_opt(2025, 9, 5));
    }

    #[test]
    fn icici_credit() {
        let b = "Your account XXXXXX1234 has been credited with INR 2,000.00 on 05-Sep-25. - ICICI Bank";
        let t = extract("Transaction alert", b, "no-reply@icicibank.com").unwrap();
        assert_eq!(t.amount, 2000.0);
        assert_eq!(t.direction, "credit");
    }

    #[test]
    fn card_spend() {
        let b = "Thank you for using HDFC Bank Card ending 5678 for Rs 1,299.00 at AMAZON on 04-09-2025.";
        let t = extract("Card transaction", b, "cards@hdfcbank.net").unwrap();
        assert_eq!(t.amount, 1299.0);
        assert_eq!(t.direction, "debit");
        assert_eq!(t.account_label.as_deref(), Some("XX5678"));
    }

    #[test]
    fn rejects_otp() {
        assert!(extract("OTP", "Your OTP for a txn of Rs 500 is 123456", "hdfcbank.net").is_none());
    }

    #[test]
    fn rejects_statement_ready() {
        let b = "Your account statement for Sep 2025 is ready. Closing balance Rs 12,345.00.";
        assert!(extract("Statement ready", b, "hdfcbank.net").is_none());
    }

    #[test]
    fn rejects_future_debit() {
        let b = "Rs 999.00 will be debited from your account on 10-09-2025 towards your SIP.";
        assert!(extract("SIP reminder", b, "icicibank.com").is_none());
    }

    #[test]
    fn rejects_non_bank_sender_and_body() {
        let b = "Grab this deal! Save Rs 500 credited as cashback on your next order.";
        assert!(extract("Sale!", b, "offers@shopping.com").is_none());
    }
}
