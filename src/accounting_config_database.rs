use axum::async_trait;
use mongodb::bson::Uuid;

use crate::{database_handler_mongodb::DbHandlerMongoDB, datatypes::FinanceAccountType};

#[async_trait(?Send)]
pub trait DBFinanceConfigFunctions {
    async fn finance_account_type_list(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<FinanceAccountType>, String>;
    async fn finance_account_type_upsert(
        &self,
        user_id: &Uuid,
        finance_account_type: *mut FinanceAccountType,
    ) -> Result<(), String>;
}

#[async_trait(?Send)]
impl DBFinanceConfigFunctions for DbHandlerMongoDB {
    async fn finance_account_type_list(
        &self,
        _user_id: &Uuid,
    ) -> Result<Vec<FinanceAccountType>, String> {
        // Get a handle to the deployment.
        /*
        let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        */

        unimplemented!("Listing finance account type not impplemented")

        //debug!(target:"app::FinanceOverView","new id of finance account type {}",insert_result.unwrap().inserted_id);
    }

    async fn finance_account_type_upsert(
        &self,
        _user_id: &Uuid,
        _finance_account_type: *mut FinanceAccountType,
    ) -> Result<(), String> {
        // Get a handle to the deployment.
        /*
        let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        */
        unimplemented!("upsert new finance account type not impplemented")

        //debug!(target:"app::FinanceOverView","new id of finance account type {}",insert_result.unwrap().inserted_id);
    }
}
