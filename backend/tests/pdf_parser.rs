#[path = "pdf/gen.rs"]
mod gen;

use gen::{Col, Row, StatementSpec};
use khata::ingest::pdf::{extract, parse_pdf};
use khata::ingest::profiles;

fn ready() -> bool {
    let ok = extract::init(std::env::var("PDFIUM_LIB_PATH").ok().as_deref()).is_ok();
    if !ok {
        if std::env::var("CI").is_ok() {
            panic!("pdfium unavailable but CI=true — integration tests must run in CI");
        }
        eprintln!("skip: no pdfium");
    }
    ok
}

fn row(cells: &[&str]) -> Row {
    Row {
        cells: cells.iter().map(|s| s.to_string()).collect(),
    }
}

fn hdfc_basic_spec() -> StatementSpec {
    StatementSpec {
        cols: vec![
            Col { title: "Date", x_mm: 15.0 },
            Col { title: "Narration", x_mm: 40.0 },
            Col { title: "Withdrawal Amt", x_mm: 115.0 },
            Col { title: "Deposit Amt", x_mm: 150.0 },
            Col { title: "Closing Balance", x_mm: 180.0 },
        ],
        rows: vec![
            row(&["01/02/24", "UPI-SWIGGY-416823456789", "450.00", "", "9,550.00"]),
            row(&["02/02/24", "NEFT SALARY CREDIT", "", "50,000.00", "59,550.00"]),
            row(&["03/02/24", "ATM WDL", "2,000.00", "", "57,550.00"]),
            row(&["05/02/24", "IMPS RENT", "15,000.00", "", "42,550.00"]),
            row(&["07/02/24", "INTEREST", "", "125.00", "42,675.00"]),
        ],
        repeat_header_pages: false,
        preamble: vec!["HDFC BANK".into(), "Statement of account".into()],
    }
}

#[test]
fn account_hdfc_basic_parses_all_rows_high_confidence() {
    if !ready() {
        return;
    }
    let pdf = gen::statement(&hdfc_basic_spec());
    let (rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert_eq!(rows.len(), 5, "all 5 transactions");
    assert_eq!(reason, None, "balances reconcile -> no warning");
    assert_eq!(rows[0].debit, Some(450.00));
    assert!(rows[0].description.contains("SWIGGY"));
    assert_eq!(rows[1].credit, Some(50_000.00));
    assert_eq!(rows[1].debit, None, "credit row must not be a debit");
}

#[test]
fn spaced_dates_are_not_missed() {
    if !ready() {
        return;
    }
    let mut spec = hdfc_basic_spec();
    for (i, r) in spec.rows.iter_mut().enumerate() {
        r.cells[0] = format!("{:02} Feb 2024", i + 1);
    }
    let pdf = gen::statement(&spec);
    let (rows, ..) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert_eq!(rows.len(), 5);
}

#[test]
fn wrong_balance_is_flagged_low() {
    if !ready() {
        return;
    }
    let mut spec = hdfc_basic_spec();
    spec.rows[2].cells[4] = "1.00".into();
    let pdf = gen::statement(&spec);
    let (_rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert!(reason.unwrap().contains("reconcile"));
}

#[test]
fn credit_card_layout_reads_cr_dr() {
    if !ready() {
        return;
    }
    let spec = StatementSpec {
        cols: vec![
            Col { title: "Date", x_mm: 15.0 },
            Col { title: "Transaction Details", x_mm: 45.0 },
            Col { title: "Amount", x_mm: 160.0 },
        ],
        rows: vec![
            row(&["05/02/2024", "AMAZON IN", "2,000.00 Dr"]),
            row(&["07/02/2024", "REFUND ETSY", "500.00 Cr"]),
            row(&["09/02/2024", "SWIGGY", "800.00 Dr"]),
        ],
        repeat_header_pages: false,
        preamble: vec![
            "Your Credit Card Statement".into(),
            "Total Amount Due 2,300.00".into(),
        ],
    };
    let pdf = gen::statement(&spec);
    let (rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::generic_cc::profile()).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].debit, Some(2_000.00));
    assert_eq!(rows[1].credit, Some(500.00));
    assert_eq!(reason, None, "2000 - 500 + 800 = 2300 matches total due");
}

#[test]
fn multiline_narration_is_joined() {
    if !ready() {
        return;
    }
    let mut spec = hdfc_basic_spec();
    spec.rows.insert(1, row(&["", "CONTD PART TWO OF NARRATION", "", "", ""]));
    let pdf = gen::statement(&spec);
    let (rows, ..) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert_eq!(rows.len(), 5, "continuation is not a new transaction");
    assert!(rows[0].description.contains("PART TWO"));
}

#[test]
fn repeated_page_header_does_not_create_rows() {
    if !ready() {
        return;
    }
    let mut spec = hdfc_basic_spec();
    spec.repeat_header_pages = true;
    for i in 0..40 {
        spec.rows.push(row(&[
            &format!("{:02}/03/24", (i % 28) + 1),
            "FILLER TXN",
            "1.00",
            "",
            "42,674.00",
        ]));
    }
    let pdf = gen::statement(&spec);
    let (rows, ..) = parse_pdf(&pdf, &profiles::hdfc::profile()).unwrap();
    assert!(rows
        .iter()
        .all(|r| !r.description.contains("Narration") && !r.description.contains("Withdrawal")));
    assert!(
        rows.len() >= 40,
        "expected the 5 real + 40 filler txns, got {}",
        rows.len()
    );
}

#[test]
fn unreadable_layout_falls_back_with_warning() {
    if !ready() {
        return;
    }
    let spec = StatementSpec {
        cols: vec![
            Col { title: "Foo", x_mm: 15.0 },
            Col { title: "Bar", x_mm: 80.0 },
        ],
        rows: vec![row(&["01/02/2024 something 100.00", ""])],
        repeat_header_pages: false,
        preamble: vec![],
    };
    let pdf = gen::statement(&spec);
    let (_rows, _h, _t, reason) = parse_pdf(&pdf, &profiles::generic::profile()).unwrap();
    assert!(reason.is_some());
}
