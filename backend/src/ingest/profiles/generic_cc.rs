use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "GENERIC_CC",
        detect_keywords: &[], // selected by detect_bank's CC heuristic, not keywords
        txn_date_aliases: &["date", "transaction date", "txn date"],
        value_date_aliases: &["posting date", "post date"],
        description_aliases: &["description", "transaction details", "particulars", "merchant"],
        debit_aliases: &[],
        credit_aliases: &[],
        amount_aliases: &["amount", "amount (inr)", "amount in rs"],
        balance_aliases: &[],
        ref_aliases: &["reference", "ref"],
        date_formats: &["%d/%m/%Y", "%d-%m-%Y", "%d %b %Y", "%d/%m/%y", "%d-%b-%Y"],
        skip_rows: 0,
        statement_kind: StatementKind::CreditCard,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate, headers: &["date", "transaction date", "txn date"] },
            PdfColumn { kind: ColKind::ValueDate, headers: &["posting date", "post date"] },
            PdfColumn { kind: ColKind::Description, headers: &["description", "transaction details", "particulars", "merchant"] },
            PdfColumn { kind: ColKind::Amount, headers: &["amount"] },
        ],
    }
}
