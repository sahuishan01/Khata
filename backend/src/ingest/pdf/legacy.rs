use crate::ingest::{models::RawRow, profiles::BankProfile};

const MAX_PARSE_ROWS: usize = 100_000;

pub fn extract_text(bytes: &[u8]) -> String {
    pdf_extract::extract_text_from_mem(bytes).unwrap_or_default()
}

pub fn parse_pdf(
    bytes: &[u8],
    _profile: &BankProfile,
) -> anyhow::Result<(Vec<RawRow>, Vec<String>, String)> {
    let text = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to extract text from PDF: {:?}", e))?;

    let file_hint = text.to_lowercase();
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    let mut raw_rows = Vec::new();
    let headers = vec![
        "Txn Date".to_string(),
        "Description".to_string(),
        "Debit".to_string(),
        "Credit".to_string(),
        "Balance".to_string(),
    ];

    let mut current_row: Option<RawRow> = None;

    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        let first_word = parts[0];
        let is_date = first_word.contains('/') || first_word.contains('-');

        if is_date && parts.len() >= 3 {
            if let Some(r) = current_row.take() {
                raw_rows.push(r);
            }

            let date_str = first_word.to_string();
            let mut desc_parts = Vec::new();
            let mut num_parts = Vec::new();

            for &p in &parts[1..] {
                let clean_p = p.replace(',', "");
                if clean_p.parse::<f64>().is_ok() {
                    num_parts.push(clean_p.parse::<f64>().unwrap());
                } else {
                    desc_parts.push(p);
                }
            }

            let description = desc_parts.join(" ");
            let (debit, credit, balance) = match num_parts.as_slice() {
                [d, b] => (Some(*d), None, Some(*b)),
                [d, c, b] => (Some(*d), Some(*c), Some(*b)),
                [amt] => (Some(*amt), None, None),
                _ => (None, None, None),
            };

            current_row = Some(RawRow {
                txn_date: date_str.clone(),
                value_date: date_str,
                description,
                debit,
                credit,
                balance,
                bank_ref: None,
            });
        } else if let Some(ref mut r) = current_row {
            r.description.push(' ');
            r.description.push_str(line);
        }
    }

    if let Some(r) = current_row {
        raw_rows.push(r);
    }

    if raw_rows.len() > MAX_PARSE_ROWS {
        raw_rows.truncate(MAX_PARSE_ROWS);
    }

    Ok((raw_rows, headers, file_hint))
}
