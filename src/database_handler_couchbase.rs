use mongodb::{options::{ClientOptions, Credential}, Client, Collection, Cursor, results::{InsertOneResult, UpdateResult}, bson::Document};
use futures::executor;

pub struct DbConnectionSetting{
    pub url: String,
    pub user: String,
    pub password: String,
    pub instance: String
}

pub struct DbHandlerCouchbase{

}

impl DbHandlerCouchbase{
    pub const COLLECTION_NAME_GENERAL_INFORMATION:&'static str ="GeneralInformation";
    pub const COLLECTION_NAME_WEBSITE_TRAFFIC:&'static str ="WebSiteTraffic";

    pub fn validate_db_structure(conncetion_settings: &DbConnectionSetting) -> bool
    {


    // Get a handle to the deployment.
    let client_create_result = DbHandlerCouchbase::create_client_connection(conncetion_settings);
    if client_create_result.is_err()
    {
        println!("{}",client_create_result.unwrap_err());
        return false;
    }
    let client = client_create_result.unwrap();
    

    // List the names of the databases in that deployment.
    let query_result = executor::block_on(client.list_database_names(None, None));
    if query_result.is_err(){
        //#[cfg(debug_assertions)]
        println!("error listing databases: {}",query_result.as_ref().unwrap_err());

        warn!("error listing databases: {}",query_result.unwrap_err());
        return false;
    }

    let instance_list = query_result.unwrap();
    if cfg!(debug_assertions){
        // debug build
        for db_name in &instance_list {
            println!("{}", db_name);
        }
    }

    if !instance_list.contains(&conncetion_settings.instance)
    {
       //#[cfg(debug_assertions)]
        println!("entry {} not found in list of database names, HINT: does it have at least one collection?", conncetion_settings.instance);

        warn!("entry {} not found in list of database names, HINT: does it have at least one collection?", conncetion_settings.instance);
        return false;
    }

    let db_instance = client.database(&conncetion_settings.instance);

    let query_result_collections = executor::block_on(db_instance.list_collection_names(None));
    if query_result_collections.is_err(){
        //#[cfg(debug_assertions)]
        println!("error listing collections: {}",query_result_collections.as_ref().unwrap_err());

        warn!("error listing collections: {}",query_result_collections.unwrap_err());
        return false;
    }
    let collection_list = query_result_collections.unwrap();
    if collection_list.contains(&DbHandlerCouchbase::COLLECTION_NAME_GENERAL_INFORMATION.to_string())
    {   
        //#[cfg(debug_assertions)]
        println!("found collection {}",DbHandlerCouchbase::COLLECTION_NAME_GENERAL_INFORMATION);
    }
    else
    {
        //#[cfg(debug_assertions)]
        println!("collection {} not found, trying to create it",DbHandlerCouchbase::COLLECTION_NAME_GENERAL_INFORMATION);
        let create_result=executor::block_on( db_instance.create_collection(DbHandlerCouchbase::COLLECTION_NAME_GENERAL_INFORMATION,None));
        if create_result.is_err(){
            //#[cfg(debug_assertions)]
            println!("could not create collection {} in database {}, error: {}",DbHandlerCouchbase::COLLECTION_NAME_GENERAL_INFORMATION, conncetion_settings.instance, create_result.as_ref().unwrap_err());

            warn!("could not create collection {} in database {}, error: {}",DbHandlerCouchbase::COLLECTION_NAME_GENERAL_INFORMATION, conncetion_settings.instance, create_result.unwrap_err());
            return false
        }
    }
    if collection_list.contains(&DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string())
    {
        //#[cfg(debug_assertions)]
        println!("found collection {}",DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC);
    }
    else
    {
        //#[cfg(debug_assertions)]
        println!("collection {} not found, trying to create it",DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC);

        let create_result=executor::block_on( db_instance.create_collection(&DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC,None));
        if create_result.is_err(){
            //#[cfg(debug_assertions)]
            println!("could not create collection {} in database {}, error: {}",DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC, conncetion_settings.instance, create_result.as_ref().unwrap_err());
            
            warn!("could not missing create collection {} in database {}, error: {}",DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC, conncetion_settings.instance, create_result.unwrap_err());
            return false
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
        let client_create_result = DbHandlerCouchbase::create_client_connection(conncetion_settings);
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
        let client_create_result = DbHandlerCouchbase::create_client_connection(conncetion_settings);
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
        let client_create_result = DbHandlerCouchbase::create_client_connection(conncetion_settings);
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
        //sett credentials
        let co_source = conncetion_settings.instance.to_string();
        client_options.credential = Some(Credential::builder().username(conncetion_settings.user.clone()).password(conncetion_settings.password.clone()).source(Some(co_source)).build());
        

        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();
        return Result::Ok(client)
    }
}