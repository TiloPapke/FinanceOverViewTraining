use std::collections::HashMap;

use async_session::chrono::{DateTime, Utc};
use axum::async_trait;
use mongodb::bson::Uuid;

use crate::{
    database_handler_mongodb::DbConnectionSetting,
    datatypes::{
        FinanceAccountBookingEntry, FinanceBookingRequest, FinanceBookingResult,
        FinanceJournalEntry,
    },
};

#[async_trait(?Send)]
pub trait DBFinanceAccountingFunctions {
    async fn finance_journal_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        from: Option<DateTime<Utc>>,
        till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String>;

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        from: Option<DateTime<Utc>>,
        till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String>;

    async fn finance_insert_booking_entry(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        action_to_insert: FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String>;

    async fn finance_get_last_saldo_account_entries(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        list_account_ids: Option<&Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String>;
}