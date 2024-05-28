use mongodb::bson::Uuid;

use crate::{accounting_config_database::DBFinanceConfigFunctions, datatypes::FinanceAccountType};

pub struct FinanceAccounttingHandle<'a> {
    user_id: &'a Uuid,
    db_connector: &'a dyn DBFinanceConfigFunctions,
}

impl<'a> FinanceAccounttingHandle<'a> {
    pub fn new(user_id: &'a Uuid, db_connector: &'a dyn DBFinanceConfigFunctions) -> Self {
        Self {
            user_id,
            db_connector,
        }
    }

    pub async fn finance_account_type_list(&self) -> Result<Vec<FinanceAccountType>, String> {
        let temp_var_0 = self.db_connector.finance_account_type_list(&self.user_id);
        let temp_var_1: Result<Vec<FinanceAccountType>, String> = temp_var_0.await;
        return temp_var_1;
    }

    pub async fn finance_account_type_upsert(
        &mut self,
        finance_account_type: &mut FinanceAccountType,
    ) -> Result<(), String> {
        let temp_var_0 = self
            .db_connector
            .finance_account_type_upsert(&self.user_id, finance_account_type);
        let temp_var_1 = temp_var_0.await;
        return temp_var_1;
    }
}
