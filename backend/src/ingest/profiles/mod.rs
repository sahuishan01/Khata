pub mod axis;
pub mod generic;
pub mod generic_cc;
pub mod hdfc;
pub mod icici;
pub mod sbi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementKind {
    Account,
    CreditCard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColKind {
    TxnDate,
    ValueDate,
    Description,
    Debit,
    Credit,
    Amount,
    Balance,
    Ref,
}

#[derive(Debug, Clone, Copy)]
pub struct PdfColumn {
    pub kind: ColKind,
    /// lowercased substrings; a header word matches this column if the
    /// lowercased header-cell text contains any of them
    pub headers: &'static [&'static str],
}

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
    pub statement_kind: StatementKind,
    pub pdf_columns: &'static [PdfColumn],
}

pub fn registry() -> Vec<BankProfile> {
    vec![
        hdfc::profile(),
        icici::profile(),
        sbi::profile(),
        axis::profile(),
        generic_cc::profile(), // credit-card fallback — before generic
        generic::profile(),    // must be last — account fallback
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_account_profile_has_pdf_columns_covering_date_desc_and_money() {
        for p in registry() {
            let kinds: Vec<ColKind> = p.pdf_columns.iter().map(|c| c.kind).collect();
            assert!(kinds.contains(&ColKind::TxnDate), "{} missing TxnDate", p.name);
            assert!(
                kinds.contains(&ColKind::Description),
                "{} missing Description",
                p.name
            );
            match p.statement_kind {
                StatementKind::Account => assert!(
                    kinds.contains(&ColKind::Balance)
                        && (kinds.contains(&ColKind::Debit) || kinds.contains(&ColKind::Amount)),
                    "{} account profile needs Balance + Debit/Amount",
                    p.name
                ),
                StatementKind::CreditCard => assert!(
                    kinds.contains(&ColKind::Amount),
                    "{} cc profile needs Amount",
                    p.name
                ),
            }
        }
    }

    #[test]
    fn registry_has_a_credit_card_profile_before_generic() {
        let names: Vec<&str> = registry().iter().map(|p| p.name).collect();
        let cc = names
            .iter()
            .position(|n| *n == "GENERIC_CC")
            .expect("GENERIC_CC present");
        let generic = names
            .iter()
            .position(|n| *n == "GENERIC")
            .expect("GENERIC present");
        assert!(cc < generic);
    }
}
