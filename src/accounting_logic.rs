use std::collections::HashMap;

use async_session::chrono::{DateTime, Utc};
use futures::executor;
use mongodb::bson::Uuid;

use crate::{
    accounting_database::{
        DBFinanceAccountingFunctions, FinanceAccountBookingEntryListSearchOption,
    },
    database_handler_mongodb::DbConnectionSetting,
    datatypes::{
        AccountBalanceInfo, AccountBalanceType, BookingEntryType, FinanceAccountBookingEntry,
        FinanceBookingRequest, FinanceBookingResult, FinanceJournalEntry,
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

    pub async fn list_journal_entries(
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
        let temp_var_1 = self
            .db_connector
            .finance_journal_entry_list(
                &self.db_connection_settings,
                &self.user_id,
                booking_time_from,
                booking_time_till,
            )
            .await;

        return temp_var_1;
    }

    pub fn list_account_booking_entries(
        &self,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        let search_option = FinanceAccountBookingEntryListSearchOption::new(
            finance_account_id,
            booking_time_from,
            booking_time_till,
        );
        let temp_var_1 = self.list_account_booking_entries_multi(vec![search_option]);

        return temp_var_1;
    }

    pub fn list_account_booking_entries_multi(
        &self,
        search_options: Vec<FinanceAccountBookingEntryListSearchOption>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        if search_options.len() == 0 {
            return Err("could not query because search options is empty".into());
        }
        for search_option in &search_options {
            if search_option.booking_time_from.is_some()
                && search_option.booking_time_till.is_some()
            {
                if search_option
                    .booking_time_from
                    .unwrap()
                    .gt(&search_option.booking_time_till.unwrap())
                {
                    return Err(
                        "could not query because booking_time_from is after booking_time_till"
                            .into(),
                    );
                }
            }
        }
        let temp_var_1 =
            executor::block_on(self.db_connector.finance_account_booking_entry_list_multi(
                &self.db_connection_settings,
                &self.user_id,
                search_options,
            ));

        return temp_var_1;
    }

    pub async fn finance_insert_booking_entry(
        &self,
        action_to_insert: &FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String> {
        let check_journal_entries_result = self
            .list_journal_entries(
                Some(action_to_insert.booking_time),
                Some(action_to_insert.booking_time),
            )
            .await;
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
                    && elem.booking_time.eq(&action_to_insert.booking_time)
            });
            if position_credit_result.is_some() {
                return Err("Could not perform request: there is already an entry for credit account at specified booking time".into());
            }
            let position_debit_result = check_journal_entries.iter().position(|elem| {
                elem.debit_finance_account_id
                    .eq(&action_to_insert.debit_finance_account_id)
                    && elem.booking_time.eq(&action_to_insert.booking_time)
            });
            if position_debit_result.is_some() {
                return Err("Could not perform request: there is already an entry for debit account at specified booking time".into());
            }
        }

        let saldo_information_result =
            executor::block_on(self.finance_get_last_saldo_account_entries(Some(vec![
                action_to_insert.credit_finance_account_id,
                action_to_insert.debit_finance_account_id,
            ])));
        if saldo_information_result.is_err() {
            return Err(format!(
                "Error checking already existing entries: {}",
                saldo_information_result.unwrap_err()
            ));
        }
        let saldo_information = saldo_information_result.unwrap();
        if saldo_information.contains_key(&action_to_insert.credit_finance_account_id) {
            let credit_account_saldo_entry =
                saldo_information.get(&action_to_insert.credit_finance_account_id);
            if credit_account_saldo_entry.is_some() {
                if action_to_insert
                    .booking_time
                    .le(&credit_account_saldo_entry.unwrap().booking_time)
                {
                    return Err("Can not insert before saldo of credit account".into());
                }
            }
        }
        if saldo_information.contains_key(&action_to_insert.debit_finance_account_id) {
            let debit_account_saldo_entry =
                saldo_information.get(&action_to_insert.debit_finance_account_id);
            if debit_account_saldo_entry.is_some() {
                if action_to_insert
                    .booking_time
                    .le(&debit_account_saldo_entry.unwrap().booking_time)
                {
                    return Err("Can not insert before saldo of debit account".into());
                }
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
        let mut return_object: Vec<AccountBalanceInfo> = Vec::new();
        // at first get last saldo information per account

        let saldo_information_list_result = executor::block_on(
            self.finance_get_last_saldo_account_entries(Some(accounts_to_calculate.clone())),
        );

        if saldo_information_list_result.is_err() {
            return Err(format!(
                "Error getting saldo information: {}",
                saldo_information_list_result.unwrap_err()
            ));
        }

        let saldo_information_list = saldo_information_list_result.unwrap();

        let mut search_options = Vec::new();
        for account_id in accounts_to_calculate {
            let time_start_option;
            if saldo_information_list.contains_key(account_id) {
                let saldo_datetime = saldo_information_list.get(account_id).unwrap().booking_time;
                time_start_option = Some(saldo_datetime);
            } else {
                time_start_option = None;
            }
            let search_option = FinanceAccountBookingEntryListSearchOption::new(
                account_id,
                time_start_option,
                None,
            );
            search_options.push(search_option);
        }
        let booking_entries_multi_result = self.list_account_booking_entries_multi(search_options);
        if booking_entries_multi_result.is_err() {
            return Err(format!(
                "Error getting multi booking entries: {}",
                booking_entries_multi_result.unwrap_err()
            ));
        }
        let booking_entries_multi = booking_entries_multi_result.unwrap();

        // per account get entries since last saldo
        for account_id in accounts_to_calculate {
            let mut sum_credit_amount = 0;
            let mut sum_debit_amount = 0;
            let booking_entries: std::iter::Filter<
                std::slice::Iter<FinanceAccountBookingEntry>,
                _,
            > = booking_entries_multi
                .iter()
                .filter(|elem| elem.finance_account_id.eq(account_id));
            for booking_entry in booking_entries {
                if booking_entry.booking_type.eq(&BookingEntryType::Credit)
                    || booking_entry
                        .booking_type
                        .eq(&BookingEntryType::SaldoCredit)
                {
                    sum_credit_amount += booking_entry.amount;
                } else {
                    sum_debit_amount += booking_entry.amount;
                }
            }

            let balance_amount = sum_credit_amount.abs_diff(sum_debit_amount);
            let balance_type = if sum_credit_amount.gt(&sum_debit_amount) {
                AccountBalanceType::Credit
            } else {
                AccountBalanceType::Debit
            };
            let balance_info = AccountBalanceInfo {
                account_id: account_id.clone(),
                amount: balance_amount,
                balance_type,
            };

            return_object.push(balance_info);
        }

        let temp_var0 = Result::Ok(return_object);
        return temp_var0;
    }

    pub async fn finance_get_last_saldo_account_entries(
        &self,
        list_account_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String> {
        let value = self
            .db_connector
            .finance_get_last_saldo_account_entries(
                &self.db_connection_settings,
                &self.user_id,
                list_account_ids,
            )
            .await;
        return value;
    }
}
