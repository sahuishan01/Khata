use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "GENERIC",
        detect_keywords: &[],
        txn_date_aliases: &["date", "txn date", "transaction date", "value date"],
        value_date_aliases: &["value date", "date"],
        description_aliases: &[
            "description",
            "narration",
            "particulars",
            "remarks",
            "transaction remarks",
        ],
        debit_aliases: &["debit", "withdrawal", "dr", "amount(dr)"],
        credit_aliases: &["credit", "deposit", "cr", "amount(cr)"],
        amount_aliases: &["amount", "transaction amount"],
        balance_aliases: &["balance", "closing balance", "available balance"],
        ref_aliases: &["ref", "ref no", "cheque no", "reference"],
        date_formats: &[
            "%d/%m/%Y",
            "%d-%m-%Y",
            "%d %b %Y",
            "%Y-%m-%d",
            "%d/%m/%y",
        ],
        skip_rows: 0,
        statement_kind: StatementKind::Account,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate, headers: &["date", "txn date", "transaction date"] },
            PdfColumn { kind: ColKind::ValueDate, headers: &["value date"] },
            PdfColumn { kind: ColKind::Description, headers: &["description", "narration", "particulars", "remarks", "transaction remarks"] },
            PdfColumn { kind: ColKind::Ref, headers: &["ref", "ref no", "cheque no", "reference"] },
            PdfColumn { kind: ColKind::Debit, headers: &["debit", "withdrawal", "dr", "amount(dr)"] },
            PdfColumn { kind: ColKind::Credit, headers: &["credit", "deposit", "cr", "amount(cr)"] },
            PdfColumn { kind: ColKind::Amount, headers: &["amount", "transaction amount"] },
            PdfColumn { kind: ColKind::Balance, headers: &["balance", "closing balance", "available balance"] },
        ],
    }
}
