use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "SBI",
        detect_keywords: &["sbi", "state bank of india", "statebank"],
        txn_date_aliases: &["txn date", "date"],
        value_date_aliases: &["value date"],
        description_aliases: &["description", "particulars", "narration"],
        debit_aliases: &["debit", "dr"],
        credit_aliases: &["credit", "cr"],
        amount_aliases: &[],
        balance_aliases: &["balance"],
        ref_aliases: &["ref no./cheque no.", "ref number", "chq no"],
        date_formats: &["%d %b %Y", "%d/%m/%Y", "%d-%m-%Y"],
        skip_rows: 0,
        statement_kind: StatementKind::Account,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate, headers: &["txn date", "date"] },
            PdfColumn { kind: ColKind::ValueDate, headers: &["value date"] },
            PdfColumn { kind: ColKind::Description, headers: &["description", "particulars", "narration"] },
            PdfColumn { kind: ColKind::Ref, headers: &["ref no./cheque no.", "ref number", "chq no"] },
            PdfColumn { kind: ColKind::Debit, headers: &["debit", "dr"] },
            PdfColumn { kind: ColKind::Credit, headers: &["credit", "cr"] },
            PdfColumn { kind: ColKind::Balance, headers: &["balance"] },
        ],
    }
}
