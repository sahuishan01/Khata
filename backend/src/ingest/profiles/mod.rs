pub mod generic;
pub mod hdfc;
pub mod icici;
pub mod sbi;

#[derive(Debug, Clone)]
pub struct BankProfile {
    pub name: &'static str,
    pub detect_keywords: &'static [&'static str],
    pub txn_date_aliases: &'static [&'static str],
    pub value_date_aliases: &'static [&'static str],
    pub description_aliases: &'static [&'static str],
    pub debit_aliases: &'static [&'static str],
    pub credit_aliases: &'static [&'static str],
    pub amount_aliases: &'static [&'static str],
    pub balance_aliases: &'static [&'static str],
    pub ref_aliases: &'static [&'static str],
    pub date_formats: &'static [&'static str],
    pub skip_rows: usize,
}

pub fn registry() -> Vec<BankProfile> {
    vec![
        hdfc::profile(),
        icici::profile(),
        sbi::profile(),
        generic::profile(),
    ]
}
