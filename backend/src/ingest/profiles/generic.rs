use super::BankProfile;

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
    }
}
