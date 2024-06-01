use axum::async_trait;
use futures::StreamExt;
use log::{debug, warn};
use mongodb::{
    bson::{doc, Document, Uuid},
    options::{FindOptions, UpdateOptions},
    Collection,
};

use crate::{
    convert_tools::ConvertTools,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
    datatypes::FinanceAccountType,
};

#[async_trait(?Send)]
pub trait DBFinanceConfigFunctions {
    async fn finance_account_type_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
    ) -> Result<Vec<FinanceAccountType>, String>;
    async fn finance_account_type_upsert(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_type: &FinanceAccountType,
    ) -> Result<(), String>;
}

#[async_trait(?Send)]
impl DBFinanceConfigFunctions for DbHandlerMongoDB {
    async fn finance_account_type_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
    ) -> Result<Vec<FinanceAccountType>, String> {
        // Get a handle to the deployment.
        let client_create_result = DbHandlerMongoDB::create_client_connection(&conncetion_settings);
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let accounting_type_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_ACCOUNTING_TYPES);

        let filter = doc! {"user_id":&user_id};
        let projection = doc! {"finance_acount_type_id":<i32>::from(1),
        "title":<i32>::from(1),
        "description":<i32>::from(1),};
        let options = FindOptions::builder().projection(projection).build();

        let query_execute_result = accounting_type_collection.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut result_list = Vec::new();

        while let Some(data_doc) = cursor.next().await {
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc = data_doc.unwrap();

            let some_uuid_parse_result =
                ConvertTools::get_uuid_from_document(&inner_doc, "finance_acount_type_id");
            if some_uuid_parse_result.is_err() {
                return Err(some_uuid_parse_result.unwrap_err().to_string());
            }
            let stored_title = inner_doc.get_str("title");
            if stored_title.is_err() {
                return Err(stored_title.unwrap_err().to_string());
            }
            let stored_description = inner_doc.get_str("description");
            if stored_description.is_err() {
                return Err(stored_description.unwrap_err().to_string());
            }

            let accounting_type = FinanceAccountType {
                id: some_uuid_parse_result.unwrap(),
                title: stored_title.unwrap().into(),
                description: stored_description.unwrap().into(),
            };

            result_list.push(accounting_type);
        }

        debug!(target:"app::FinanceOverView","returned {} finance account types",result_list.len());

        return Ok(result_list);
    }

    async fn finance_account_type_upsert(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_type: &FinanceAccountType,
    ) -> Result<(), String> {
        // Get a handle to the deployment.
        let client_create_result = DbHandlerMongoDB::create_client_connection(&conncetion_settings);
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let accounting_type_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_ACCOUNTING_TYPES);

        let filter = doc! {"finance_acount_type_id":&finance_account_type.id};
        let inner_doc = doc! {
            "finance_acount_type_id":&finance_account_type.id,
            "user_id": &user_id,
            "title": &finance_account_type.title,
            "description": &finance_account_type.description,
        };
        let upsert_doc = doc! {"$set": inner_doc  };
        let opts = UpdateOptions::builder().upsert(true).build();

        let upsert_result = accounting_type_collection
            .update_one(filter, upsert_doc, opts)
            .await;
        if upsert_result.is_err() {
            let upsert_err = &upsert_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",upsert_err);
            return Err(upsert_err.to_string());
        }
        let upsert_info = upsert_result.unwrap();

        if (upsert_info.matched_count > 1) || (upsert_info.modified_count > 1) {
            return Err(format!(
                "Error upserting element, matched count was {}, changed count was {}",
                upsert_info.matched_count, upsert_info.modified_count
            ));
        }

        debug!(target:"app::FinanceOverView","upserted finance accpunt type for user id {}",&user_id);

        Ok(())
    }
}
