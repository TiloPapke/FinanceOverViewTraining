use futures::executor;
use mongodb::bson::Uuid;

use crate::{
    accounting_config_database::DBFinanceConfigFunctions,
    database_handler_mongodb::DbConnectionSetting,
    datatypes::{FinanceAccount, FinanceAccountType},
};

pub struct FinanceAccountingConfigHandle<'a> {
    db_connection_settings: &'a DbConnectionSetting,
    user_id: &'a Uuid,
    db_connector: &'a dyn DBFinanceConfigFunctions,
}

impl<'a> FinanceAccountingConfigHandle<'a> {
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

    pub fn finance_account_list(
        &self,
        limit_account_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<FinanceAccount>, String> {
        let temp_var_1 = executor::block_on(self.finance_account_list_async(limit_account_ids));
        return temp_var_1;
    }

    pub async fn finance_account_list_async(
        &self,
        limit_account_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<FinanceAccount>, String> {
        let temp_var_1 = self
            .db_connector
            .finance_account_list(
                &self.db_connection_settings,
                &self.user_id,
                limit_account_ids,
            )
            .await;
        return temp_var_1;
    }

    pub fn finance_account_upsert(
        &mut self,
        finance_account: &FinanceAccount,
    ) -> Result<(), String> {
        let temp_var_0 = executor::block_on(
            self.db_connector
                .finance_account_type_list(&self.db_connection_settings, &self.user_id),
        );
        if temp_var_0.is_err() {
            return Err(format!(
                "Err upserting finance account, could not get list of available account types: {}",
                temp_var_0.unwrap_err()
            ));
        }

        let available_types = temp_var_0.unwrap();
        let position_option = available_types
            .iter()
            .position(|elem| elem.id.eq(&finance_account.finance_account_type_id));
        if position_option.is_none() {
            return Err(
                "could not upsert finance account because account type is not available".into(),
            );
        }

        let temp_var_1 = self.db_connector.finance_account_upsert(
            &self.db_connection_settings,
            &self.user_id,
            finance_account,
        );
        let temp_var_2 = executor::block_on(temp_var_1);
        return temp_var_2;
    }
}
