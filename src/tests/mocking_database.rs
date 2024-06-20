#[cfg(test)]
use crate::database_handler_mongodb::DbConnectionSetting;
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
    ) -> Result<Vec<FinanceAccount>, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get().unwrap();
        let data_obj2 = data_obj.lock().unwrap();

        let position_option = data_obj2
            .data_per_user
            .iter()
            .position(|elem| elem.user_id.eq(&user_id));
        if let Some(position) = position_option {
            let copy_list = InMemoryDatabaseData::clone_finance_account_vector(
                &data_obj2.data_per_user[position].accounts_per_user,
            );
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
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        from: Option<DateTime<Utc>>,
        till: Option<DateTime<Utc>>,
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

            let return_object = journal_entries_list.clone();
            drop(data_obj3);
            Ok(return_object)
        } else {
            drop(data_obj3);
            Err("User not found".to_string())
        }
    }

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        from: Option<DateTime<Utc>>,
        till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        unimplemented!("trait is not implemented");
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
            let booking_entries_list = &mut data_obj3
                .data_per_user
                .get_mut(position)
                .unwrap()
                .booking_entries_per_user;
            let journal_entries_list = &mut data_obj3
                .data_per_user
                .get_mut(position)
                .unwrap()
                .journal_entries_per_user;
            let new_running_number = 1;
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
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        list_account_ids: Option<&Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String> {
        unimplemented!("trait is not implemented");
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
