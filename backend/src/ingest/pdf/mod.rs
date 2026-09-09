pub mod extract;
pub mod reconcile;
pub mod rows;
pub mod table;
mod legacy;

use anyhow::Result;
use crate::ingest::models::RawRow;
use crate::ingest::profiles::BankProfile;

const HEADERS: [&str; 5] = ["Txn Date", "Description", "Debit", "Credit", "Balance"];

/// Text-only extraction for bank detection. Uses pdfium if available and it
/// yields words, else falls back to `pdf_extract`. Never errors on a readable PDF.
pub fn extract_text(bytes: &[u8]) -> String {
    if extract::is_available() {
        if let Ok(words) = extract::words(bytes) {
            if !words.is_empty() {
                return words
                    .iter()
                    .map(|w| w.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
            }
        }
    }
    legacy::extract_text(bytes)
}

/// Returns (rows, headers, lowercased_full_text, low_confidence_reason).
pub fn parse_pdf(
    bytes: &[u8],
    profile: &BankProfile,
) -> Result<(Vec<RawRow>, Vec<String>, String, Option<String>)> {
    let headers = HEADERS.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    // Try the coordinate-aware path.
    if extract::is_available() {
        if let Ok(words) = extract::words(bytes) {
            let full_text = words
                .iter()
                .map(|w| w.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            let all_rows = table::rows(words);

            // group rows by page for header detection
            let mut by_page: Vec<Vec<Vec<extract::Word>>> = Vec::new();
            for row in &all_rows {
                if row.is_empty() {
                    continue;
                }
                let p = row[0].page;
                if by_page.len() <= p {
                    by_page.resize(p + 1, Vec::new());
                }
                by_page[p].push(row.clone());
            }

            let mut bands = None;
            for page in &by_page {
                if let Some(b) = table::columns(page, profile) {
                    bands = Some(b);
                    break;
                }
            }

            if let Some(bands) = bands {
                let cell_rows = all_rows
                    .iter()
                    .map(|r| table::cells(r, &bands))
                    .collect::<Vec<_>>();
                let parsed = rows::assemble(cell_rows, profile);
                if parsed.len() >= 2 {
                    let reason = match reconcile::confidence(
                        &parsed,
                        profile.statement_kind,
                        &full_text,
                    ) {
                        reconcile::Confidence::Ok => None,
                        reconcile::Confidence::Low(m) => Some(m),
                    };
                    return Ok((parsed, headers, full_text, reason));
                }
            }
        }
    }

    // Fallback: legacy line heuristic.
    let (rows_l, headers_l, text_l) = legacy::parse_pdf(bytes, profile)?;
    let reason = Some(if extract::is_available() {
        "layout not recognised; used the fallback parser — please verify amounts".to_string()
    } else {
        "high-accuracy PDF parser unavailable; used the fallback parser — please verify amounts"
            .to_string()
    });
    Ok((rows_l, headers_l, text_l.to_lowercase(), reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::profiles::generic;

    #[test]
    fn parse_pdf_falls_back_and_flags_when_pdfium_absent_or_no_header() {
        let res = parse_pdf(b"not a pdf at all", &generic::profile());
        assert!(res.is_err() || res.as_ref().unwrap().3.is_some());
    }

    #[test]
    fn extract_text_never_panics_on_junk() {
        assert_eq!(extract_text(b"junk"), String::new());
    }
}
