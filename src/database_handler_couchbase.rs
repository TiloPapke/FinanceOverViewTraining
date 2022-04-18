use mongodb::{options::{ClientOptions, Credential}, Client};
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
    pub fn validate_db_structure(conncetion_settings: DbConnectionSetting) -> bool
    {
        let collection_name_general_information:String ="GeneralInformation".to_string();

    // Parse a connection string into an options struct.
    let v = executor::block_on(ClientOptions::parse(conncetion_settings.url));
    if v.is_err(){
        println!("error could not parse url: {}",v.unwrap_err());
        return false;
    }
    let mut client_options = v.unwrap();
    //sett credentials
    let co_source = conncetion_settings.instance.to_string();
    client_options.credential = Some(Credential::builder().username(conncetion_settings.user).password(conncetion_settings.password).source(Some(co_source)).build());
    

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();
    

    // List the names of the databases in that deployment.
    let query_result = executor::block_on(client.list_database_names(None, None));
    if query_result.is_err(){
        println!("error listing databases: {}",query_result.unwrap_err());
        return false;
    }

    let instance_list = query_result.unwrap();
    for db_name in &instance_list {
        println!("{}", db_name);
    }

    if !instance_list.contains(&conncetion_settings.instance)
    {
        println!("entry {} not found in list of database names, HINT: does it have at least one collection?", conncetion_settings.instance);
        return false;
    }

    let db_instance = client.database(&conncetion_settings.instance);

    let query_result_collections = executor::block_on(db_instance.list_collection_names(None));
    if query_result_collections.is_err(){
        println!("error listing collections: {}",query_result_collections.unwrap_err());
        return false;
    }
    let collection_list = query_result_collections.unwrap();
    if collection_list.contains(&collection_name_general_information)
    {println!("found collection {}",collection_name_general_information);}
    else
    {
        println!("collection {} not found, trying to create it",collection_name_general_information);
        let create_result=executor::block_on( db_instance.create_collection(&collection_name_general_information,None));
        if create_result.is_err(){
            println!("could not create collection {} in database {}, error: {}",collection_name_general_information, conncetion_settings.instance, create_result.unwrap_err());
            return false
        }
    }

    return true;
    }
    
}