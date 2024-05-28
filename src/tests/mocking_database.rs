#[cfg(test)]
use crate::datatypes::FinanceAccountType;
#[cfg(test)]
use mongodb::bson::Uuid;
#[cfg(test)]
use once_cell::sync::OnceCell;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
pub struct InMemoryDatabaseHandler {}

#[cfg(test)]
pub struct InMemoryDatabaseData {
    user_ids: Vec<Uuid>,
    account_types_per_user: Vec<Vec<FinanceAccountType>>,
}

#[cfg(test)]
pub static GLOBAL_IN_MEMORY_DATA: OnceCell<Mutex<InMemoryDatabaseData>> = OnceCell::new();

#[cfg(test)]
#[axum::async_trait(?Send)]
impl crate::accounting_config_database::DBFinanceConfigFunctions for InMemoryDatabaseHandler {
    async fn finance_account_type_list(
        &self,
        // _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        //_finance_account_type: &FinanceAccountType,
    ) -> Result<Vec<FinanceAccountType>, String> {
        let data_obj = GLOBAL_IN_MEMORY_DATA.get().unwrap();
        let data_obj2 = data_obj.lock().unwrap();

        let position_option = data_obj2.user_ids.iter().position(|elem| elem.eq(&user_id));
        if let Some(position) = position_option {
            let copy_list = InMemoryDatabaseData::clone_finance_account_type_vector(
                &data_obj2.account_types_per_user[position],
            );
            Ok(copy_list)
        } else {
            Err("User not found".to_string())
        }
    }

    async fn finance_account_type_upsert(
        &self,
        //_conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_type: *mut FinanceAccountType,
    ) -> Result<(), String> {
        unsafe {
            let data_obj = GLOBAL_IN_MEMORY_DATA.get();
            let data_obj2 = data_obj.unwrap();
            let mut data_obj3 = data_obj2.lock().unwrap();
            let position_option = data_obj3.user_ids.iter().position(|elem| elem.eq(&user_id));
            if let Some(position) = position_option {
                let current_list = &mut data_obj3.account_types_per_user.get_mut(position).unwrap();
                let position2_option = current_list
                    .iter()
                    .position(|elem| elem.id.eq(&(*finance_account_type).id));
                if let Some(position2) = position2_option {
                    let temp_var = finance_account_type.as_mut().unwrap();
                    let temp_var2 = temp_var.clone();
                    current_list.push(temp_var2);
                    current_list.remove(position2);
                } else {
                    let temp_var = finance_account_type.as_mut().unwrap();
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
}

#[cfg(test)]
impl InMemoryDatabaseData {
    pub fn create_in_memory_database_data_object(
        user_ids: Vec<Uuid>,
        account_types_per_user: Vec<Vec<FinanceAccountType>>,
    ) -> InMemoryDatabaseData {
        let data_obj = InMemoryDatabaseData {
            account_types_per_user: Vec::from(account_types_per_user),
            user_ids: Vec::from(user_ids),
        };
        return data_obj;
    }
    /*
        pub fn global() -> &'static TestSettingStruct {
            GLOBAL_TEST_SETTING
                .get()
                .expect("GLOBAL_TEST_SETTING is not initialized")
        }
    */
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
}
