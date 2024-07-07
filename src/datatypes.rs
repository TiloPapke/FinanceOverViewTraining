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
    pub amount: u64,
    pub title: String,
    pub description: String,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub enum BookingEntryType {
    Credit,
    Debit,
    SaldoCredit,
    SaldoDebit,
}

impl BookingEntryType {
    pub fn get_from_int(type_value: i32) -> Result<BookingEntryType, String> {
        match type_value {
            0 => std::result::Result::Ok(BookingEntryType::Credit),
            1 => std::result::Result::Ok(BookingEntryType::Debit),
            2 => std::result::Result::Ok(BookingEntryType::SaldoCredit),
            3 => std::result::Result::Ok(BookingEntryType::SaldoDebit),
            _ => Err(format!("value not supported: {}", type_value)),
        }
    }

    pub fn to_int(&self) -> i32 {
        match self {
            BookingEntryType::Credit => 0,
            BookingEntryType::Debit => 1,
            BookingEntryType::SaldoCredit => 2,
            BookingEntryType::SaldoDebit => 3,
        }
    }
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct FinanceAccountBookingEntry {
    pub id: Uuid,
    pub finance_account_id: Uuid,
    pub finance_journal_diary_id: Uuid,
    pub booking_type: BookingEntryType,
    pub booking_time: DateTime<Utc>,
    pub amount: u64,
    pub title: String,
    pub description: String,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct FinanceBookingRequest {
    pub is_simple_entry: bool,
    pub is_saldo: bool,
    pub debit_finance_account_id: Uuid,
    pub credit_finance_account_id: Uuid,
    pub booking_time: DateTime<Utc>,
    pub amount: u64,
    pub title: String,
    pub description: String,
}

#[derive(PartialEq, Deserialize, Debug, Clone)]
pub struct FinanceBookingResult {
    pub journal_entry: FinanceJournalEntry,
    pub debit_account_entry: FinanceAccountBookingEntry,
    pub credit_account_entry: FinanceAccountBookingEntry,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AccountBalanceType {
    Credit,
    Debit,
}

impl std::fmt::Display for AccountBalanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.eq(&AccountBalanceType::Credit) {
            write!(f, "credit balance");
            std::result::Result::Ok(())
        } else {
            if self.eq(&AccountBalanceType::Debit) {
                write!(f, "debit balance");
                std::result::Result::Ok(())
            } else {
                Err(std::fmt::Error)
            }
        }
    }
}

#[derive(Debug)]
pub struct AccountBalanceInfo {
    pub account_id: Uuid,
    pub balance_type: AccountBalanceType,
    pub amount: u64,
}
