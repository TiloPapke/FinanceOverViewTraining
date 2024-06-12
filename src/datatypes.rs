use async_session::chrono::{DateTime, Utc};
use mongodb::bson::Uuid;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GenerallUserData {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordResetTokenRequest {
    pub user_name: String,
    pub reset_secret: secrecy::Secret<String>,
}

#[derive(Debug)]
pub struct PasswordResetTokenRequestResult {
    pub reset_token: String,
    pub expires_at: DateTime<Utc>,
    pub user_email: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordResetRequest {
    pub username: String,
    pub reset_token: String,
    pub new_password: secrecy::Secret<String>,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct FinanceAccountType {
    pub id: Uuid,
    pub title: String,
    pub description: String,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct FinanceAccount {
    pub id: Uuid,
    pub finance_account_type_id: Uuid,
    pub title: String,
    pub description: String,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct FinanceJournalEntry {
    pub id: Uuid,
    pub is_simple_entry: bool,
    pub is_saldo: bool,
    pub debit_finance_account_id: Uuid,
    pub credit_finance_account_id: Uuid,
    pub running_number: u64,
    pub booking_time: DateTime<Utc>,
    pub amount: u128,
    pub title: String,
    pub description: String,
}

pub enum BookingEntryType {
    Credit,
    Debit,
    SaldoCredit,
    SaldoDebit,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct FinanceBookingEntry {
    pub id: Uuid,
    pub finance_account_id: Uuid,
    pub finance_journal_diary_id: Uuid,
    pub booking_type: i8,
    pub booking_time: DateTime<Utc>,
    pub amount: u128,
    pub title: String,
    pub description: String,
}
