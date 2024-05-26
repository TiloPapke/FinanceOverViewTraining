#[cfg(test)]
use crate::datatypes::FinanceAccountType;
#[cfg(test)]
use mongodb::bson::Uuid;
#[cfg(test)]
use std::rc::Rc;
#[cfg(test)]
#[cfg(test)]
pub struct InMemoryDatabaseHandler<'a, 'b> {
    internal_data: &'a mut Rc<InMemoryDatabaseData<'a, 'b>>,
}

#[cfg(test)]
pub struct InMemoryDatabaseData<'a, 'b> {
    user_ids: Vec<&'a mut Uuid>,
    account_types_per_user: Vec<Vec<&'b mut FinanceAccountType>>,
}

#[cfg(test)]
#[axum::async_trait(?Send)]
impl<'a, 'b> crate::accounting_config_database::DBFinanceConfigFunctions
    for InMemoryDatabaseHandler<'a, 'b>
{
    async fn finance_account_type_list(
        &self,
        // _conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        //_finance_account_type: &FinanceAccountType,
    ) -> Result<Vec<FinanceAccountType>, String> {
        let data_obj = Rc::clone(self.internal_data);
        let position_option = data_obj.user_ids.iter().position(|elem| elem.eq(&user_id));
        if let Some(position) = position_option {
            return Ok(InMemoryDatabaseHandler::clone_finance_account_type_vector(
                &data_obj.account_types_per_user[position],
            ));
        } else {
            Err("User not found".to_string())
        }
    }

    async fn finance_account_type_upsert(
        &self,
        //_conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_type: &FinanceAccountType,
    ) -> Result<(), String> {
        let mut data_obj = Rc::clone(self.internal_data);
        let data_obj2 = Rc::get_mut(&mut data_obj).unwrap();
        let position_option = data_obj2.user_ids.iter().position(|elem| elem.eq(&user_id));
        if let Some(position) = position_option {
            let current_list = &mut data_obj2.account_types_per_user.get_mut(position).unwrap();
            let position2_option = current_list
                .iter()
                .position(|elem| elem.id.eq(&finance_account_type.id));
            if let Some(position2) = position2_option {
                current_list[position2] = &mut finance_account_type.clone();
            } else {
                current_list.push(&mut finance_account_type.clone());
            }
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

#[cfg(test)]
impl<'a, 'b> InMemoryDatabaseHandler<'a, 'b> {
    pub fn new(data_obj: &'a mut Rc<InMemoryDatabaseData<'a, 'b>>) -> Self {
        Self {
            internal_data: data_obj,
        }
    }

    pub fn create_in_memory_database_data_object(
        user_ids: Vec<&'a mut Uuid>,
    ) -> InMemoryDatabaseData<'a, 'b> {
        let data_object = InMemoryDatabaseData {
            account_types_per_user: Vec::with_capacity(user_ids.len()),
            user_ids: user_ids,
        };
        return data_object;
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
        vector_in: &Vec<&'a mut FinanceAccountType>,
    ) -> Vec<FinanceAccountType> {
        let mut return_vetor: Vec<FinanceAccountType> = Vec::new();

        for some_finance_account_type in vector_in {
            return_vetor.push(InMemoryDatabaseHandler::clone_finance_account_type(
                &some_finance_account_type,
            ));
        }

        return return_vetor;
    }
}
