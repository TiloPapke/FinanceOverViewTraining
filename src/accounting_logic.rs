use async_session::chrono::{DateTime, Utc};
use futures::executor;
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
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String> {
        if booking_time_from.is_some() && booking_time_till.is_some() {
            if booking_time_from.unwrap().gt(&booking_time_till.unwrap()) {
                return Err(
                    "could not query because booking_time_from is after booking_time_till".into(),
                );
            }
        }
        let temp_var_1 = executor::block_on(self.db_connector.finance_journal_entry_list(
            &self.db_connection_settings,
            &self.user_id,
            booking_time_from,
            booking_time_till,
        ));

        return temp_var_1;
    }

    pub fn list_account_booking_entries(
        &self,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        if booking_time_from.is_some() && booking_time_till.is_some() {
            if booking_time_from.unwrap().gt(&booking_time_till.unwrap()) {
                return Err(
                    "could not query because booking_time_from is after booking_time_till".into(),
                );
            }
        }
        let temp_var_1 = executor::block_on(self.db_connector.finance_account_booking_entry_list(
            &self.db_connection_settings,
            &self.user_id,
            finance_account_id,
            booking_time_from,
            booking_time_till,
        ));

        return temp_var_1;
    }

    pub fn finance_insert_booking_entry(
        &self,
        action_to_insert: &FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String> {
        let check_journal_entries_result = self.list_journal_entries(
            Some(action_to_insert.booking_time),
            Some(action_to_insert.booking_time),
        );
        if check_journal_entries_result.is_err() {
            return Err(format!(
                "Error checking already existing entries: {}",
                check_journal_entries_result.unwrap_err()
            ));
        }
        let check_journal_entries = check_journal_entries_result.unwrap();
        if check_journal_entries.len() > 0 {
            let position_credit_result = check_journal_entries.iter().position(|elem| {
                elem.credit_finance_account_id
                    .eq(&action_to_insert.credit_finance_account_id)
                    || elem
                        .debit_finance_account_id
                        .eq(&action_to_insert.credit_finance_account_id)
            });
            if position_credit_result.is_some() {
                return Err("Could not perform request: there is already an entry for credit account at specified booking time".into());
            }
            let position_debit_result = check_journal_entries.iter().position(|elem| {
                elem.credit_finance_account_id
                    .eq(&action_to_insert.debit_finance_account_id)
                    || elem
                        .debit_finance_account_id
                        .eq(&action_to_insert.debit_finance_account_id)
            });
            if position_debit_result.is_some() {
                return Err("Could not perform request: there is already an entry for debit account at specified booking time".into());
            }
        }

        let temp_var_0 = self.db_connector.finance_insert_booking_entry(
            &self.db_connection_settings,
            &self.user_id,
            action_to_insert.clone(),
        );
        let temp_var_1 = executor::block_on(temp_var_0);
        return temp_var_1;
    }

    pub fn calculate_balance_info(
        &self,
        accounts_to_calculate: &Vec<Uuid>,
    ) -> Result<Vec<AccountBalanceInfo>, String> {
        unimplemented!("logic for calculate_balance_info is not implemented");
    }
}
