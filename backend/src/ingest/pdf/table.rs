//! Row clustering, header/band detection and cell assignment over positioned
//! [`Word`]s lifted from a PDF statement page.

use std::collections::BTreeMap;

use crate::ingest::profiles::{BankProfile, ColKind};

use super::extract::Word;

const Y_TOL: f32 = 3.0; // points; words whose y-centres are closer share a row
// Header must be in the first N rows of a page. A wide scan is safe because the
// `MIN_HEADER_MATCHES >= 3` gate rejects non-header rows — address/summary/legend
// blocks above the table do not match three column aliases at once.
const MAX_SCAN: usize = 40;
const MIN_HEADER_MATCHES: usize = 3; // a header row must match at least this many columns

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub kind: ColKind,
    pub x0: f32,
    pub x1: f32,
}

fn yc(w: &Word) -> f32 {
    (w.y0 + w.y1) / 2.0
}
fn xc(w: &Word) -> f32 {
    (w.x0 + w.x1) / 2.0
}

/// Group words into visual rows: same page, y-centres within `Y_TOL` points.
/// Rows are returned top-to-bottom; words within a row left-to-right.
pub fn rows(mut words: Vec<Word>) -> Vec<Vec<Word>> {
    words.sort_by(|a, b| {
        a.page
            .cmp(&b.page)
            .then(yc(b).partial_cmp(&yc(a)).unwrap())
            .then(a.x0.partial_cmp(&b.x0).unwrap())
    });
    let mut out: Vec<Vec<Word>> = Vec::new();
    for wd in words {
        match out.last_mut() {
            Some(row)
                if row[0].page == wd.page && (yc(&row[0]) - yc(&wd)).abs() <= Y_TOL =>
            {
                row.push(wd);
            }
            _ => out.push(vec![wd]),
        }
    }
    for row in &mut out {
        row.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap());
    }
    out
}

/// The `ColKind` whose *longest* header alias is contained in the lowercased
/// word, across ALL columns. Longest-match (not first-match) so `"value date"`
/// beats the bare `"date"` alias of `TxnDate`. Ties break by declared column
/// order. `None` if nothing matches.
pub(crate) fn match_kind(text: &str, profile: &BankProfile) -> Option<ColKind> {
    let t = text.to_lowercase();
    let mut best: Option<(ColKind, usize)> = None;
    for c in profile.pdf_columns {
        for h in c.headers {
            if t.contains(h) && best.map_or(true, |(_, len)| h.len() > len) {
                best = Some((c.kind, h.len()));
            }
        }
    }
    best.map(|(k, _)| k)
}

/// True if a row is a repeated header row: at least two of its cell values
/// match a column header alias (same longest-match logic as [`match_kind`]).
pub(crate) fn cell_is_header_row(values: &[&str], profile: &BankProfile) -> bool {
    values
        .iter()
        .filter(|v| match_kind(v, profile).is_some())
        .count()
        >= 2
}

/// Scan the first `MAX_SCAN` rows for the row matching the most
/// `profile.pdf_columns` (>= 3 required). Returns X-bands spanning the page,
/// boundaries at midpoints between adjacent matched header-word centres.
pub fn columns(page_rows: &[Vec<Word>], profile: &BankProfile) -> Option<Vec<Band>> {
    let mut best: Option<(usize, Vec<(ColKind, f32)>)> = None;
    for row in page_rows.iter().take(MAX_SCAN) {
        let mut hits: Vec<(ColKind, f32)> = Vec::new();
        for wd in row {
            if let Some(k) = match_kind(&wd.text, profile) {
                if !hits.iter().any(|(hk, _)| *hk == k) {
                    hits.push((k, xc(wd)));
                }
            }
        }
        if hits.len() >= MIN_HEADER_MATCHES
            && best.as_ref().map_or(true, |(n, _)| hits.len() > *n)
        {
            best = Some((hits.len(), hits));
        }
    }
    let (_, mut hits) = best?;
    hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let mut bands = Vec::with_capacity(hits.len());
    for i in 0..hits.len() {
        let x0 = if i == 0 {
            f32::MIN
        } else {
            (hits[i - 1].1 + hits[i].1) / 2.0
        };
        let x1 = if i == hits.len() - 1 {
            f32::MAX
        } else {
            (hits[i].1 + hits[i + 1].1) / 2.0
        };
        bands.push(Band {
            kind: hits[i].0,
            x0,
            x1,
        });
    }
    Some(bands)
}

/// Assign each word in a row to the band whose `[x0,x1)` contains its x-centre.
/// Multiple words in the same band are space-joined in x order.
pub fn cells(row: &[Word], bands: &[Band]) -> BTreeMap<ColKind, String> {
    let mut map: BTreeMap<ColKind, Vec<(f32, &str)>> = BTreeMap::new();
    for wd in row {
        let c = xc(wd);
        if let Some(band) = bands.iter().find(|b| c >= b.x0 && c < b.x1) {
            map.entry(band.kind)
                .or_default()
                .push((wd.x0, wd.text.as_str()));
        }
    }
    map.into_iter()
        .map(|(k, mut v)| {
            v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            (k, v.into_iter().map(|(_, s)| s).collect::<Vec<_>>().join(" "))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::pdf::extract::Word;
    use crate::ingest::profiles::{generic, hdfc};

    fn w(text: &str, x0: f32, x1: f32, y: f32) -> Word {
        Word {
            text: text.into(),
            x0,
            x1,
            y0: y,
            y1: y + 8.0,
            page: 0,
        }
    }

    #[test]
    fn rows_groups_by_y_and_sorts() {
        let ws = vec![
            w("b", 50.0, 60.0, 700.0),
            w("a", 10.0, 20.0, 701.0),
            w("c", 10.0, 20.0, 680.0),
        ];
        let r = rows(ws);
        assert_eq!(r.len(), 2);
        assert_eq!(
            r[0].iter().map(|x| x.text.as_str()).collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(r[1][0].text, "c");
    }

    #[test]
    fn columns_finds_header_and_builds_bands() {
        let header = vec![
            w("Date", 10.0, 40.0, 750.0),
            w("Narration", 60.0, 130.0, 750.0),
            w("Withdrawal", 200.0, 270.0, 750.0),
            w("Deposit", 300.0, 350.0, 750.0),
            w("Balance", 400.0, 450.0, 750.0),
        ];
        let bands = columns(&[header], &generic::profile()).expect("header matched");
        let kinds: Vec<ColKind> = bands.iter().map(|b| b.kind).collect();
        assert_eq!(
            kinds,
            vec![
                ColKind::TxnDate,
                ColKind::Description,
                ColKind::Debit,
                ColKind::Credit,
                ColKind::Balance
            ]
        );
        assert!(bands[0].x0 < 10.0 && bands[4].x1 > 450.0);
        assert!((bands[0].x1 - 60.0).abs() < 1.0);
    }

    #[test]
    fn columns_returns_none_when_too_few_headers_match() {
        let junk = vec![w("Hello", 10.0, 40.0, 750.0), w("World", 60.0, 90.0, 750.0)];
        assert!(columns(&[junk], &generic::profile()).is_none());
    }

    #[test]
    fn cells_buckets_words_by_x_centre() {
        let bands = vec![
            Band {
                kind: ColKind::TxnDate,
                x0: f32::MIN,
                x1: 60.0,
            },
            Band {
                kind: ColKind::Description,
                x0: 60.0,
                x1: 190.0,
            },
            Band {
                kind: ColKind::Balance,
                x0: 190.0,
                x1: f32::MAX,
            },
        ];
        let row = vec![
            w("01/02/2024", 10.0, 50.0, 700.0),
            w("UPI", 70.0, 90.0, 700.0),
            w("swiggy", 95.0, 130.0, 700.0),
            w("1,234.00", 200.0, 240.0, 700.0),
        ];
        let c = cells(&row, &bands);
        assert_eq!(c[&ColKind::TxnDate], "01/02/2024");
        assert_eq!(c[&ColKind::Description], "UPI swiggy");
        assert_eq!(c[&ColKind::Balance], "1,234.00");
    }

    #[test]
    fn value_date_column_is_reachable_despite_bare_date_alias() {
        let p = hdfc::profile();
        assert_eq!(match_kind("Date", &p), Some(ColKind::TxnDate));
        assert_eq!(match_kind("Value Date", &p), Some(ColKind::ValueDate));
        assert_eq!(match_kind("Posting Date", &p), Some(ColKind::TxnDate)); // hdfc: posting date -> TxnDate alias
    }

    #[test]
    fn header_with_both_date_and_value_date_yields_distinct_bands() {
        let header2 = vec![
            w("Date", 10.0, 40.0, 750.0),
            w("Value Date", 60.0, 115.0, 750.0),
            w("Narration", 140.0, 210.0, 750.0),
            w("Withdrawal Amt", 260.0, 330.0, 750.0),
            w("Balance", 400.0, 450.0, 750.0),
        ];
        let bands = columns(&[header2], &hdfc::profile()).expect("header matched");
        let kinds: Vec<ColKind> = bands.iter().map(|b| b.kind).collect();
        assert!(kinds.contains(&ColKind::TxnDate));
        assert!(kinds.contains(&ColKind::ValueDate));
        let ti = kinds.iter().position(|k| *k == ColKind::TxnDate).unwrap();
        let vi = kinds.iter().position(|k| *k == ColKind::ValueDate).unwrap();
        assert!(ti < vi, "TxnDate band left of ValueDate band");
    }

    #[test]
    fn match_kind_respects_declared_order_and_rejects_plain_words() {
        let p = hdfc::profile();
        let header = ["Date", "Narration", "Withdrawal Amt", "Deposit Amt", "Closing Balance"];
        let mapped: Vec<ColKind> = header
            .iter()
            .map(|h| match_kind(h, &p).expect("header word matched"))
            .collect();
        assert_eq!(
            mapped,
            vec![
                ColKind::TxnDate,
                ColKind::Description,
                ColKind::Debit,
                ColKind::Credit,
                ColKind::Balance
            ]
        );
        // "Narration" must not fall through to Credit (des-cr-iption style collisions)
        assert_ne!(match_kind("Narration", &p), Some(ColKind::Credit));
        assert_eq!(match_kind("Statement", &p), None);
    }
}
