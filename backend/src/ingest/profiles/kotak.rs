use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "KOTAK",
        detect_keywords: &["kotak", "kkbk"],
        txn_date_aliases: &["date", "txn date", "transaction date", "value date"],
        value_date_aliases: &["value date"],
        description_aliases: &["description", "narration", "particulars", "remarks"],
        debit_aliases: &["withdrawal", "debit", "dr"],
        credit_aliases: &["deposit", "credit", "cr"],
        amount_aliases: &[
            "withdrawal (dr.) / deposit (cr.)",
            "withdrawal(dr)/deposit(cr)",
            "withdrawal / deposit",
            "amount",
        ],
        balance_aliases: &["balance", "closing balance"],
        ref_aliases: &["chq/ref. no.", "chq/ref no", "ref no", "reference", "cheque"],
        date_formats: &["%d-%m-%Y", "%d/%m/%Y", "%d %b %Y", "%d-%b-%Y", "%d/%m/%y"],
        skip_rows: 0,
        statement_kind: StatementKind::Account,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate, headers: &["date", "txn date", "transaction date"] },
            PdfColumn { kind: ColKind::ValueDate, headers: &["value date"] },
            PdfColumn { kind: ColKind::Description, headers: &["description", "narration", "particulars", "remarks"] },
            PdfColumn { kind: ColKind::Ref, headers: &["chq/ref. no.", "chq/ref no", "ref. no.", "ref no", "cheque", "reference"] },
            PdfColumn {
                kind: ColKind::Amount,
                headers: &[
                    "withdrawal (dr.) / deposit (cr.)",
                    "withdrawal(dr)/deposit(cr)",
                    "withdrawal / deposit",
                    "dr/cr",
                ],
            },
            PdfColumn { kind: ColKind::Debit, headers: &["withdrawal", "debit"] },
            PdfColumn { kind: ColKind::Credit, headers: &["deposit", "credit"] },
            PdfColumn { kind: ColKind::Balance, headers: &["balance", "closing balance"] },
        ],
    }
}
