use futures::executor;
use mongodb::bson::Uuid;

use crate::{
    accounting_config_database::DBFinanceConfigFunctions,
    database_handler_mongodb::DbConnectionSetting, datatypes::FinanceAccountType,
};

pub struct FinanceAccounttingHandle<'a> {
    db_connection_settings: &'a DbConnectionSetting,
    user_id: &'a Uuid,
    db_connector: &'a dyn DBFinanceConfigFunctions,
}

impl<'a> FinanceAccounttingHandle<'a> {
    pub fn new(
        connection_settings: &'a DbConnectionSetting,
        user_id: &'a Uuid,
        db_connector: &'a dyn DBFinanceConfigFunctions,
    ) -> Self {
        Self {
            db_connection_settings: connection_settings,
            user_id,
            db_connector,
        }
    }

    pub fn finance_account_type_list(&self) -> Result<Vec<FinanceAccountType>, String> {
        let temp_var_1 = executor::block_on(
            self.db_connector
                .finance_account_type_list(&self.db_connection_settings, &self.user_id),
        );

        return temp_var_1;
    }

    pub fn finance_account_type_upsert(
        &mut self,
        finance_account_type: &mut FinanceAccountType,
    ) -> Result<(), String> {
        let temp_var_0 = self.db_connector.finance_account_type_upsert(
            &self.db_connection_settings,
            &self.user_id,
            finance_account_type,
        );
        let temp_var_1 = executor::block_on(temp_var_0);
        return temp_var_1;
    }
}
