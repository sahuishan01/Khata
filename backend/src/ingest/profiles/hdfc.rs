use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "HDFC",
        detect_keywords: &["hdfc", "hdfcbank"],
        txn_date_aliases: &["date", "txn date", "transaction date", "posting date"],
        value_date_aliases: &["value date", "value dt"],
        description_aliases: &["narration", "description", "particulars", "remarks"],
        debit_aliases: &["withdrawal amt", "debit amount", "dr", "debit"],
        credit_aliases: &["deposit amt", "credit amount", "cr", "credit"],
        amount_aliases: &[],
        balance_aliases: &["closing balance", "balance", "running balance"],
        ref_aliases: &["chq./ref.no.", "chq/ref number", "cheque no", "chq no", "ref no", "reference number"],
        date_formats: &["%d/%m/%y", "%d/%m/%Y", "%d-%m-%Y", "%d-%b-%Y", "%d %b %Y"],
        skip_rows: 0,
        statement_kind: StatementKind::Account,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate, headers: &["date", "txn date", "transaction date", "posting date"] },
            PdfColumn { kind: ColKind::ValueDate, headers: &["value date", "value dt"] },
            PdfColumn { kind: ColKind::Description, headers: &["narration", "description", "particulars", "remarks"] },
            PdfColumn { kind: ColKind::Ref, headers: &["chq./ref.no.", "chq/ref number", "cheque no", "chq no", "ref no", "reference number"] },
            PdfColumn { kind: ColKind::Debit, headers: &["withdrawal amt", "debit amount", "dr", "debit"] },
            PdfColumn { kind: ColKind::Credit, headers: &["deposit amt", "credit amount", "cr", "credit"] },
            PdfColumn { kind: ColKind::Balance, headers: &["closing balance", "balance", "running balance"] },
        ],
    }
}
