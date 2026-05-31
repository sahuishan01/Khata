use super::BankProfile;

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
        ref_aliases: &["cheque no", "chq no", "ref no", "reference number", "chq/ref number"],
        date_formats: &["%d/%m/%y", "%d/%m/%Y", "%d-%m-%Y", "%d-%b-%Y"],
        skip_rows: 0,
    }
}
