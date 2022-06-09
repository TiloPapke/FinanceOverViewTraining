use mongodb::{options::{ClientOptions, Credential, FindOptions}, Client, Collection, Cursor, results::{InsertOneResult, UpdateResult}, bson::{Document, doc}};
use futures::{executor, StreamExt};
use log::{warn, trace, info, debug};
use secrecy::ExposeSecret;

use crate::password_handle::{UserCredentials, StoredCredentials};
use crate::convert_tools::ConvertTools;

pub struct DbConnectionSetting{
    pub url: String,
    pub user: String,
    pub password: String,
    pub instance: String
}

pub struct DbHandlerMongoDB{

}

impl DbHandlerMongoDB{
    pub const COLLECTION_NAME_GENERAL_INFORMATION:&'static str ="GeneralInformation";
    pub const COLLECTION_NAME_WEBSITE_TRAFFIC:&'static str ="WebSiteTraffic";
    pub const COLLECTION_NAME_SESSION_INFO:&'static str ="SessionInfo";
    pub const COLLECTION_NAME_USER_LIST:&'static str ="UserList";
    
    pub fn validate_db_structure(conncetion_settings: &DbConnectionSetting) -> bool
    {


    // Get a handle to the deployment.
    let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
    if client_create_result.is_err()
    {
        warn!(target:"app::FinanceOverView","{}",client_create_result.unwrap_err());
        return false;
    }
    let client = client_create_result.unwrap();
    

    // List the names of the databases in that deployment.
    let query_result = executor::block_on(client.list_database_names(None, None));
    if query_result.is_err(){
        warn!(target: "app::FinanceOverView","error listing databases: {}",query_result.unwrap_err());
        return false;
    }

    let instance_list = query_result.unwrap();
    if cfg!(debug_assertions){
        // debug build
        for db_name in &instance_list {
            trace!(target:"app::FinanceOverView","{}", db_name);
        }
    }

    if !instance_list.contains(&conncetion_settings.instance)
    {
        warn!(target: "app::FinanceOverView","entry {} not found in list of database names, HINT: does it have at least one collection?", conncetion_settings.instance);
        return false;
    }

    let db_instance = client.database(&conncetion_settings.instance);

    let arr_required_collection:[&str;4]=[
        &DbHandlerMongoDB::COLLECTION_NAME_GENERAL_INFORMATION,
        &DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC,
        &DbHandlerMongoDB::COLLECTION_NAME_SESSION_INFO,
        &DbHandlerMongoDB::COLLECTION_NAME_USER_LIST
    ];

    let query_result_collections = executor::block_on(db_instance.list_collection_names(None));
    if query_result_collections.is_err(){
        warn!(target: "app::FinanceOverView","error listing collections: {}",query_result_collections.unwrap_err());
        return false;
    }
    let collection_list = query_result_collections.unwrap();

    for required_collection in arr_required_collection {
        if collection_list.contains(&required_collection.to_string())
        {   
            trace!(target: "app::FinanceOverView","found collection {}",required_collection);
        }
        else
        {
            info!(target: "app::FinanceOverView","collection {} not found, trying to create it",required_collection);
            let create_result=executor::block_on( db_instance.create_collection(required_collection,None));
            if create_result.is_err(){
                warn!(target: "app::FinanceOverView","could not create collection {} in database {}, error: {}",required_collection, conncetion_settings.instance, create_result.unwrap_err());
                return false
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

    pub fn query_table_with_filter(conncetion_settings: &DbConnectionSetting, table_to_query: &String, filter_info : Document)->Result<Cursor<Document>,String>
    {
        let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
        if client_create_result.is_err() {return Result::Err(client_create_result.unwrap_err().to_string()); }
        let client = client_create_result.unwrap();
        let some_cursor_result = executor::block_on( client.database(&conncetion_settings.instance).collection(table_to_query).find(filter_info, None));
        if some_cursor_result.is_err()
        {
            return Result::Err(some_cursor_result.unwrap_err().to_string());
        }

        return Result::Ok(some_cursor_result.unwrap());
    }

    pub fn insert_document_in_table (conncetion_settings: &DbConnectionSetting, table_to_insert: &String, new_document : &Document)->Result<InsertOneResult,String>
    {
        let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
        if client_create_result.is_err() {return Result::Err(client_create_result.unwrap_err().to_string()); }
        let client = client_create_result.unwrap();
        let some_collections:Collection<Document> = client.database(&conncetion_settings.instance).collection(table_to_insert);

        let insert_result_execute_result = executor::block_on( some_collections.insert_one(new_document, None));
        if insert_result_execute_result.is_err()
        {
            return Result::Err(insert_result_execute_result.unwrap_err().to_string());
        }

        return Result::Ok(insert_result_execute_result.unwrap());
    }

    pub fn update_document_in_table (conncetion_settings: &DbConnectionSetting, table_to_insert: &String, query_info : Document, update_info: Document)->Result<UpdateResult,String>
    {
        let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
        if client_create_result.is_err() {return Result::Err(client_create_result.unwrap_err().to_string()); }
        let client = client_create_result.unwrap();
        let some_collections:Collection<Document> = client.database(&conncetion_settings.instance).collection(table_to_insert);

        let update_result_execute_result = executor::block_on( some_collections.update_one(query_info, update_info,None));
        if update_result_execute_result.is_err()
        {
            return Result::Err(update_result_execute_result.unwrap_err().to_string());
        }

        return Result::Ok(update_result_execute_result.unwrap());
    }

    //private functions
    pub fn create_client_connection(conncetion_settings: &DbConnectionSetting) -> Result<Client,String>
    {
        // Parse a connection string into an options struct.
        let v = executor::block_on(ClientOptions::parse(conncetion_settings.url.clone()));
        if v.is_err() { return Result::Err(v.unwrap_err().to_string());}
    
        let mut client_options = v.unwrap();
        //set credentials
        let co_source = conncetion_settings.instance.to_string();
        client_options.credential = Some(Credential::builder().username(conncetion_settings.user.clone()).password(conncetion_settings.password.clone()).source(Some(co_source)).build());
        

        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();
        return Result::Ok(client)
    }

    pub async fn check_user_exsists_by_name(conncetion_settings: &DbConnectionSetting, user_name:&String) -> Result<bool,String>
    {
    // Get a handle to the deployment.
    let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
    if client_create_result.is_err()
    {
        let client_err = &client_create_result.unwrap_err();
        warn!(target:"app::FinanceOverView","{}",client_err);
        return Err(client_err.to_string());
    }
    let client = client_create_result.unwrap();
    
    let db_instance = client.database(&conncetion_settings.instance);

    let filter = doc!{"user_name":&user_name};
    let projection = doc!{"user_name":<i32>::from(1)};
    let options=FindOptions::builder().projection(projection).build();

    let data_collcetion:Collection<Document> = db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);
    let query_execute_result = data_collcetion.find(filter, options).await;
    
    if query_execute_result.is_err()
     {
        return Result::Err(query_execute_result.unwrap_err().to_string());
    }

    let mut cursor = query_execute_result.unwrap();

    let mut doc_counter=0;
    
    while let Some(data_doc) = cursor.next().await{
        doc_counter=doc_counter+1;
        if data_doc.is_err() {
            return Err(data_doc.unwrap_err().to_string());
        }
        

        let inner_doc = data_doc.unwrap();
        let stored_name = inner_doc.get_str("user_name");
        if stored_name.is_err(){
            return Err(stored_name.unwrap_err().to_string());
        }
        if stored_name.unwrap().eq(user_name){
            return Ok(true);
        }
    }

    if doc_counter>0{
        return Err(format!("found {} entries",doc_counter));
    }

    return Result::Ok(false);

    }

    pub async fn insert_user(conncetion_settings: &DbConnectionSetting, some_credentials: &UserCredentials) -> Result<uuid::Uuid,String>
    {
    // Get a handle to the deployment.
    let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
    if client_create_result.is_err()
    {
        let client_err = &client_create_result.unwrap_err();
        warn!(target:"app::FinanceOverView","{}",client_err);
        return Err(client_err.to_string());
    }
    let client = client_create_result.unwrap();
    
    let db_instance = client.database(&conncetion_settings.instance);
    let user_collcetion:Collection<Document> = db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);

    let new_user_uuid = uuid::Uuid::new_v4();
    let insert_doc = doc! {"user_id": mongodb::bson::Uuid::from_uuid_0_8(new_user_uuid),
                                     "user_name":&some_credentials.username,
                                     "password": &some_credentials.password.expose_secret()};

    let insert_result = user_collcetion.insert_one(insert_doc, None).await;
    if insert_result.is_err(){
        let insert_err = &insert_result.unwrap_err();
        warn!(target:"app::FinanceOverView","{}",insert_err);
        return Err(insert_err.to_string()); 
    }
    
    debug!(target:"app::FinanceOverView","new id of user object (not user_id): {}",insert_result.unwrap().inserted_id);

    return Ok(new_user_uuid);
    }

    pub async fn get_stored_credentials_by_name(conncetion_settings: &DbConnectionSetting, user_name:&String) -> Result<StoredCredentials,String>
    {
     // Get a handle to the deployment.
     let client_create_result = DbHandlerMongoDB::create_client_connection(conncetion_settings);
     if client_create_result.is_err()
     {
         let client_err = &client_create_result.unwrap_err();
         warn!(target:"app::FinanceOverView","{}",client_err);
         return Err(client_err.to_string());
     }
     let client = client_create_result.unwrap();
     
     let db_instance = client.database(&conncetion_settings.instance);
     let user_collcetion:Collection<Document> = db_instance.collection(DbHandlerMongoDB::COLLECTION_NAME_USER_LIST);       

     let filter = doc!{"user_name":&user_name};

     let projection = doc!{"user_name":<i32>::from(1),
                                     "user_id":<i32>::from(1),
                                     "password":<i32>::from(1)};

    let options=FindOptions::builder().projection(projection).build();

    let query_execute_result = user_collcetion.find(filter, options).await;
    
    if query_execute_result.is_err()
    {
        return Result::Err(query_execute_result.unwrap_err().to_string());
    }

    let mut cursor = query_execute_result.unwrap();
    
    let mut doc_counter=0;
    let mut result_doc=doc!{};
  
    while let Some(data_doc) = cursor.next().await{
        doc_counter=doc_counter+1;
        if data_doc.is_err() {
            return Err(data_doc.unwrap_err().to_string());
        }
        

        let inner_doc = data_doc.unwrap();
        let stored_name = inner_doc.get_str("user_name");
        if stored_name.is_err(){
            return Err(stored_name.unwrap_err().to_string());
        }
        if stored_name.unwrap().eq(user_name){
            result_doc = inner_doc;
        }
    }

    if doc_counter != 1{
        return Err(format!("found {} entries",doc_counter));
    }
/*
    let stored_user_id_as_bytes_read = result_doc.get_binary_generic("user_id");
    if stored_user_id_as_bytes_read.is_err()
    {return Err(stored_user_id_as_bytes_read.unwrap_err().to_string());}
*/


    let stored_password_read =result_doc.get_str("password");
    if stored_password_read.is_err()
    {return Err(stored_password_read.unwrap_err().to_string());}

    let some_uuid_parse_result = ConvertTools::get_uuid_from_document(&result_doc,"user_id");

    if some_uuid_parse_result.is_err()
    {return Err("Could not parse UUID".to_string());}

    let some_password =secrecy::Secret::<String>::new(stored_password_read.unwrap().to_string());

    let some_cred=StoredCredentials { user_id:some_uuid_parse_result.unwrap(), password: some_password };

    return Ok(some_cred);
                                 
    }

}
