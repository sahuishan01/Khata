use super::{BankProfile, ColKind, PdfColumn, StatementKind};

pub fn profile() -> BankProfile {
    BankProfile {
        name: "Axis",
        detect_keywords: &["axis bank", "axisbank", "axis"],
        txn_date_aliases: &["tran date", "transaction date", "date", "txn date", "value date"],
        value_date_aliases: &["value date", "tran date", "date"],
        description_aliases: &[
            "particulars",
            "transaction remarks",
            "narration",
            "description",
            "remarks",
        ],
        debit_aliases: &[
            "withdrawal (dr)",
            "debit amount",
            "withdrawal amt",
            "dr",
            "debit",
            "withdrawal",
        ],
        credit_aliases: &[
            "deposit (cr)",
            "credit amount",
            "deposit amt",
            "cr",
            "credit",
            "deposit",
        ],
        amount_aliases: &[],
        balance_aliases: &["bal", "balance", "closing balance", "available balance"],
        ref_aliases: &["chq/ref number", "chqno", "cheque number", "ref no", "ref"],
        date_formats: &[
            "%d-%m-%Y",
            "%d/%m/%Y",
            "%d-%b-%Y",
            "%d %b %Y",
            "%Y-%m-%d",
            "%d/%m/%y",
        ],
        skip_rows: 0,
        statement_kind: StatementKind::Account,
        pdf_columns: &[
            PdfColumn { kind: ColKind::TxnDate, headers: &["tran date", "transaction date", "date", "txn date"] },
            PdfColumn { kind: ColKind::ValueDate, headers: &["value date"] },
            PdfColumn { kind: ColKind::Description, headers: &["particulars", "transaction remarks", "narration", "description", "remarks"] },
            PdfColumn { kind: ColKind::Ref, headers: &["chq/ref number", "chqno", "cheque number", "ref no", "ref"] },
            PdfColumn { kind: ColKind::Debit, headers: &["withdrawal (dr)", "debit amount", "withdrawal amt", "dr", "debit", "withdrawal"] },
            PdfColumn { kind: ColKind::Credit, headers: &["deposit (cr)", "credit amount", "deposit amt", "cr", "credit", "deposit"] },
            PdfColumn { kind: ColKind::Balance, headers: &["bal", "balance", "closing balance", "available balance"] },
        ],
    }
}
