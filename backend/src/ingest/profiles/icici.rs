use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "ICICI",
        detect_keywords: &["icici", "icicibank"],
        txn_date_aliases: &["transaction date", "date", "txn date"],
        value_date_aliases: &["value date"],
        description_aliases: &[
            "transaction remarks",
            "description",
            "narration",
            "particulars",
        ],
        debit_aliases: &["withdrawal amount (inr)", "debit", "dr"],
        credit_aliases: &["deposit amount (inr)", "credit", "cr"],
        amount_aliases: &[],
        balance_aliases: &["balance (inr)", "balance"],
        ref_aliases: &["cheque number", "ref no.", "reference no"],
        date_formats: &["%d/%m/%Y", "%d-%m-%Y", "%d %b %Y", "%Y-%m-%d"],
        skip_rows: 0,
        statement_kind: StatementKind::Account,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate, headers: &["transaction date", "date", "txn date"] },
            PdfColumn { kind: ColKind::ValueDate, headers: &["value date"] },
            PdfColumn { kind: ColKind::Description, headers: &["transaction remarks", "description", "narration", "particulars"] },
            PdfColumn { kind: ColKind::Ref, headers: &["cheque number", "ref no.", "reference no"] },
            PdfColumn { kind: ColKind::Debit, headers: &["withdrawal amount (inr)", "debit", "dr"] },
            PdfColumn { kind: ColKind::Credit, headers: &["deposit amount (inr)", "credit", "cr"] },
            PdfColumn { kind: ColKind::Balance, headers: &["balance (inr)", "balance"] },
        ],
    }
}
