use std::time::Duration;

use mongodb::{options::{ClientOptions, Credential, AuthMechanism, ServerAddress}, Client};
use futures::{executor, stream::Collect};

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
    // Parse a connection string into an options struct.
    let v = executor::block_on(ClientOptions::parse(conncetion_settings.url));
    if v.is_err(){
        println!("error could not parse url: {}",v.unwrap_err());
        return false;
    }
    let mut client_options = v.unwrap();
    //sett credentials
    client_options.credential = Some(Credential::builder().username(conncetion_settings.user).password(conncetion_settings.password).source(Some(conncetion_settings.instance)).build());
    

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();
    

    // List the names of the databases in that deployment.
    let query_result = executor::block_on(client.list_database_names(None, None));
    if query_result.is_err(){
        println!("error listing databases: {}",query_result.unwrap_err());
        return false;
    }
    for db_name in query_result.unwrap() {
        println!("{}", db_name);
    }

    return true;
    }
    
}