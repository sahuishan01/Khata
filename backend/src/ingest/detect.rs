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

pub fn detect_bank<'a>(profiles: &'a [BankProfile], headers: &[String], sheet_name: &str) -> &'a BankProfile {
    let haystack = format!("{} {}", headers.join(" "), sheet_name).to_lowercase();
    for p in profiles {
        if p.detect_keywords.iter().any(|kw| haystack.contains(kw)) {
            return p;
        }
    }
    profiles.last().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hdfc() {
        let profiles = registry();
        let headers = vec!["Date".into(), "HDFC Bank Statement".into()];
        let p = detect_bank(&profiles, &headers, "");
        assert_eq!(p.name, "HDFC");
    }

    #[test]
    fn falls_back_to_generic() {
        let profiles = registry();
        let p = detect_bank(&profiles, &["Date".into(), "Amount".into()], "Sheet1");
        assert_eq!(p.name, "GENERIC");
    }
}
