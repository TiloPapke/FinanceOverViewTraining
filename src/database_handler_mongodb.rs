use argon2::{Argon2, PasswordHasher};
use async_session::chrono::Duration;
use futures::{executor, StreamExt};
use log::{debug, info, trace, warn};
use mongodb::{
    bson::{doc, uuid, Document, Uuid},
    options::{ClientOptions, Credential, FindOptions},
    results::{InsertOneResult, UpdateResult},
    Client, Collection, Cursor,
};

use secrecy::{ExposeSecret, Secret};

use crate::{
    convert_tools::ConvertTools, datatypes::GenerallUserData, mail_handle::validate_email_format,
    password_handle::verify_password_hash,
};
use crate::{
    datatypes::PasswordResetTokenRequestResult,
    password_handle::{StoredCredentials, UserCredentialsHashed},
};

pub struct DbConnectionSetting {
    pub url: String,
    pub user: String,
    pub password: String,
    pub instance: String,
}

#[derive(Debug)]
pub enum EmailVerificationStatus {
    NotGiven,
    NotVerified,
    Verified,
}

pub struct DbHandlerMongoDB {
    internal_mongodb_client: Option<Client>,
}

impl DbHandlerMongoDB {
    pub const COLLECTION_NAME_GENERAL_INFORMATION: &'static str = "GeneralInformation";
    pub const COLLECTION_NAME_WEBSITE_TRAFFIC: &'static str = "WebSiteTraffic";
    pub const COLLECTION_NAME_SESSION_INFO: &'static str = "SessionInfo";
    pub const COLLECTION_NAME_USER_LIST: &'static str = "UserList";
    pub const COLLECTION_NAME_ACCOUNTING_TYPES: &'static str = "FinanceAccountTypes";
    pub const COLLECTION_NAME_ACCOUNTS: &'static str = "FinanceAccounts";
    pub const COLLECTION_NAME_BOOKING_ENTRIES: &'static str = "BookingEntries";
    pub const COLLECTION_NAME_JOURNAL_DIARY: &'static str = "FinanceJournalDiary";
    pub const COLLECTION_NAME_COUNTERS: &'static str = "CounterCollection";

    pub fn new(connection_settings: &DbConnectionSetting) -> DbHandlerMongoDB {
        let db_client =
            DbHandlerMongoDB::create_client_connection_sync(connection_settings).unwrap();
        return DbHandlerMongoDB {
            internal_mongodb_client: Some(db_client),
        };
    }

    pub(crate) fn get_internal_db_client(&self) -> Result<Client, String> {
        if self.internal_mongodb_client.is_some() {
            return Ok(self.internal_mongodb_client.clone().unwrap());
        }
        return Err("no DB client prepared".into());
    }

    pub fn validate_db_structure(conncetion_settings: &DbConnectionSetting) -> bool {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_sync(conncetion_settings);
        if client_create_result.is_err() {
            warn!(target:"app::FinanceOverView","{}",client_create_result.unwrap_err());
            return false;
        }
        let client = client_create_result.unwrap();

        // List the names of the databases in that deployment.
        let query_result = executor::block_on(client.list_database_names(None, None));
        if query_result.is_err() {
            warn!(target: "app::FinanceOverView","error listing databases: {}",query_result.unwrap_err());
            return false;
        }

        let instance_list = query_result.unwrap();
        if cfg!(debug_assertions) {
            // debug build
            for db_name in &instance_list {
                trace!(target:"app::FinanceOverView","{}", db_name);
            }
        }

        if !instance_list.contains(&conncetion_settings.instance) {
            warn!(target: "app::FinanceOverView","entry {} not found in list of database names, HINT: does it have at least one collection?", conncetion_settings.instance);
            return false;
        }

        let db_instance = client.database(&conncetion_settings.instance);

        let arr_required_collection: [&str; 9] = [
            &DbHandlerMongoDB::COLLECTION_NAME_GENERAL_INFORMATION,
            &DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC,
            &DbHandlerMongoDB::COLLECTION_NAME_SESSION_INFO,
            &DbHandlerMongoDB::COLLECTION_NAME_USER_LIST,
            &DbHandlerMongoDB::COLLECTION_NAME_ACCOUNTING_TYPES,
            &DbHandlerMongoDB::COLLECTION_NAME_ACCOUNTS,
            &DbHandlerMongoDB::COLLECTION_NAME_BOOKING_ENTRIES,
            &DbHandlerMongoDB::COLLECTION_NAME_COUNTERS,
            &DbHandlerMongoDB::COLLECTION_NAME_JOURNAL_DIARY,
        ];

        let query_result_collections = executor::block_on(db_instance.list_collection_names(None));
        if query_result_collections.is_err() {
            warn!(target: "app::FinanceOverView","error listing collections: {}",query_result_collections.unwrap_err());
            return false;
        }
        let collection_list = query_result_collections.unwrap();

        for required_collection in arr_required_collection {
            if collection_list.contains(&required_collection.to_string()) {
                trace!(target: "app::FinanceOverView","found collection {}",required_collection);
            } else {
                info!(target: "app::FinanceOverView","collection {} not found, trying to create it",required_collection);
                let create_result =
                    executor::block_on(db_instance.create_collection(required_collection, None));
                if create_result.is_err() {
                    warn!(target: "app::FinanceOverView","could not create collection {} in database {}, error: {}",required_collection, conncetion_settings.instance, create_result.unwrap_err());
                    return false;
                }
            }
        }

        return true;
    }

    /* currently not used
    pub fn query_table(conncetion_settings: &DbConnectionSetting, table_to_query: &String)->Result<Collection<CollectionType>,String>
    {
        let client_create_result = DbHandlerCouchbase::create_client_connection(conncetion_settings);
        if client_create_result.is_err() {return Result::Err(client_create_result.unwrap_err().to_string()); }
        let client = client_create_result.unwrap();
        let some_collections:Collection<CollectionType> = client.database(&conncetion_settings.instance).collection(table_to_query);

        return Result::Ok(some_collections);
    }
    */

    pub fn query_table_with_filter(
        conncetion_settings: &DbConnectionSetting,
        table_to_query: &String,
        filter_info: Document,
    ) -> Result<Cursor<Document>, String> {
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_sync(conncetion_settings);
        if client_create_result.is_err() {
            return Result::Err(client_create_result.unwrap_err().to_string());
        }
        let client = client_create_result.unwrap();
        let some_cursor_result = executor::block_on(
            client
                .database(&conncetion_settings.instance)
                .collection(table_to_query)
                .find(filter_info, None),
        );
        if some_cursor_result.is_err() {
            return Result::Err(some_cursor_result.unwrap_err().to_string());
        }

        return Result::Ok(some_cursor_result.unwrap());
    }

    pub fn insert_document_in_table(
        conncetion_settings: &DbConnectionSetting,
        table_to_insert: &String,
        new_document: &Document,
    ) -> Result<InsertOneResult, String> {
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_sync(conncetion_settings);
        if client_create_result.is_err() {
            return Result::Err(client_create_result.unwrap_err().to_string());
        }
        let client = client_create_result.unwrap();
        let some_collections: Collection<Document> = client
            .database(&conncetion_settings.instance)
            .collection(table_to_insert);

        let insert_result_execute_result =
            executor::block_on(some_collections.insert_one(new_document, None));
        if insert_result_execute_result.is_err() {
            return Result::Err(insert_result_execute_result.unwrap_err().to_string());
        }

        return Result::Ok(insert_result_execute_result.unwrap());
    }

    pub fn update_document_in_table(
        conncetion_settings: &DbConnectionSetting,
        table_to_insert: &String,
        query_info: Document,
        update_info: Document,
    ) -> Result<UpdateResult, String> {
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_sync(conncetion_settings);
        if client_create_result.is_err() {
            return Result::Err(client_create_result.unwrap_err().to_string());
        }
        let client = client_create_result.unwrap();
        let some_collections: Collection<Document> = client
            .database(&conncetion_settings.instance)
            .collection(table_to_insert);

        let update_result_execute_result =
            executor::block_on(some_collections.update_one(query_info, update_info, None));
        if update_result_execute_result.is_err() {
            return Result::Err(update_result_execute_result.unwrap_err().to_string());
        }

        return Result::Ok(update_result_execute_result.unwrap());
    }

    //private functions
    pub fn create_client_connection_sync(
        conncetion_settings: &DbConnectionSetting,
    ) -> Result<Client, String> {
        // Parse a connection string into an options struct.
        let v = executor::block_on(ClientOptions::parse(conncetion_settings.url.clone()));
        if v.is_err() {
            return Result::Err(v.unwrap_err().to_string());
        }

        let mut client_options = v.unwrap();
        //set credentials
        let co_source = conncetion_settings.instance.to_string();
        client_options.credential = Some(
            Credential::builder()
                .username(conncetion_settings.user.clone())
                .password(conncetion_settings.password.clone())
                .source(Some(co_source))
                .build(),
        );

        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();
        return Result::Ok(client);
    }

    pub async fn create_client_connection_async(
        conncetion_settings: &DbConnectionSetting,
    ) -> Result<Client, String> {
        // Parse a connection string into an options struct.
        let v = ClientOptions::parse(conncetion_settings.url.clone()).await;
        if v.is_err() {
            return Result::Err(v.unwrap_err().to_string());
        }

        let mut client_options = v.unwrap();
        //set credentials
        let co_source = conncetion_settings.instance.to_string();
        client_options.credential = Some(
            Credential::builder()
                .username(conncetion_settings.user.clone())
                .password(conncetion_settings.password.clone())
                .source(Some(co_source))
                .build(),
        );

        // Get a handle to the deployment.
        let client: Client = Client::with_options(client_options).unwrap();
        return Result::Ok(client);
    }

    pub async fn check_user_exsists_by_name(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
    ) -> Result<bool, String> {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let filter = doc! {"user_name":&user_name};
        let projection = doc! {"user_name":<i32>::from(1)};
        let options = FindOptions::builder().projection(projection).build();

        let data_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
        let query_execute_result = data_collcetion.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut doc_counter = 0;

        while let Some(data_doc) = cursor.next().await {
            doc_counter = doc_counter + 1;
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc = data_doc.unwrap();
            let stored_name = inner_doc.get_str("user_name");
            if stored_name.is_err() {
                return Err(stored_name.unwrap_err().to_string());
            }
            if stored_name.unwrap().eq(user_name) {
                return Ok(true);
            }
        }

        if doc_counter > 0 {
            return Err(format!("found {} entries", doc_counter));
        }

        return Result::Ok(false);
    }

    pub async fn insert_user(
        conncetion_settings: &DbConnectionSetting,
        some_credentials: &UserCredentialsHashed,
    ) -> Result<uuid::Uuid, String> {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        let user_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);

        let new_user_uuid = Uuid::new();
        let insert_doc = doc! {"user_id": new_user_uuid,
        "user_name":&some_credentials.username,
        "password_hash": &some_credentials.password_hash.expose_secret()};

        let insert_result = user_collcetion.insert_one(insert_doc, None).await;
        if insert_result.is_err() {
            let insert_err = &insert_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",insert_err);
            return Err(insert_err.to_string());
        }

        debug!(target:"app::FinanceOverView","new id of user object (not user_id): {}",insert_result.unwrap().inserted_id);

        return Ok(new_user_uuid);
    }

    pub async fn update_user_password(
        conncetion_settings: &DbConnectionSetting,
        some_credentials: &UserCredentialsHashed,
    ) -> Result<bool, String> {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        let user_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);

        let filter_doc = doc! {
        "user_name":&some_credentials.username};
        let inner_update_doc = doc! {
        "password_hash": &some_credentials.password_hash.expose_secret()};
        //otherwise we get "update document must have first key starting with '$"
        let update_doc = doc! {"$set": inner_update_doc};

        let update_result = user_collcetion
            .update_one(filter_doc, update_doc, None)
            .await;
        if update_result.is_err() {
            let update_err = &update_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",update_err);
            return Err(update_err.to_string());
        }
        let unwrapped_result = update_result.unwrap();

        debug!(target:"app::FinanceOverView","count of updated objects: {}",unwrapped_result.modified_count);

        return Ok(true);
    }

    pub async fn get_stored_credentials_by_name(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
    ) -> Result<StoredCredentials, String> {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        let user_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);

        let filter = doc! {"user_name":&user_name};

        let projection = doc! {"user_name":<i32>::from(1),
        "user_id":<i32>::from(1),
        "password_hash":<i32>::from(1)};

        let options = FindOptions::builder().projection(projection).build();

        let query_execute_result = user_collcetion.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut doc_counter = 0;
        let mut result_doc = doc! {};

        while let Some(data_doc) = cursor.next().await {
            doc_counter = doc_counter + 1;
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc = data_doc.unwrap();
            let stored_name = inner_doc.get_str("user_name");
            if stored_name.is_err() {
                return Err(stored_name.unwrap_err().to_string());
            }
            if stored_name.unwrap().eq(user_name) {
                result_doc = inner_doc;
            }
        }

        if doc_counter != 1 {
            return Err(format!("found {} entries", doc_counter));
        }

        let stored_password_hash_read = result_doc.get_str("password_hash");
        if stored_password_hash_read.is_err() {
            return Err(stored_password_hash_read.unwrap_err().to_string());
        }

        let some_uuid_parse_result = ConvertTools::get_uuid_from_document(&result_doc, "user_id");

        if some_uuid_parse_result.is_err() {
            return Err("Could not parse UUID".to_string());
        }

        let some_password_hash =
            secrecy::Secret::<String>::new(stored_password_hash_read.unwrap().to_string());

        let some_cred = StoredCredentials {
            user_id: some_uuid_parse_result.unwrap(),
            password_hash: some_password_hash,
        };

        return Ok(some_cred);
    }

    pub async fn update_user_email(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
        new_email: &String,
    ) -> Result<String, String> {
        //first validate if correct password is given
        let query_username =
            DbHandlerMongoDB::check_user_exsists_by_name(conncetion_settings, user_name).await;

        if query_username.is_err() {
            return Err(format!(
                "Unable to verify username: {}",
                query_username.unwrap_err()
            ));
        }

        if !query_username.unwrap() {
            return Err(format!("username does not exists: {}", user_name));
        }

        //generate a random hash for email validation
        let salt = argon2::password_hash::SaltString::generate(&mut rand::thread_rng());
        let current_time = async_session::chrono::Utc::now();

        let mail_validation_token_result =
            Argon2::default().hash_password(&current_time.to_string().as_bytes(), &salt);
        if mail_validation_token_result.is_err() {
            return Err(format!(
                "Unable to generate email check token: {}",
                mail_validation_token_result.unwrap_err()
            ));
        }
        let mail_validation_token = mail_validation_token_result.unwrap().to_string();

        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        let user_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);

        let filter_doc = doc! {
        "user_name":user_name};
        let inner_update_doc = doc! {
        "user_email": new_email,
        "mail_validated": false,
        "mail_validation_token": &mail_validation_token};
        //otherwise we get "update document must have first key starting with '$"
        let update_doc = doc! {"$set": inner_update_doc};

        let update_result = user_collcetion
            .update_one(filter_doc, update_doc, None)
            .await;
        if update_result.is_err() {
            let update_err = &update_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",update_err);
            return Err(format!("Error updating email address: {}", update_err));
        }
        let unwrapped_result = update_result.unwrap();

        debug!(target:"app::FinanceOverView","count of updated objects: {}",unwrapped_result.modified_count);

        return Ok(mail_validation_token);
    }

    pub async fn check_email_verfification_by_name(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
    ) -> Result<EmailVerificationStatus, String> {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let filter = doc! {"user_name":&user_name};
        let projection = doc! {"user_name":<i32>::from(1),
        "mail_validated":<i32>::from(1),
        "user_email":<i32>::from(1)};
        let options = FindOptions::builder().projection(projection).build();

        let data_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
        let query_execute_result = data_collcetion.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut doc_counter = 0;

        while let Some(data_doc) = cursor.next().await {
            doc_counter = doc_counter + 1;
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc: Document = data_doc.unwrap();
            let stored_name = inner_doc.get_str("user_name");
            if stored_name.is_err() {
                return Err(stored_name.unwrap_err().to_string());
            }

            if stored_name.unwrap().eq(user_name) {
                if inner_doc.contains_key("user_email") {
                    let stored_email = inner_doc.get_str("user_email");
                    if stored_email.is_err() {
                        return Err(stored_email.unwrap_err().to_string());
                    }
                    if stored_email.unwrap().is_empty() {
                        return Ok(EmailVerificationStatus::NotGiven);
                    }
                    let stored_verfication_flag = inner_doc.get_bool("mail_validated");
                    if stored_verfication_flag.is_err() {
                        return Err(stored_verfication_flag.unwrap_err().to_string());
                    }
                    if stored_verfication_flag.unwrap() {
                        return Ok(EmailVerificationStatus::Verified);
                    } else {
                        return Ok(EmailVerificationStatus::NotVerified);
                    };
                } else {
                    return Ok(EmailVerificationStatus::NotGiven);
                }
            }
        }

        if doc_counter > 0 {
            return Err(format!("found {} entries", doc_counter));
        }

        return Result::Ok(EmailVerificationStatus::NotVerified);
    }

    pub async fn verify_email_by_name(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
        email_validation_string: &Secret<String>,
    ) -> Result<EmailVerificationStatus, String> {
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let filter = doc! {
            "user_name":user_name
        };
        let projection = doc! {"user_name":<i32>::from(1),
            "mail_validated":<i32>::from(1),
            "mail_validation_token":<i32>::from(1),
            "user_email":<i32>::from(1),
            "user_id":<i32>::from(1),
        };
        let options = FindOptions::builder().projection(projection).build();

        let data_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
        let query_execute_result = data_collcetion.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut doc_counter = 0;

        while let Some(data_doc) = cursor.next().await {
            doc_counter = doc_counter + 1;
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc: Document = data_doc.unwrap();
            let stored_name = inner_doc.get_str("user_name");
            if stored_name.is_err() {
                return Err(stored_name.unwrap_err().to_string());
            }

            if stored_name.unwrap().eq(user_name) {
                if inner_doc.contains_key("user_email") {
                    let stored_email = inner_doc.get_str("user_email");
                    if stored_email.is_err() {
                        return Err(stored_email.unwrap_err().to_string());
                    }
                    let stored_verfication_flag = inner_doc.get_bool("mail_validated");
                    if stored_verfication_flag.is_err() {
                        return Err(stored_verfication_flag.unwrap_err().to_string());
                    }
                    let stored_verfication_token = inner_doc.get_str("mail_validation_token");
                    if stored_verfication_token.is_err() {
                        return Err(stored_verfication_token.unwrap_err().to_string());
                    }
                    let stored_user_id =
                        ConvertTools::get_uuid_from_document(&inner_doc, "user_id");
                    if stored_user_id.is_err() {
                        return Err(stored_user_id.unwrap_err().to_string());
                    }

                    if stored_verfication_flag.unwrap() {
                        return Ok(EmailVerificationStatus::Verified);
                    } else {
                        if stored_verfication_token
                            .unwrap()
                            .eq(email_validation_string.expose_secret())
                        {
                            //addtional check for later: include E-Mail in hashed token and compared it to ensure that the token "belongs" to the E-Mail address
                            //currently only the time plus a salt is used to generate token
                            let filter_doc = doc! {
                            "user_id":stored_user_id.unwrap()};
                            let inner_update_doc = doc! {
                            "mail_validated": true,
                            };
                            //otherwise we get "update document must have first key starting with '$"
                            let update_doc = doc! {"$set": inner_update_doc};

                            let user_collcetion: Collection<Document> =
                                db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
                            let update_result = user_collcetion
                                .update_one(filter_doc, update_doc, None)
                                .await;
                            if update_result.is_err() {
                                let update_err = &update_result.unwrap_err();
                                warn!(target:"app::FinanceOverView","{}",update_err);
                                return Err(format!(
                                    "Error updating email validated token: {}",
                                    update_err
                                ));
                            }
                            let unwrapped_result = update_result.unwrap();

                            debug!(target:"app::FinanceOverView","count of updated objects during user email verfication: {}",unwrapped_result.modified_count);

                            if unwrapped_result.modified_count == 1 {
                                return Ok(EmailVerificationStatus::Verified);
                            } else {
                                return Err("number of updated monrecord not 1".to_string());
                            }
                        } else {
                            return Err(
                                "provided E-Mail Validation token not matching stored token"
                                    .to_string(),
                            );
                        }
                    }
                } else {
                    return Err("no E-Mail given".to_string());
                }
            }
        }

        if doc_counter > 0 {
            return Err(format!("found {} entries", doc_counter));
        }

        return Result::Ok(EmailVerificationStatus::NotVerified);
    }

    pub async fn get_user_general_data_by_user_name(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
    ) -> Result<GenerallUserData, String> {
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let filter = doc! {
            "user_name":user_name
        };
        let projection = doc! {"user_name":<i32>::from(1),
            "first_name":<i32>::from(1),
            "last_name":<i32>::from(1),
        };
        let options = FindOptions::builder().projection(projection).build();

        let data_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
        let query_execute_result = data_collcetion.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        let mut doc_counter = 0;

        while let Some(data_doc) = cursor.next().await {
            doc_counter = doc_counter + 1;
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc: Document = data_doc.unwrap();
            let stored_name = inner_doc.get_str("user_name");
            if stored_name.is_err() {
                return Err(stored_name.unwrap_err().to_string());
            }

            if stored_name.unwrap().eq(user_name) {
                let mut return_data: GenerallUserData = GenerallUserData {
                    first_name: "".to_string(),
                    last_name: "".to_string(),
                };
                let stored_first_name = inner_doc.get_str("first_name");
                if stored_first_name.is_ok() {
                    return_data.first_name = stored_first_name.unwrap().to_string();
                }
                let stored_last_name = inner_doc.get_str("last_name");
                if stored_last_name.is_ok() {
                    return_data.last_name = stored_last_name.unwrap().to_string();
                }

                return Ok(return_data);
            }
        }

        return Err(format!("found {} entries", doc_counter));
    }

    pub async fn update_general_user_data_by_name(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
        general_user_data: &GenerallUserData,
    ) -> Result<String, String> {
        //first validate if correct password is given
        let query_username =
            DbHandlerMongoDB::check_user_exsists_by_name(conncetion_settings, user_name).await;

        if query_username.is_err() {
            return Err(format!(
                "Unable to verify username: {}",
                query_username.unwrap_err()
            ));
        }

        if !query_username.unwrap() {
            return Err(format!("username does not exists: {}", user_name));
        }

        // Get a handle to the database.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        let user_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);

        let filter_doc = doc! {
        "user_name":user_name};
        let inner_update_doc = doc! {
        "first_name": general_user_data.first_name.clone(),
        "last_name": general_user_data.last_name.clone(),
        };
        //otherwise we get "update document must have first key starting with '$"
        let update_doc = doc! {"$set": inner_update_doc};

        let update_result = user_collcetion
            .update_one(filter_doc, update_doc, None)
            .await;
        if update_result.is_err() {
            let update_err = &update_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",update_err);
            return Err(format!("Error updating email address: {}", update_err));
        }
        let unwrapped_result = update_result.unwrap();

        debug!(target:"app::FinanceOverView","count of updated objects: {}",unwrapped_result.modified_count);

        return Ok("updated".to_string());
    }

    pub async fn update_user_reset_secret(
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
        reset_secret_hash: &Secret<String>,
    ) -> Result<bool, String> {
        // Get a handle to the deployment.
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);
        let user_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);

        let filter_doc = doc! {
        "user_id":user_id};
        let inner_update_doc = doc! {
        "reset_secret_hash": reset_secret_hash.expose_secret()};
        //otherwise we get "update document must have first key starting with '$"
        let update_doc = doc! {"$set": inner_update_doc};

        let update_result = user_collcetion
            .update_one(filter_doc, update_doc, None)
            .await;
        if update_result.is_err() {
            let update_err = &update_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",update_err);
            return Err(update_err.to_string());
        }
        let unwrapped_result = update_result.unwrap();

        debug!(target:"app::FinanceOverView","count of updated objects: {}",unwrapped_result.modified_count);

        return Ok(true);
    }

    pub async fn generate_passwort_reset_token(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
        reset_secret: &Secret<String>,
        passwort_reset_time_limit_minutes: &u16,
    ) -> Result<PasswordResetTokenRequestResult, String> {
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let filter = doc! {
            "user_name":user_name
        };
        let projection = doc! {
            "user_name":<i32>::from(1),
            "user_email":<i32>::from(1),
            "reset_secret_hash":<i32>::from(1),
        };
        let options = FindOptions::builder().projection(projection).build();

        let data_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
        let query_execute_result = data_collcetion.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        while let Some(data_doc) = cursor.next().await {
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc: Document = data_doc.unwrap();
            let stored_name = inner_doc.get_str("user_name");
            if stored_name.is_err() {
                return Err(stored_name.unwrap_err().to_string());
            }
            let stored_email = inner_doc.get_str("user_email");
            if stored_email.is_err() {
                return Err(stored_email.unwrap_err().to_string());
            }
            let stored_user_email = stored_email.unwrap();

            if stored_name.unwrap().eq(user_name) {
                let stored_reset_secret_raw = inner_doc.get_str("reset_secret_hash");
                if stored_reset_secret_raw.is_ok() {
                    let transformed_stored_serect = secrecy::Secret::<String>::new(
                        stored_reset_secret_raw.unwrap().to_string(),
                    );
                    let verify_serect_result =
                        verify_password_hash(&transformed_stored_serect, &reset_secret);
                    if verify_serect_result.is_ok() {
                        let email_validation_result =
                            validate_email_format(&stored_user_email.to_string());
                        if email_validation_result.is_err() {
                            return Err(email_validation_result.unwrap_err().to_string());
                        }
                        if !email_validation_result.unwrap() {
                            return Err("no valid e-mail address for operation".to_string());
                        }
                        let reset_token_value = Uuid::new().to_string();
                        let reset_token_timestamp = async_session::chrono::Utc::now()
                            + Duration::minutes(passwort_reset_time_limit_minutes.clone() as i64);

                        let inner_update_doc = doc! {
                        "password_reset_token_value": reset_token_value.clone(),
                        "password_reset_token_timestamp": reset_token_timestamp.timestamp()
                        };

                        let filter2 = doc! {
                            "user_name":user_name
                        };
                        //otherwise we get "update document must have first key starting with '$"
                        let update_doc = doc! {"$set": inner_update_doc};

                        let update_result =
                            data_collcetion.update_one(filter2, update_doc, None).await;
                        if update_result.is_err() {
                            let update_err = &update_result.unwrap_err();
                            warn!(target:"app::FinanceOverView","{}",update_err);
                            return Err("Error validating reset token".to_string());
                        }
                        let unwrapped_result = update_result.unwrap();

                        debug!(target:"app::FinanceOverView","count of updated objects: {}",unwrapped_result.modified_count);

                        let return_value = PasswordResetTokenRequestResult {
                            reset_token: reset_token_value,
                            expires_at: reset_token_timestamp,
                            user_email: stored_user_email.to_string(),
                        };

                        return Ok(return_value);
                    }
                    return Err("error in token generation".to_string());
                }
                return Err("error generating token".to_string());
            }

            return Err("can not generate token".to_string());
        }
        return Err("unable to generate token".to_string());
    }

    pub async fn change_password_with_token(
        conncetion_settings: &DbConnectionSetting,
        user_name: &String,
        reset_token: &String,
        new_password: &Secret<String>,
    ) -> Result<bool, String> {
        let client_create_result =
            DbHandlerMongoDB::create_client_connection_async(conncetion_settings).await;
        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let filter = doc! {
            "user_name":user_name
        };
        let projection = doc! {"user_name":<i32>::from(1),
            "password_reset_token_value":<i32>::from(1),
            "password_reset_token_timestamp":<i32>::from(1),
        };
        let options = FindOptions::builder().projection(projection).build();

        let data_collcetion: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
        let query_execute_result = data_collcetion.find(filter, options).await;

        if query_execute_result.is_err() {
            return Result::Err(query_execute_result.unwrap_err().to_string());
        }

        let mut cursor = query_execute_result.unwrap();

        while let Some(data_doc) = cursor.next().await {
            if data_doc.is_err() {
                return Err(data_doc.unwrap_err().to_string());
            }

            let inner_doc: Document = data_doc.unwrap();
            let stored_name = inner_doc.get_str("user_name");
            if stored_name.is_err() {
                return Err(stored_name.unwrap_err().to_string());
            }

            if stored_name.unwrap().eq(user_name) {
                let stored_reset_token_value = inner_doc.get_str("password_reset_token_value");
                let stored_reset_token_timestamp =
                    inner_doc.get_i64("password_reset_token_timestamp");

                if stored_reset_token_timestamp.is_err() || stored_reset_token_value.is_err() {
                    return Err("unable to retrive reset settings".to_string());
                }
                if stored_reset_token_value.unwrap().ne(reset_token) {
                    return Err("token missmatch".to_string());
                }
                let timestamp_now = async_session::chrono::Utc::now();
                if timestamp_now.timestamp() > stored_reset_token_timestamp.unwrap() {
                    return Err("token expired".to_string());
                }

                let inner_update_doc = doc! {
                "password_reset_token_value": "",
                "password_reset_token_timestamp": "",
                "password_hash":new_password.expose_secret()
                };

                let filter2 = doc! {
                    "user_name":user_name
                };
                //otherwise we get "update document must have first key starting with '$"
                let update_doc = doc! {"$set": inner_update_doc};

                let update_result = data_collcetion.update_one(filter2, update_doc, None).await;
                if update_result.is_err() {
                    let update_err = &update_result.unwrap_err();
                    warn!(target:"app::FinanceOverView","{}",update_err);
                    return Err("Error resetting value".to_string());
                }
                let unwrapped_result = update_result.unwrap();

                debug!(target:"app::FinanceOverView","count of updated objects: {}",unwrapped_result.modified_count);

                return Ok(true);
            }

            return Err("can not  value".to_string());
        }
        return Err("unable to reset value".to_string());
    }

    pub async fn repair_counter_record_for_user(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
    ) -> Result<(), String> {
        // Get a handle to the deployment.
        let client_create_result = self.get_internal_db_client();

        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let counter_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_COUNTERS);

        //get a binary of UUID or it will not work in production
        let user_id_value = mongodb::bson::Binary::from_uuid(user_id.clone());
        let filter = doc! {"user_id":&user_id_value};

        let find_result = counter_collection.find_one(filter.clone(), None).await;
        if find_result.is_err() {
            let error_var = find_result.unwrap_err();
            return Err(format!("Error getting record: {}", error_var));
        }
        let find_record_option = find_result.unwrap();
        if find_record_option.is_none() {
            let journal_max_number_result = self
                .get_max_running_journal_number(&conncetion_settings, user_id)
                .await;
            if journal_max_number_result.is_err() {
                return Err(journal_max_number_result.unwrap_err());
            }
            let base_doc = doc! {"user_id": &user_id_value,
            "counter_entry_id":mongodb::bson::Binary::from_uuid(Uuid::new()),
            "booking_journal_max_number": journal_max_number_result.unwrap()};

            let insert_result = counter_collection.insert_one(base_doc, None).await;
            if insert_result.is_err() {
                return Err(format!(
                    "Error inserting base counter record: {}",
                    insert_result.unwrap_err()
                ));
            }
            let insert_one_info = insert_result.unwrap();
            return Ok(());
        }

        let counter_record = find_record_option.unwrap();
        if !counter_record.contains_key("booking_journal_max_number") {
            let journal_max_number_result = self
                .get_max_running_journal_number(&conncetion_settings, user_id)
                .await;
            if journal_max_number_result.is_err() {
                return Err(journal_max_number_result.unwrap_err());
            }

            let mut updated_info = counter_record.clone();
            updated_info.insert(
                "booking_journal_max_number",
                journal_max_number_result.unwrap(),
            );

            let update_result = counter_collection
                .update_one(filter.clone(), updated_info, None)
                .await;
            if update_result.is_err() {
                return Err(format!(
                    "Error updating base counter record: {}",
                    update_result.unwrap_err()
                ));
            }
            let update_info = update_result.unwrap();
            if update_info.modified_count.ne(&1) {
                return Err(format!(
                    "Error updating base counter record: {} records modfied",
                    update_info.modified_count
                ));
            }
        }

        return Ok(());
    }

    async fn get_max_running_journal_number(
        &self,
        conncetion_settings: &DbConnectionSetting,
        user_id: &Uuid,
    ) -> Result<i64, String> {
        // Get a handle to the deployment.
        let client_create_result = self.get_internal_db_client();

        if client_create_result.is_err() {
            let client_err = &client_create_result.unwrap_err();
            warn!(target:"app::FinanceOverView","{}",client_err);
            return Err(client_err.to_string());
        }
        let client = client_create_result.unwrap();

        let db_instance = client.database(&conncetion_settings.instance);

        let journal_collection: Collection<Document> =
            db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_JOURNAL_DIARY);

        //get a binary of UUID or it will not work in production
        let user_id_value = mongodb::bson::Binary::from_uuid(user_id.clone());
        let aggregate_pipeline = [
            doc! {"$match": doc! {"user_id":user_id_value}},
            doc! {"$group": doc! {"_id": "user_id", "max_journal_number": doc! {"$max": "running_number"}}},
        ];
        let max_current_number_result =
            journal_collection.aggregate(aggregate_pipeline, None).await;
        if max_current_number_result.is_err() {
            return Err(format!(
                "Error getting max running from journal: {}",
                max_current_number_result.unwrap_err()
            ));
        }

        let mut max_current_number_cursor = max_current_number_result.unwrap();
        let mut count_doc = 0;
        let mut global_max = 0;

        while let Some(data_doc) = max_current_number_cursor.next().await {
            if data_doc.is_err() {
                return Err(format!(
                    "Error receving max running from journal: {}",
                    data_doc.unwrap_err().to_string()
                ));
            }
            let inner_doc = data_doc.unwrap();

            let current_number_result = inner_doc.get_i64("max_journal_number");
            if current_number_result.is_err() {
                return Err(format!(
                    "Error extracting max running from journal: {}",
                    current_number_result.unwrap_err().to_string()
                ));
            }

            count_doc += 1;
            global_max = current_number_result.unwrap();
        }
        if count_doc <= 1 {
            return Ok(global_max);
        } else {
            return Err(format!(
                "Error extracting max running from journal: more than 1 record found {}",
                count_doc
            ));
        }
    }
}
