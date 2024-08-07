#[cfg(test)]
use crate::accounting_database::FinanceAccountBookingEntryListSearchOption;
#[cfg(test)]
use crate::database_handler_mongodb::DbConnectionSetting;
#[cfg(test)]
use crate::datatypes::BookingEntryType;
#[cfg(test)]
use crate::datatypes::FinanceAccount;
#[cfg(test)]
use crate::datatypes::FinanceAccountType;
#[cfg(test)]
use crate::datatypes::{
    FinanceAccountBookingEntry, FinanceBookingRequest, FinanceBookingResult, FinanceJournalEntry,
};
#[cfg(test)]
use async_session::chrono::{DateTime, Utc};
#[cfg(test)]
use mongodb::bson::Uuid;
#[cfg(test)]
use once_cell::sync::OnceCell;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
pub struct InMemoryDatabaseHandler {}

#[cfg(test)]
pub struct InMemoryDatabaseEntryObj {
    user_id: Uuid,
    account_types_per_user: Vec<FinanceAccountType>,
    accounts_per_user: Vec<FinanceAccount>,
    booking_entries_per_user: Vec<FinanceAccountBookingEntry>,
    journal_entries_per_user: Vec<FinanceJournalEntry>,
}

#[cfg(test)]
pub struct InMemoryDatabaseData {
    data_per_user: Vec<InMemoryDatabaseEntryObj>,
}

#[cfg(test)]
pub static GLOBAL_IN_MEMORY_DATA: OnceCell<Mutex<InMemoryDatabaseData>> = OnceCell::new();

#[cfg(test)]
#[axum::async_trait(?Send)]
impl crate::accounting_config_database::DBFinanceConfigFunctions for InMemoryDatabaseHandler {
    async fn finance_account_type_list(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
    ) -> Result<Vec<FinanceAccountType>, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get().unwrap();
        let data_obj2 = data_obj.lock().unwrap();

        let position_option = data_obj2
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            let copy_list = InMemoryDatabaseData::clone_finance_account_type_vector(
                &data_obj2.data_per_user[position].account_types_per_user,
            );
            Ok(copy_list)
        } else {
            Err("User not found".to_string())
        }
    }

    async fn finance_account_type_upsert(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_type: &FinanceAccountType,
    ) -> Result<(), String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get();
        let data_obj2 = data_obj.unwrap();
        let mut data_obj3 = data_obj2.lock().unwrap();
        let position_option = data_obj3
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            //let current_list = &mut data_obj3.account_types_per_user.get_mut(position).unwrap();
            let current_list = &mut data_obj3
                .data_per_user
                .get_mut(position)
                .unwrap()
                .account_types_per_user;
            let position2_option = current_list
                .iter()
                .position(|elem| elem.id.eq(&(*finance_account_type).id));
            if let Some(position2) = position2_option {
                let temp_var = finance_account_type;
                let temp_var2 = temp_var.clone();
                current_list.push(temp_var2);
                current_list.remove(position2);
            } else {
                let temp_var = finance_account_type;
                let temp_var2 = temp_var.clone();
                current_list.push(temp_var2);
            }
            drop(data_obj3);
            Ok(())
        } else {
            drop(data_obj3);
            Err("User not found".to_string())
        }
    }
    async fn finance_account_list(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        limit_account_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<FinanceAccount>, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get().unwrap();
        let data_obj2 = data_obj.lock().unwrap();

        let position_option = data_obj2
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            let mut copy_list = InMemoryDatabaseData::clone_finance_account_vector(
                &data_obj2.data_per_user[position].accounts_per_user,
            );
            if limit_account_ids.is_some() {
                let limit_list = limit_account_ids.unwrap();
                copy_list.retain(|elem| limit_list.contains(&&elem.id));
            }
            drop(data_obj2);
            Ok(copy_list)
        } else {
            drop(data_obj2);
            Err("User not found".to_string())
        }
    }

    async fn finance_account_upsert(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account: &FinanceAccount,
    ) -> Result<(), String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get();
        let data_obj2 = data_obj.unwrap();
        let mut data_obj3 = data_obj2.lock().unwrap();
        let position_option = data_obj3
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            //let current_list = &mut data_obj3.account_per_user.get_mut(position).unwrap();
            let current_list = &mut data_obj3
                .data_per_user
                .get_mut(position)
                .unwrap()
                .accounts_per_user;
            let position2_option = current_list
                .iter()
                .position(|elem| elem.id.eq(&finance_account.id));
            if let Some(position2) = position2_option {
                let temp_var = finance_account;
                let temp_var2 = temp_var.clone();
                current_list.push(temp_var2);
                current_list.remove(position2);
            } else {
                let temp_var = finance_account;
                let temp_var2 = temp_var.clone();
                current_list.push(temp_var2);
            }
            drop(data_obj3);
            Ok(())
        } else {
            drop(data_obj3);
            Err("User not found".to_string())
        }
    }
}

#[cfg(test)]
#[axum::async_trait(?Send)]
impl crate::accounting_database::DBFinanceAccountingFunctions for InMemoryDatabaseHandler {
    async fn finance_journal_entry_list(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get();
        let data_obj2 = data_obj.unwrap();
        let data_obj3 = data_obj2.lock().unwrap();
        let position_option = data_obj3
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            let journal_entries_list = &data_obj3
                .data_per_user
                .get(position)
                .unwrap()
                .journal_entries_per_user;

            let mut return_object = journal_entries_list.clone();
            if booking_time_from.is_some() {
                return_object.retain(|elem| elem.booking_time.ge(&booking_time_from.unwrap()))
            }
            if booking_time_till.is_some() {
                return_object.retain(|elem| elem.booking_time.le(&booking_time_till.unwrap()))
            }
            drop(data_obj3);
            Ok(return_object)
        } else {
            drop(data_obj3);
            Err("User not found".to_string())
        }
    }

    async fn finance_account_booking_entry_list(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        search_options: Vec<FinanceAccountBookingEntryListSearchOption>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get();
        let data_obj2 = data_obj.unwrap();
        let data_obj3 = data_obj2.lock().unwrap();
        let position_option = data_obj3
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            let user_object = &data_obj3.data_per_user.get(position).unwrap();
            let booking_entries_list = &user_object.booking_entries_per_user;

            let account_list = &user_object.accounts_per_user;
            for search_option in &search_options {
                let account_position_option = account_list
                    .iter()
                    .position(|elem| elem.id.eq(&search_option.finance_account_id));
                if account_position_option.is_none() {
                    return Err(format!(
                        "account {} not avaiable",
                        search_option.finance_account_id
                    ));
                }
            }

            let mut return_object = Vec::new();
            for search_option in &search_options {
                let mut list_per_account = booking_entries_list.clone();
                list_per_account.retain(|elem| {
                    elem.finance_account_id
                        .eq(&search_option.finance_account_id)
                });
                if search_option.booking_time_from.is_some() {
                    list_per_account.retain(|elem| {
                        elem.booking_time
                            .ge(&search_option.booking_time_from.unwrap())
                    })
                }
                if search_option.booking_time_till.is_some() {
                    list_per_account.retain(|elem| {
                        elem.booking_time
                            .le(&search_option.booking_time_till.unwrap())
                    })
                }
                return_object.append(&mut list_per_account);
            }
            drop(data_obj3);
            Ok(return_object)
        } else {
            drop(data_obj3);
            Err("User not found".to_string())
        }
    }

    async fn finance_insert_booking_entry(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        action_to_insert: FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get();
        let data_obj2 = data_obj.unwrap();
        let mut data_obj3 = data_obj2.lock().unwrap();
        let position_option = data_obj3
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            //let current_list = &mut data_obj3.account_per_user.get_mut(position).unwrap();
            let user_object = &mut data_obj3.data_per_user.get_mut(position).unwrap();
            let booking_entries_list = &mut user_object.booking_entries_per_user;
            let journal_entries_list = &mut user_object.journal_entries_per_user;

            let account_list = &user_object.accounts_per_user;
            let check_credit_account_check_option = account_list
                .iter()
                .position(|elem| elem.id.eq(&action_to_insert.credit_finance_account_id));
            if check_credit_account_check_option.is_none() {
                return Err("credit account is not available".into());
            }
            let check_debit_account_check_option = account_list
                .iter()
                .position(|elem| elem.id.eq(&action_to_insert.debit_finance_account_id));
            if check_debit_account_check_option.is_none() {
                return Err("debit account is not available".into());
            }

            let max_current_running_number_option = journal_entries_list
                .iter()
                .max_by_key(|elem| elem.running_number);
            let max_current_running_number = if max_current_running_number_option.is_some() {
                max_current_running_number_option.unwrap().running_number
            } else {
                0
            };
            let new_running_number = max_current_running_number + 1;

            let new_journal_entry = FinanceJournalEntry {
                id: Uuid::new(),
                is_simple_entry: action_to_insert.is_simple_entry,
                is_saldo: action_to_insert.is_saldo,
                debit_finance_account_id: action_to_insert.debit_finance_account_id,
                credit_finance_account_id: action_to_insert.credit_finance_account_id,
                running_number: new_running_number,
                booking_time: action_to_insert.booking_time,
                amount: action_to_insert.amount,
                title: action_to_insert.title.clone(),
                description: action_to_insert.description.clone(),
            };
            let credit_booking_type = if action_to_insert.is_saldo {
                BookingEntryType::SaldoCredit
            } else {
                BookingEntryType::Credit
            };
            let new_credit_account_entry = FinanceAccountBookingEntry {
                id: Uuid::new(),
                finance_account_id: action_to_insert.credit_finance_account_id,
                finance_journal_diary_id: new_journal_entry.id.clone(),
                booking_type: credit_booking_type,
                booking_time: action_to_insert.booking_time,
                amount: action_to_insert.amount,
                title: action_to_insert.title.clone(),
                description: action_to_insert.description.clone(),
            };
            let debit_booking_type = if action_to_insert.is_saldo {
                BookingEntryType::SaldoDebit
            } else {
                BookingEntryType::Debit
            };
            let new_debit_account_entry = FinanceAccountBookingEntry {
                id: Uuid::new(),
                finance_account_id: action_to_insert.debit_finance_account_id,
                finance_journal_diary_id: new_journal_entry.id.clone(),
                booking_type: debit_booking_type,
                booking_time: action_to_insert.booking_time,
                amount: action_to_insert.amount,
                title: action_to_insert.title.clone(),
                description: action_to_insert.description.clone(),
            };

            journal_entries_list.push(new_journal_entry.clone());
            booking_entries_list.push(new_credit_account_entry.clone());
            booking_entries_list.push(new_debit_account_entry.clone());

            let return_object = FinanceBookingResult {
                journal_entry: new_journal_entry,
                debit_account_entry: new_debit_account_entry,
                credit_account_entry: new_credit_account_entry,
            };
            drop(data_obj3);
            Ok(return_object)
        } else {
            drop(data_obj3);
            Err("User not found".to_string())
        }
    }

    async fn finance_get_last_saldo_account_entries(
        &self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        list_account_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get();
        let data_obj2 = data_obj.unwrap();
        let data_obj3 = data_obj2.lock().unwrap();
        let position_option = data_obj3
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            let booking_entries_list = &data_obj3
                .data_per_user
                .get(position)
                .unwrap()
                .booking_entries_per_user;

            let mut return_object = HashMap::new();

            let account_ids_to_check = match list_account_ids {
                Some(id_list) => id_list,
                None => booking_entries_list
                    .iter()
                    .map(|elem| elem.finance_account_id)
                    .collect::<Vec<Uuid>>(),
            };

            for account_id_to_check in account_ids_to_check {
                let saldo_entries_per_account: Vec<&FinanceAccountBookingEntry> =
                    booking_entries_list
                        .iter()
                        .filter(|elem| {
                            elem.finance_account_id.eq(&account_id_to_check)
                                && (elem.booking_type.eq(&BookingEntryType::SaldoCredit)
                                    || elem.booking_type.eq(&BookingEntryType::SaldoDebit))
                        })
                        .collect();

                let oldest_saldo_entry_option = saldo_entries_per_account
                    .iter()
                    .max_by_key(|elem| elem.booking_time);
                if oldest_saldo_entry_option.is_some() {
                    return_object.insert(
                        account_id_to_check,
                        oldest_saldo_entry_option.unwrap().to_owned().clone(),
                    );
                }
            }

            drop(data_obj3);
            Ok(return_object)
        } else {
            drop(data_obj3);
            Err("User not found".to_string())
        }
    }
}

#[cfg(test)]
impl InMemoryDatabaseData {
    pub fn insert_in_memory_database(
        mut data_per_user: Vec<InMemoryDatabaseEntryObj>,
    ) -> Result<(), String> {
        let mut data_obj = GLOBAL_IN_MEMORY_DATA.get();
        if data_obj.is_none() {
            let new_data_obj = InMemoryDatabaseData {
                data_per_user: Vec::new(),
            };
            let mutex_obj = Mutex::new(new_data_obj);

            let _ = GLOBAL_IN_MEMORY_DATA.set(mutex_obj);
            data_obj = GLOBAL_IN_MEMORY_DATA.get();
            if data_obj.is_none() {
                return Err("Could not initialise InMemoryDB".into());
            }
        }
        let data_obj2 = data_obj.unwrap();
        let mut data_obj3 = data_obj2.lock().unwrap();

        data_obj3.data_per_user.append(&mut data_per_user);
        drop(data_obj3);
        return Ok(());
    }

    pub fn create_in_memory_database_entry_object(user_id: &Uuid) -> InMemoryDatabaseEntryObj {
        return InMemoryDatabaseEntryObj {
            user_id: user_id.clone(),
            account_types_per_user: Vec::new(),
            accounts_per_user: Vec::new(),
            booking_entries_per_user: Vec::new(),
            journal_entries_per_user: Vec::new(),
        };
    }
    fn clone_finance_account_type(object_to_clone: &FinanceAccountType) -> FinanceAccountType {
        let return_obj = FinanceAccountType {
            id: object_to_clone.id,
            title: object_to_clone.title.to_owned(),
            description: object_to_clone.description.to_owned(),
        };
        return return_obj;
    }
    fn clone_finance_account_type_vector(
        vector_in: &Vec<FinanceAccountType>,
    ) -> Vec<FinanceAccountType> {
        let mut return_vetor: Vec<FinanceAccountType> = Vec::with_capacity(vector_in.len());

        for some_finance_account_type in vector_in {
            let temp_var =
                InMemoryDatabaseData::clone_finance_account_type(some_finance_account_type);
            return_vetor.push(temp_var);
        }

        return return_vetor;
    }

    fn clone_finance_account(object_to_clone: &FinanceAccount) -> FinanceAccount {
        let return_obj = FinanceAccount {
            id: object_to_clone.id,
            finance_account_type_id: object_to_clone.finance_account_type_id,
            title: object_to_clone.title.to_owned(),
            description: object_to_clone.description.to_owned(),
        };
        return return_obj;
    }
    fn clone_finance_account_vector(vector_in: &Vec<FinanceAccount>) -> Vec<FinanceAccount> {
        let mut return_vetor: Vec<FinanceAccount> = Vec::with_capacity(vector_in.len());

        for some_finance_account in vector_in {
            let temp_var = InMemoryDatabaseData::clone_finance_account(some_finance_account);
            return_vetor.push(temp_var);
        }

        return return_vetor;
    }
}
