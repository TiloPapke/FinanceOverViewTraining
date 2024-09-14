use futures::{executor, StreamExt};
use mongodb::bson::{Document, Uuid};

use crate::{
    datatypes::FinanceAccount,
    html_render::{ParentAccountOption, PARENT_ACCOUNT_ID_EMPTY_WEB_STRING},
};

pub struct MdbConvertTools {}

impl MdbConvertTools {
    pub fn get_vector_from_cursor(cursor: mongodb::Cursor<Document>) -> Vec<Document> {
        let convert_result =
            executor::block_on(MdbConvertTools::get_vector_from_cursor_async(cursor));
        if convert_result.is_ok() {
            return convert_result.unwrap();
        }

        return Vec::new();
    }

    pub async fn get_vector_from_cursor_async(
        mut cursor: mongodb::Cursor<Document>,
    ) -> Result<Vec<Document>, Box<dyn std::error::Error>> {
        let mut docs = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => {
                    docs.push(document);
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        return Ok(docs);
    }

    pub fn get_binary_from_bson_uuid(input: &mongodb::bson::Uuid) -> mongodb::bson::Binary {
        let result_value = mongodb::bson::Binary::from_uuid(input.clone());
        return result_value;
    }

    pub fn get_clean_parent_account_id_for_web(parent_account_id: &Option<Uuid>) -> String {
        if parent_account_id.is_none() {
            return PARENT_ACCOUNT_ID_EMPTY_WEB_STRING.to_string();
        }
        return parent_account_id.unwrap().to_string();
    }

    pub fn get_account_name_from_account_list_via_account_id(
        account_list: &Vec<FinanceAccount>,
        account_id: &Option<Uuid>,
    ) -> String {
        if account_id.is_none() {
            return PARENT_ACCOUNT_ID_EMPTY_WEB_STRING.to_string();
        }

        let account_id_lookup = account_id.unwrap();

        let position_option = account_list
            .iter()
            .position(|elem| elem.id.eq(&account_id_lookup));
        if position_option.is_some() {
            let position = position_option.unwrap();
            return account_list[position].title.clone();
        }

        return "not found".to_string();
    }

    //this is just a simple starting function to get the html templates running might be replaced later
    pub fn get_parent_option_list_from_account_list(
        account_list: &Vec<FinanceAccount>,
    ) -> Vec<ParentAccountOption> {
        let mut result_list = Vec::new();

        for account in account_list {
            let parent_account_option = ParentAccountOption {
                id: account.id,
                name: account.title.clone(),
            };
            result_list.push(parent_account_option);
        }
        return result_list;
    }
}
