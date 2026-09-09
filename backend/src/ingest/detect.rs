use super::profiles::{registry, BankProfile};

#[derive(Debug, Clone, PartialEq)]
pub enum FileKind {
    Csv,
    Excel,
    Pdf,
}

pub fn detect_file_kind(filename: &str) -> FileKind {
    let f = filename.to_lowercase();
    if f.ends_with(".xlsx") || f.ends_with(".xls") {
        FileKind::Excel
    } else if f.ends_with(".pdf") {
        FileKind::Pdf
    } else {
        FileKind::Csv
    }
}

/// Detect the bank profile from the full file hint (sheet name + all preamble + header row text).
/// `file_hint` is already lowercased.
pub fn detect_bank<'a>(profiles: &'a [BankProfile], file_hint: &str) -> &'a BankProfile {
    // Credit-card statements: strong CC wording + no running-balance column.
    let looks_cc = ["credit card", "minimum amount due", "total amount due", "statement of account"]
        .iter()
        .any(|k| file_hint.contains(k));
    let has_balance_header = ["closing balance", "running balance", "available balance"]
        .iter()
        .any(|k| file_hint.contains(k));
    if looks_cc && !has_balance_header {
        if let Some(p) = profiles.iter().find(|p| p.name == "GENERIC_CC") {
            return p;
        }
    }

    for p in profiles {
        if p.detect_keywords.iter().any(|kw| file_hint.contains(kw)) {
            return p;
        }
    }
    profiles.last().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hdfc_from_preamble() {
        let profiles = registry();
        // "HDFC BANK Ltd." appears in preamble, not column headers
        let hint = "hdfc bank ltd. statement of accounts date narration withdrawal amt. closing balance";
        assert_eq!(detect_bank(&profiles, hint).name, "HDFC");
    }

    #[test]
    fn detects_hdfc_from_column_headers() {
        let profiles = registry();
        let hint = "date hdfc bank statement narration withdrawal amt.";
        assert_eq!(detect_bank(&profiles, hint).name, "HDFC");
    }

    #[test]
    fn detects_axis_from_preamble() {
        let profiles = registry();
        let hint = "axis bank account statement tran date particulars dr cr bal";
        assert_eq!(detect_bank(&profiles, hint).name, "Axis");
    }

    #[test]
    fn detects_generic_cc_from_credit_card_hint() {
        let profiles = registry();
        let hint = "hdfc bank credit card statement ... total amount due 5,000 transaction date amount";
        assert_eq!(detect_bank(&profiles, hint).name, "GENERIC_CC");
    }

    #[test]
    fn falls_back_to_generic() {
        let profiles = registry();
        assert_eq!(detect_bank(&profiles, "date amount description balance").name, "GENERIC");
    }
}
