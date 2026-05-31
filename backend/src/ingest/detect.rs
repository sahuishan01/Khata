use super::profiles::{registry, BankProfile};

#[derive(Debug, Clone, PartialEq)]
pub enum FileKind {
    Csv,
    Excel,
}

pub fn detect_file_kind(filename: &str) -> FileKind {
    let f = filename.to_lowercase();
    if f.ends_with(".xlsx") || f.ends_with(".xls") {
        FileKind::Excel
    } else {
        FileKind::Csv
    }
}

/// Detect the bank profile from the full file hint (sheet name + all preamble + header row text).
/// `file_hint` is already lowercased.
pub fn detect_bank<'a>(profiles: &'a [BankProfile], file_hint: &str) -> &'a BankProfile {
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
    fn falls_back_to_generic() {
        let profiles = registry();
        assert_eq!(detect_bank(&profiles, "date amount description balance").name, "GENERIC");
    }
}
