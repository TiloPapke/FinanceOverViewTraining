use std::collections::HashMap;

use async_session::chrono::{DateTime, Utc};
use axum::async_trait;
use mongodb::bson::Uuid;

use crate::{
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
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
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String>;

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
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
        list_account_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String>;
}

#[async_trait(?Send)]
impl DBFinanceAccountingFunctions for DbHandlerMongoDB {
    async fn finance_journal_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String> {
        panic!("method finance_journal_entry_list for DbHandlerMongoDB not implemented");
    }

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        panic!("method finance_account_booking_entry_list for DbHandlerMongoDB not implemented");
    }

    async fn finance_insert_booking_entry(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        action_to_insert: FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String> {
        panic!("method finance_insert_booking_entry for DbHandlerMongoDB not implemented");
    }

    async fn finance_get_last_saldo_account_entries(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        list_account_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String> {
        panic!(
            "method finance_get_last_saldo_account_entries for DbHandlerMongoDB not implemented"
        );
    }
}
