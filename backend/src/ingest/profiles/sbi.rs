use super::BankProfile;

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
    }
}
