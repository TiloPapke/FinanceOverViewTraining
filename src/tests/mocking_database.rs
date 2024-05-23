use axum::async_trait;
use mongodb::bson::Uuid;
use std::collections::HashMap;

use crate::{
    accounting_config_database::DBFinanceConfigFunctions,
    database_handler_mongodb::DbConnectionSetting, datatypes::FinanceAccountType,
};

pub struct InMemoryDatabase {
    user_account_types: HashMap<Uuid, Vec<FinanceAccountType>>,
}

#[async_trait]
impl DBFinanceConfigFunctions for InMemoryDatabase {
    async fn finance_account_type_list(
        &mut self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        _finance_account_type: &FinanceAccountType,
    ) -> Result<Vec<FinanceAccountType>, String> {
        let empty_value: Vec<FinanceAccountType> = Vec::new();
        if self.user_account_types.contains_key(user_id) {
            match self.user_account_types.get(user_id) {
                Some(return_value) => Ok(InMemoryDatabase::clone_finance_account_type_vector(
                    return_value,
                )),
                None => Ok(empty_value),
            }
        } else {
            Err("User not found".to_string())
        }
    }

    async fn finance_account_type_upsert(
        &mut self,
        _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_type: &FinanceAccountType,
    ) -> Result<(), String> {
        let current_list_option = self.user_account_types.get_mut(&user_id);
        if let Some(current_list) = current_list_option {
            let index = current_list
                .iter()
                .position(|elem| elem.id.eq(&finance_account_type.id));
            match index {
                Some(position) => current_list[position] = finance_account_type.clone(),
                None => current_list.push(finance_account_type.clone()),
            }
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

impl InMemoryDatabase {
    fn clone_finance_account_type(object_to_clone: &FinanceAccountType) -> FinanceAccountType {
        let return_obj = FinanceAccountType {
            id: object_to_clone.id,
            title: object_to_clone.title.to_owned(),
            descriptiom: object_to_clone.descriptiom.to_owned(),
        };
        return return_obj;
    }
    fn clone_finance_account_type_vector(
        vector_in: &Vec<FinanceAccountType>,
    ) -> Vec<FinanceAccountType> {
        let mut return_vetor: Vec<FinanceAccountType> = Vec::new();

        for some_finance_account_type in vector_in {
            return_vetor.push(InMemoryDatabase::clone_finance_account_type(
                &some_finance_account_type,
            ));
        }

        return return_vetor;
    }
}
