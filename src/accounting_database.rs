use std::collections::HashMap;

use async_session::chrono::{DateTime, Utc};
use axum::async_trait;
use futures::StreamExt;
use log::{debug, warn};
use mongodb::{
    bson::{doc, Document, Uuid},
    options::FindOptions,
    Collection,
};

use crate::{
    accounting_config_logic::FinanceAccountingConfigHandle,
    convert_tools::ConvertTools,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
    datatypes::{
        BookingEntryType, FinanceAccountBookingEntry, FinanceBookingRequest, FinanceBookingResult,
        FinanceJournalEntry,
    }, mdb_convert_tools::MdbConvertTools,
};

#[async_trait(?Send)]
pub trait DBFinanceAccountingFunctions {
    async fn finance_journal_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String>;

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String>;

    async fn finance_insert_booking_entry(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        action_to_insert: FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String>;

    async fn finance_get_last_saldo_account_entries(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        list_account_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String>;
}

#[async_trait(?Send)]
impl DBFinanceAccountingFunctions for DbHandlerMongoDB {
    async fn finance_journal_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String> {
        panic!("method finance_journal_entry_list for DbHandlerMongoDB not implemented");
    }

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
              // Get a handle to the deployment.
              let client_create_result =
              DbHandlerMongoDB::create_client_connection_async(&conncetion_settings).await;
          if client_create_result.is_err() {
              let client_err = &client_create_result.unwrap_err();
              warn!(target:"app::FinanceOverView","{}",client_err);
              return Err(client_err.to_string());
          }
          let client = client_create_result.unwrap();
  
          let db_instance = client.database(&conncetion_settings.instance);

        let accounting_handle =
            FinanceAccountingConfigHandle::new(&conncetion_settings, &user_id, self);
        let account_list_result = accounting_handle.finance_account_list_async().await;
        if account_list_result.is_err() {
            return Err(format!(
                "Error retriving account list: {}",
                account_list_result.unwrap_err()
            ));
        }

        let account_list = account_list_result.unwrap();
        let account_position_option = account_list
            .iter()
            .position(|elem| elem.id.eq(&finance_account_id));
        if account_position_option.is_none() {
            return Err("account not avaiable".to_string());
        }

        let booking_entries_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_BOOKING_ENTRIES);

         //get a binary of UUID or it will not work in production
        let mut filter = doc! {"user_id":MdbConvertTools::get_binary_from_bson_uuid(user_id),
            "finance_account_id": MdbConvertTools::get_binary_from_bson_uuid(finance_account_id)};
            if booking_time_from.is_some() {
                filter.insert("booking_time", doc!{"$gte": booking_time_from.unwrap()});
            }
            if booking_time_till.is_some() {
                filter.insert("booking_time", doc!{"$lte": booking_time_from.unwrap()});
            }


            debug!(target:"app::FinanceOverView","Filter document: {}",&filter);
            let projection = doc! {"booking_entry_id":<i32>::from(1),
            "finance_account_id":<i32>::from(1),
            "finance_journal_diary_id":<i32>::from(1),
            "booking_type":<i32>::from(1),
            "booking_time":<i32>::from(1),
            "amount":<i32>::from(1),
            "title":<i32>::from(1),
            "description":<i32>::from(1),};
            let options = FindOptions::builder().projection(projection).build();
    
            let query_execute_result = booking_entries_collection.find(filter, options).await;
    
            if query_execute_result.is_err() {
                return Result::Err(query_execute_result.unwrap_err().to_string());
            }
    
            let mut cursor = query_execute_result.unwrap();
    
            let mut booking_entries_list = Vec::new();

            while let Some(data_doc) = cursor.next().await {
                if data_doc.is_err() {
                    return Err(data_doc.unwrap_err().to_string());
                }
    
                let inner_doc = data_doc.unwrap();
    
                let some_booking_entry_id_parse_result =
                    ConvertTools::get_uuid_from_document(&inner_doc, "booking_entry_id");
                if some_booking_entry_id_parse_result.is_err() {
                    return Err(some_booking_entry_id_parse_result.unwrap_err().to_string());
                }
                let some_finance_account_id_parse_result =
                    ConvertTools::get_uuid_from_document(&inner_doc, "finance_account_id");
                if some_finance_account_id_parse_result.is_err() {
                    return Err(some_finance_account_id_parse_result
                        .unwrap_err()
                        .to_string());
                }
                let some_finance_journal_diary_id_parse_result =
                    ConvertTools::get_uuid_from_document(&inner_doc, "finance_journal_diary_id");
                if some_finance_journal_diary_id_parse_result.is_err() {
                    return Err(some_finance_journal_diary_id_parse_result
                        .unwrap_err()
                        .to_string());
                }
                let stored_booking_type_int = inner_doc.get_i32("booking_type");
                if stored_booking_type_int.is_err() {
                    return Err(stored_booking_type_int.unwrap_err().to_string());
                }
                let stored_booking_type_result =
                    BookingEntryType::get_from_int(stored_booking_type_int.unwrap());
                if stored_booking_type_result.is_err() {
                    return Err(stored_booking_type_result.unwrap_err());
                }
    
                let stored_booking_time = inner_doc.get_datetime("booking_time");
                if stored_booking_time.is_err() {
                    return Err(stored_booking_time.unwrap_err().to_string());
                }
                let stored_amount = inner_doc.get_i64("amount");
                if stored_amount.is_err() {
                    return Err(stored_amount.unwrap_err().to_string());
                }
                let stored_title = inner_doc.get_str("title");
                if stored_title.is_err() {
                    return Err(stored_title.unwrap_err().to_string());
                }
                let stored_description = inner_doc.get_str("description");
                if stored_description.is_err() {
                    return Err(stored_description.unwrap_err().to_string());
                }
    
                let entry = FinanceAccountBookingEntry {
                    id: some_booking_entry_id_parse_result.unwrap(),
                    finance_account_id: some_finance_account_id_parse_result.unwrap(),
                    finance_journal_diary_id: some_finance_journal_diary_id_parse_result.unwrap(),
                    booking_type: stored_booking_type_result.unwrap(),
                    booking_time: stored_booking_time.unwrap().to_chrono(),
                    amount: stored_amount.unwrap() as u64,
                    title: stored_title.unwrap().into(),
                    description: stored_description.unwrap().into(),
                };
    
                booking_entries_list.push(entry);
            }

        Ok(booking_entries_list)
    }

    async fn finance_insert_booking_entry(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        action_to_insert: FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, String> {
        panic!("method finance_insert_booking_entry for DbHandlerMongoDB not implemented");
    }

    async fn finance_get_last_saldo_account_entries(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        list_account_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String> {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(&conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let booking_entries_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_BOOKING_ENTRIES);

        //get a binary of UUID or it will not work in production
        let user_id_value = mongodb::bson::Binary::from_uuid(user_id.clone());
        let filter = doc! {"user_id":user_id_value};

        debug!(target:"app::FinanceOverView","Filter document: {}",&filter);
        let projection = doc! {"booking_entry_id":<i32>::from(1),
        "finance_account_id":<i32>::from(1),
        "finance_journal_diary_id":<i32>::from(1),
        "booking_type":<i32>::from(1),
        "booking_time":<i32>::from(1),
        "amount":<i32>::from(1),
        "title":<i32>::from(1),
        "description":<i32>::from(1),};
        let options = FindOptions::builder().projection(projection).build();

        let query_execute_result = booking_entries_collection.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut booking_entries_list = Vec::new();

        while let Some(data_doc) = cursor.next().await {
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc = data_doc.unwrap();

            let some_booking_entry_id_parse_result =
                ConvertTools::get_uuid_from_document(&inner_doc, "booking_entry_id");
            if some_booking_entry_id_parse_result.is_err() {
                return Err(some_booking_entry_id_parse_result.unwrap_err().to_string());
            }
            let some_finance_account_id_parse_result =
                ConvertTools::get_uuid_from_document(&inner_doc, "finance_account_id");
            if some_finance_account_id_parse_result.is_err() {
                return Err(some_finance_account_id_parse_result
                    .unwrap_err()
                    .to_string());
            }
            let some_finance_journal_diary_id_parse_result =
                ConvertTools::get_uuid_from_document(&inner_doc, "finance_journal_diary_id");
            if some_finance_journal_diary_id_parse_result.is_err() {
                return Err(some_finance_journal_diary_id_parse_result
                    .unwrap_err()
                    .to_string());
            }
            let stored_booking_type_int = inner_doc.get_i32("booking_type");
            if stored_booking_type_int.is_err() {
                return Err(stored_booking_type_int.unwrap_err().to_string());
            }
            let stored_booking_type_result =
                BookingEntryType::get_from_int(stored_booking_type_int.unwrap());
            if stored_booking_type_result.is_err() {
                return Err(stored_booking_type_result.unwrap_err());
            }

            let stored_booking_time = inner_doc.get_datetime("booking_time");
            if stored_booking_time.is_err() {
                return Err(stored_booking_time.unwrap_err().to_string());
            }
            let stored_amount = inner_doc.get_i64("amount");
            if stored_amount.is_err() {
                return Err(stored_amount.unwrap_err().to_string());
            }
            let stored_title = inner_doc.get_str("title");
            if stored_title.is_err() {
                return Err(stored_title.unwrap_err().to_string());
            }
            let stored_description = inner_doc.get_str("description");
            if stored_description.is_err() {
                return Err(stored_description.unwrap_err().to_string());
            }

            let entry = FinanceAccountBookingEntry {
                id: some_booking_entry_id_parse_result.unwrap(),
                finance_account_id: some_finance_account_id_parse_result.unwrap(),
                finance_journal_diary_id: some_finance_journal_diary_id_parse_result.unwrap(),
                booking_type: stored_booking_type_result.unwrap(),
                booking_time: stored_booking_time.unwrap().to_chrono(),
                amount: stored_amount.unwrap() as u64,
                title: stored_title.unwrap().into(),
                description: stored_description.unwrap().into(),
            };

            booking_entries_list.push(entry);
        }

        let mut return_object = HashMap::new();

        let account_ids_to_check = match list_account_ids {
            Some(id_list) => id_list,
            None => booking_entries_list
                .iter()
                .map(|elem| elem.finance_account_id)
                .collect::<Vec<Uuid>>(),
        };

        for account_id_to_check in account_ids_to_check {
            let saldo_entries_per_account: Vec<&FinanceAccountBookingEntry> = booking_entries_list
                .iter()
                .filter(|elem| {
                    elem.finance_account_id.eq(&account_id_to_check)
                        && (elem.booking_type.eq(&BookingEntryType::SaldoCredit)
                            || elem.booking_type.eq(&BookingEntryType::SaldoDebit))
                })
                .collect();

            let oldest_saldo_entry_option = saldo_entries_per_account
                .iter()
                .max_by_key(|elem| elem.booking_time);
            if oldest_saldo_entry_option.is_some() {
                return_object.insert(
                    account_id_to_check,
                    oldest_saldo_entry_option.unwrap().to_owned().clone(),
                );
            }
        }

        debug!(target:"app::FinanceOverView","returned {} saldo entries",return_object.len());

        Ok(return_object)
    }
}
