use async_session::chrono::{DateTime, Utc};
use mongodb::bson::Uuid;

use crate::{
    accounting_database::DBFinanceAccountingFunctions,
    database_handler_mongodb::DbConnectionSetting,
    datatypes::{
        AccountBalanceInfo, FinanceAccountBookingEntry, FinanceBookingRequest,
        FinanceBookingResult, FinanceJournalEntry,
    },
};

pub struct FinanceBookingHandle<'a> {
    db_connection_settings: &'a DbConnectionSetting,
    user_id: &'a Uuid,
    db_connector: &'a dyn DBFinanceAccountingFunctions,
}

impl<'a> FinanceBookingHandle<'a> {
    pub fn new(
        connection_settings: &'a DbConnectionSetting,
        user_id: &'a Uuid,
        db_connector: &'a dyn DBFinanceAccountingFunctions,
    ) -> Self {
        Self {
            db_connection_settings: connection_settings,
            user_id,
            db_connector,
        }
    }

    pub fn list_journal_entries(
        &self,
        from: Option<DateTime<Utc>>,
        till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String> {
        unimplemented!("logic for finance_account_booking_entry_list is not implemented");
    }

    pub fn list_account_booking_entries(
        &self,
        finance_account_id: &Uuid,
        from: Option<DateTime<Utc>>,
        till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        unimplemented!("logic for finance_account_booking_entry_list is not implemented");
    }

    pub fn finance_insert_booking_entry(
        &self,
        action_to_insert: &FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String> {
        unimplemented!("logic for finance_insert_booking_entry is not implemented");
    }

    pub fn calculate_balance_info(
        &self,
        accounts_to_calculate: &Vec<Uuid>,
    ) -> Result<Vec<AccountBalanceInfo>, String> {
        unimplemented!("logic for calculate_balance_info is not implemented");
    }
}
