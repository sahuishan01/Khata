use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NormalizedTxn {
    pub user_id: Uuid,
    pub statement_id: Uuid,
    pub bank: String,
    pub account_label: String,
    pub txn_date: NaiveDate,
    pub value_date: NaiveDate,
    pub description: String,
    pub raw_description: String,
    pub amount: f64,
    pub direction: String,
    pub balance: Option<f64>,
    pub bank_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub bank_detected: String,
    pub rows_parsed: usize,     // raw rows found in file
    pub normalized: usize,      // rows that passed date+amount parsing
    pub inserted: usize,        // new unique transactions stored
    pub skipped_duplicates: usize,
}

#[derive(Debug, Default, Clone)]
pub struct RawRow {
    pub txn_date: String,
    pub value_date: String,
    pub description: String,
    pub debit: Option<f64>,
    pub credit: Option<f64>,
    pub balance: Option<f64>,
    pub bank_ref: Option<String>,
}
