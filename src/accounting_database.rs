use std::collections::HashMap;

use async_session::chrono::{DateTime, Utc};
use axum::async_trait;
use futures::StreamExt;
use log::{debug, warn};
use mongodb::{
    bson::{doc, Document, Uuid},
    error::{TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT},
    options::{Acknowledgment, FindOptions, ReadConcern, TransactionOptions, WriteConcern},
    ClientSession, Collection,
};

use crate::{
    accounting_config_logic::FinanceAccountingConfigHandle,
    convert_tools::ConvertTools,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
    datatypes::{
        BookingEntryType, FinanceAccountBookingEntry, FinanceBookingRequest, FinanceBookingResult,
        FinanceJournalEntry,
    },
    mdb_convert_tools::MdbConvertTools,
};

pub struct FinanceAccountBookingEntryListSearchOption {
    pub(crate) finance_account_id: Uuid,
    pub(crate) booking_time_from: Option<DateTime<Utc>>,
    pub(crate) booking_time_till: Option<DateTime<Utc>>,
}

impl FinanceAccountBookingEntryListSearchOption {
    pub fn new(
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            finance_account_id: finance_account_id.clone(),
            booking_time_from,
            booking_time_till,
        }
    }
}

#[async_trait(?Send)]
pub trait DBFinanceAccountingFunctions {
    async fn finance_journal_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceJournalEntry>, String>;

    async fn finance_account_booking_entry_list_single(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String>;

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        search_options: Vec<FinanceAccountBookingEntryListSearchOption>,
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
        // Get a handle to the deployment.
        let client_create_result = self.get_internal_db_client();
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let journal_diary_entries_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_JOURNAL_DIARY);

        //get a binary of UUID or it will not work in production
        let user_filter = doc! {"user_id":MdbConvertTools::get_binary_from_bson_uuid(user_id)};
        let mut sub_filters = Vec::new();
        sub_filters.push(user_filter);
        if booking_time_from.is_some() {
            let sub_doc1 = doc! {"booking_time": doc! {"$gte": booking_time_from.unwrap()}};
            sub_filters.push(sub_doc1);
        }
        if booking_time_till.is_some() {
            let sub_doc2 = doc! {"booking_time": doc! {"$lte": booking_time_till.unwrap()}};
            sub_filters.push(sub_doc2);
        }
        let filter = if sub_filters.len().eq(&1) {
            sub_filters[0].clone()
        } else {
            doc!("$and": sub_filters)
        };

        debug!(target:"app::FinanceOverView","Filter document: {}",&filter);
        let projection = doc! {
        "finance_journal_diary_id":<i32>::from(1),
        "is_simple_entry":<i32>::from(1),
        "is_saldo":<i32>::from(1),
        "debit_finance_account_id":<i32>::from(1),
        "credit_finance_account_id":<i32>::from(1),
        "running_number":<i32>::from(1),
        "booking_time":<i32>::from(1),
        "amount":<i32>::from(1),
        "title":<i32>::from(1),
        "description":<i32>::from(1),};
        let options = FindOptions::builder().projection(projection).build();

        let query_execute_result = journal_diary_entries_collection.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut journal_entries_list = Vec::new();

        while let Some(data_doc) = cursor.next().await {
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc = data_doc.unwrap();

            let some_journal_entry_id_parse_result =
                ConvertTools::get_uuid_from_document(&inner_doc, "finance_journal_diary_id");
            if some_journal_entry_id_parse_result.is_err() {
                return Err(some_journal_entry_id_parse_result.unwrap_err().to_string());
            }
            let some_debit_account_id_parse_result =
                ConvertTools::get_uuid_from_document(&inner_doc, "debit_finance_account_id");
            if some_debit_account_id_parse_result.is_err() {
                return Err(some_debit_account_id_parse_result.unwrap_err().to_string());
            }
            let some_credit_account_id_parse_result =
                ConvertTools::get_uuid_from_document(&inner_doc, "credit_finance_account_id");
            if some_credit_account_id_parse_result.is_err() {
                return Err(some_credit_account_id_parse_result.unwrap_err().to_string());
            }

            let stored_booking_time = inner_doc.get_datetime("booking_time");
            if stored_booking_time.is_err() {
                return Err(stored_booking_time.unwrap_err().to_string());
            }
            let stored_amount = inner_doc.get_i64("amount");
            if stored_amount.is_err() {
                return Err(stored_amount.unwrap_err().to_string());
            }
            let stored_running_number = inner_doc.get_i64("running_number");
            if stored_running_number.is_err() {
                return Err(stored_running_number.unwrap_err().to_string());
            }
            let stored_title = inner_doc.get_str("title");
            if stored_title.is_err() {
                return Err(stored_title.unwrap_err().to_string());
            }
            let stored_description = inner_doc.get_str("description");
            if stored_description.is_err() {
                return Err(stored_description.unwrap_err().to_string());
            }
            let stored_is_simple_entry = inner_doc.get_bool("is_simple_entry");
            if stored_is_simple_entry.is_err() {
                return Err(stored_is_simple_entry.unwrap_err().to_string());
            }
            let stored_is_saldo = inner_doc.get_bool("is_saldo");
            if stored_is_saldo.is_err() {
                return Err(stored_is_simple_entry.unwrap_err().to_string());
            }

            let entry = FinanceJournalEntry {
                id: some_journal_entry_id_parse_result.unwrap(),
                booking_time: stored_booking_time.unwrap().to_chrono(),
                amount: stored_amount.unwrap() as u64,
                title: stored_title.unwrap().into(),
                description: stored_description.unwrap().into(),
                is_simple_entry: stored_is_simple_entry.unwrap(),
                is_saldo: stored_is_saldo.unwrap(),
                debit_finance_account_id: some_debit_account_id_parse_result.unwrap(),
                credit_finance_account_id: some_credit_account_id_parse_result.unwrap(),
                running_number: stored_running_number.unwrap() as u64,
            };

            journal_entries_list.push(entry);
        }

        Ok(journal_entries_list)
    }

    async fn finance_account_booking_entry_list_single(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        finance_account_id: &Uuid,
        booking_time_from: Option<DateTime<Utc>>,
        booking_time_till: Option<DateTime<Utc>>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        // Get a handle to the deployment.
        let client_create_result = self.get_internal_db_client();
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let accounting_handle =
            FinanceAccountingConfigHandle::new(&conncetion_settings, &user_id, self);
        let account_list_result = accounting_handle
            .finance_account_list_async(Some(&vec![finance_account_id.clone()]))
            .await;
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
            filter.insert("booking_time", doc! {"$gte": booking_time_from.unwrap()});
        }
        if booking_time_till.is_some() {
            filter.insert("booking_time", doc! {"$lte": booking_time_from.unwrap()});
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

    async fn finance_account_booking_entry_list(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        search_options: Vec<FinanceAccountBookingEntryListSearchOption>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        // Get a handle to the deployment.
        let client_create_result = self.get_internal_db_client();
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        //extract account id
        let account_ids_to_check = search_options
            .iter()
            .map(|elem| elem.finance_account_id)
            .collect::<Vec<Uuid>>();

        let accounting_handle =
            FinanceAccountingConfigHandle::new(&conncetion_settings, &user_id, self);
        let account_list_exists_result = accounting_handle
            .finance_account_list_async(Some(&account_ids_to_check))
            .await;
        if account_list_exists_result.is_err() {
            return Err(format!(
                "Error retriving account list: {}",
                account_list_exists_result.unwrap_err()
            ));
        }

        let account_list_exists = account_list_exists_result.unwrap();

        for account_id_to_check in &account_ids_to_check {
            let account_position_option = account_list_exists
                .iter()
                .position(|elem| elem.id.eq(&account_id_to_check));
            if account_position_option.is_none() {
                return Err(format!("account {} not avaiable", account_id_to_check));
            }
        }

        let booking_entries_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_BOOKING_ENTRIES);

        let mut sub_filter_docs = Vec::new();
        for search_option in &search_options {
            //get a binary of UUID or it will not work in production
            let mut sub_filter = doc! { "finance_account_id": MdbConvertTools::get_binary_from_bson_uuid(&search_option.finance_account_id)};
            if search_option.booking_time_from.is_some() {
                sub_filter.insert(
                    "booking_time",
                    doc! {"$gte": search_option.booking_time_from.unwrap()},
                );
            }
            if search_option.booking_time_till.is_some() {
                sub_filter.insert(
                    "booking_time",
                    doc! {"$lte": search_option.booking_time_from.unwrap()},
                );
            }
            sub_filter_docs.push(sub_filter);
        }
        let filter = doc! {"user_id":MdbConvertTools::get_binary_from_bson_uuid(user_id),
        "$or":  sub_filter_docs};

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
        // Get a handle to the deployment.
        let client_create_result = self.get_internal_db_client();
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let accounting_handle =
            FinanceAccountingConfigHandle::new(&conncetion_settings, &user_id, self);
        let account_list_result = accounting_handle
            .finance_account_list_async(Some(&vec![
                action_to_insert.credit_finance_account_id,
                action_to_insert.debit_finance_account_id,
            ]))
            .await;
        if account_list_result.is_err() {
            return Err(format!(
                "Error retriving account list: {}",
                account_list_result.unwrap_err()
            ));
        }

        let account_list = account_list_result.unwrap();
        let check_credit_account_check_option = account_list
            .iter()
            .position(|elem| elem.id.eq(&action_to_insert.credit_finance_account_id));
        if check_credit_account_check_option.is_none() {
            return Err("credit account is not available".into());
        }
        let check_debit_account_check_option = account_list
            .iter()
            .position(|elem| elem.id.eq(&action_to_insert.debit_finance_account_id));
        if check_debit_account_check_option.is_none() {
            return Err("debit account is not available".into());
        }

        let session_result = client.start_session(None).await;
        if session_result.is_err() {
            return Err(format!(
                "problem getting session: {}",
                session_result.unwrap_err()
            ));
        }

        let options = TransactionOptions::builder()
            .read_concern(ReadConcern::majority())
            .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
            .build();

        let mut session = session_result.unwrap();
        let transaction_start_result = session.start_transaction(options).await;
        if transaction_start_result.is_err() {
            return Err(format!(
                "problem starting transaction: {}",
                transaction_start_result.unwrap_err()
            ));
        }

        loop {
            let execute_result =
                DbHandlerMongoDB::execute_finance_insert_booking_entry_with_transaction(
                    &mut session,
                    &conncetion_settings.instance,
                    &user_id,
                    action_to_insert.clone(),
                )
                .await;
            if execute_result.is_ok() {
                let result_object = execute_result.unwrap();
                return Ok(result_object);
            } else {
                let error_var = execute_result.unwrap_err();

                if !error_var.contains_label(TRANSIENT_TRANSACTION_ERROR) {
                    let error_message;
                    let custom_info = error_var.get_custom::<String>();
                    if custom_info.is_some() {
                        error_message = custom_info.unwrap().to_string();
                    } else {
                        error_message = error_var.to_string();
                    }
                    return Err(format!("Problem closing transaction: {}", error_message));
                }
            }
        }
    }

    async fn finance_get_last_saldo_account_entries(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        list_account_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<Uuid, FinanceAccountBookingEntry>, String> {
        // Get a handle to the deployment.
        let client_create_result = self.get_internal_db_client();

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

impl DbHandlerMongoDB {
    /// Helper function for DBFinanceAccountingFunctions::finance_insert_booking_entry()
    /// see https://github.com/mongodb/mongo-rust-driver/blob/main/tests/transactions_example.rs
    /// see https://docs.rs/mongodb/2.8.2/mongodb/struct.ClientSession.html
    /// see https://www.mongodb.com/docs/v2.2/reference/operator/update/inc/
    async fn execute_finance_insert_booking_entry_with_transaction(
        session: &mut ClientSession,
        db_instance_name: &String,
        user_id: &Uuid,
        action_to_insert: FinanceBookingRequest,
    ) -> Result<FinanceBookingResult, mongodb::error::Error> {
        let client = session.client();
        let db_instance = client.database(&db_instance_name);

        let booking_entries_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_BOOKING_ENTRIES);

        let journal_diary_entries_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_JOURNAL_DIARY);

        let counter_entries_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_COUNTERS);

        let user_id_value = mongodb::bson::Binary::from_uuid(user_id.clone());

        let increasing_journal_max_result = counter_entries_collection
            .update_one_with_session(
                doc! {"user_id": user_id_value.clone()},
                doc! {"$inc": doc! {"booking_journal_max_number":1}},
                None,
                session,
            )
            .await?;

        if increasing_journal_max_result.modified_count.ne(&1) {
            return Err(mongodb::error::Error::custom(format!(
                "could not update max number record, upated record: {}",
                increasing_journal_max_result.modified_count
            )));
        }

        let filter = doc! {"user_id":user_id_value.clone()};

        let max_number_execute_result = counter_entries_collection
            .find_one_with_session(filter, None, session)
            .await?;
        let counter_document = max_number_execute_result.unwrap();
        let new_running_number_result = counter_document.get_i64("booking_journal_max_number");
        if new_running_number_result.is_err() {
            return Err(mongodb::error::Error::custom(format!(
                "could not get new max number: {}",
                new_running_number_result.unwrap_err()
            )));
        }
        let new_running_number = new_running_number_result.unwrap() as u64;

        let journal_diary_entry_id = Uuid::new();
        let journal_diary_entry_id_value =
            mongodb::bson::Binary::from_uuid(journal_diary_entry_id.clone());
        let new_journal_entry = FinanceJournalEntry {
            id: Uuid::new(),
            is_simple_entry: action_to_insert.is_simple_entry,
            is_saldo: action_to_insert.is_saldo,
            debit_finance_account_id: action_to_insert.debit_finance_account_id,
            credit_finance_account_id: action_to_insert.credit_finance_account_id,
            running_number: new_running_number as u64,
            booking_time: action_to_insert.booking_time,
            amount: action_to_insert.amount,
            title: action_to_insert.title.clone(),
            description: action_to_insert.description.clone(),
        };

        let credit_booking_type = if action_to_insert.is_saldo {
            BookingEntryType::SaldoCredit
        } else {
            BookingEntryType::Credit
        };
        let new_credit_account_entry = FinanceAccountBookingEntry {
            id: Uuid::new(),
            finance_account_id: action_to_insert.credit_finance_account_id,
            finance_journal_diary_id: new_journal_entry.id.clone(),
            booking_type: credit_booking_type.clone(),
            booking_time: action_to_insert.booking_time,
            amount: action_to_insert.amount,
            title: action_to_insert.title.clone(),
            description: action_to_insert.description.clone(),
        };
        let debit_booking_type = if action_to_insert.is_saldo {
            BookingEntryType::SaldoDebit
        } else {
            BookingEntryType::Debit
        };
        let new_debit_account_entry = FinanceAccountBookingEntry {
            id: Uuid::new(),
            finance_account_id: action_to_insert.debit_finance_account_id,
            finance_journal_diary_id: new_journal_entry.id.clone(),
            booking_type: debit_booking_type.clone(),
            booking_time: action_to_insert.booking_time,
            amount: action_to_insert.amount,
            title: action_to_insert.title.clone(),
            description: action_to_insert.description.clone(),
        };
        let debit_finance_account_id_value =
            mongodb::bson::Binary::from_uuid(action_to_insert.debit_finance_account_id);
        let credit_finance_account_id_value =
            mongodb::bson::Binary::from_uuid(action_to_insert.credit_finance_account_id);

        let journal_insert_result = journal_diary_entries_collection
            .insert_one_with_session(
                doc! {
                    "finance_journal_diary_id":journal_diary_entry_id_value.clone(),
                    "user_id": user_id_value.clone(),
                    "is_simple_entry": action_to_insert.is_simple_entry,
                    "is_saldo":action_to_insert.is_saldo,
                    "debit_finance_account_id":debit_finance_account_id_value.clone(),
                    "credit_finance_account_id":credit_finance_account_id_value.clone(),
                    "running_number":new_running_number as i64,
                    "booking_time":action_to_insert.booking_time,
                    "amount":action_to_insert.amount as i64,
                    "title":action_to_insert.title.clone(),
                    "description":action_to_insert.description.clone()
                },
                None,
                session,
            )
            .await;

        if journal_insert_result.is_err() {
            return Err(mongodb::error::Error::custom(format!(
                "could not update journal: {}",
                journal_insert_result.unwrap_err()
            )));
        }

        let booking_insert_1_result = booking_entries_collection
            .insert_one_with_session(
                doc! {
                    "booking_entry_id":mongodb::bson::Binary::from_uuid(new_debit_account_entry.id),
                    "user_id": user_id_value.clone(),
                    "finance_account_id":debit_finance_account_id_value.clone(),
                    "finance_journal_diary_id":journal_diary_entry_id_value.clone(),
                    "booking_type":debit_booking_type.to_int(),
                    "booking_time":action_to_insert.booking_time,
                    "amount":action_to_insert.amount  as i64,
                    "title":action_to_insert.title.clone(),
                    "description":action_to_insert.description.clone()
                },
                None,
                session,
            )
            .await;

        if booking_insert_1_result.is_err() {
            return Err(mongodb::error::Error::custom(format!(
                "could not update first booking: {}",
                booking_insert_1_result.unwrap_err()
            )));
        }

        let booking_insert_2_result=booking_entries_collection.insert_one_with_session(doc! {
    "booking_entry_id":mongodb::bson::Binary::from_uuid(new_credit_account_entry.id),
    "user_id": user_id_value.clone(),
    "finance_account_id":credit_finance_account_id_value.clone(),
    "finance_journal_diary_id":journal_diary_entry_id_value.clone(),
    "booking_type":credit_booking_type.to_int(),
    "booking_time":action_to_insert.booking_time,
    "amount":action_to_insert.amount  as i64,
    "title":action_to_insert.title.clone(),
    "description":action_to_insert.description.clone()
}, None, session).await;

        if booking_insert_2_result.is_err() {
            return Err(mongodb::error::Error::custom(format!(
                "could not update second booking: {}",
                booking_insert_2_result.unwrap_err()
            )));
        }

        loop {
            let result = session.commit_transaction().await;
            if let Err(ref error) = result {
                if error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                    continue;
                }
            }

            let return_object = FinanceBookingResult {
                journal_entry: new_journal_entry,
                debit_account_entry: new_debit_account_entry,
                credit_account_entry: new_credit_account_entry,
            };
            return Ok(return_object);
        }
    }
}
