mod database_handler_couchbase;
mod setting_struct;
mod mdb_convert_tools;

use axum::routing::get;
use axum::{response::Html, Router};
use database_handler_couchbase::DbConnectionSetting;
use database_handler_couchbase::DbHandlerCouchbase;
use futures::StreamExt;
use futures::executor;
use mdb_convert_tools::MdbConvertTools;
use mongodb::Collection;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::results::CollectionType;
use setting_struct::SettingStruct;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    //get configuration from ini file
    //define default content

    let working_dir = env::current_dir().unwrap();
    let config_dir:PathBuf = Path::new(&working_dir).join("config");
    if !config_dir.exists()
    {
        fs::create_dir_all(&config_dir).ok();
    }
    let server_settings_file = Path::new(&config_dir).join("ServerSettings.ini");
    let dummy_server_settings_file = Path::new(&config_dir).join("DUMMY_ServerSettings.ini");
    if !dummy_server_settings_file.exists()
    {
        SettingStruct::create_dummy_setting(&dummy_server_settings_file);
    }
    if !server_settings_file.exists()
    {
        println!("No ServerSettings.ini file found, exiting");
        return
    }

    let local_setting = SettingStruct::load_from_file(&server_settings_file);

    setting_struct::GLOBAL_SETTING.set(local_setting.clone()).ok();

    let db_connection=DbConnectionSetting{
        url: String::from(local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password) ,
        instance: String::from(local_setting.backend_database_instance)
    };

    if !DbHandlerCouchbase::validate_db_structure(&db_connection){
        println!("Could not validate backend structure, quitting");
        return;
    }

    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let addr = SocketAddr::from(([local_setting.web_server_ip_part1, local_setting.web_server_ip_part2, local_setting.web_server_ip_part3, local_setting.web_server_ip_part4], local_setting.web_server_port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}   

async fn handler() -> Html<String> {
    
    let local_settings:SettingStruct = SettingStruct::global().clone();
    let db_connection=DbConnectionSetting{
        url: String::from(local_settings.backend_database_url),
        user: String::from(local_settings.backend_database_user),
        password: String::from(local_settings.backend_database_password) ,
        instance: String::from(local_settings.backend_database_instance)
    };
    let current_route="main";
    let mut current_count=1;
    let mut current_document = Document::new();
    let mut documents_found =0;
    

    let mut addtional_info=String::new();

    let query_filter = doc!{"RouteName":current_route};
 
    let query_site_result_cursor = DbHandlerCouchbase::query_table_with_filter(&db_connection, &DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),query_filter);
    if query_site_result_cursor.is_ok(){
        let document_list = MdbConvertTools::get_vector_from_cursor(query_site_result_cursor.unwrap());
        for document_entry in document_list {
            if let Some(&Bson::String(ref route_value)) = document_entry.get("RouteName") {
                if let Some(&Bson::Int32(ref calling_amount_value)) = document_entry.get("CallingAmount") {
                    println!("route: {}, called: {}",route_value,calling_amount_value);
                    if  route_value.eq(current_route)
                    {
                        current_document = document_entry.clone();
                        documents_found = documents_found+1;
                    }
                }
            }
        }

        match documents_found {
            0 => {
                addtional_info = "<br> first documented call".to_string();

                current_document = doc!{
                    "RouteName" : current_route,
                    "CallingAmount": Bson::Int32(current_count)
                };

                let insert_result = DbHandlerCouchbase::insert_document_in_table(&db_connection,&DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),&current_document);
                if insert_result.is_err(){
                    addtional_info = format!("{}<br>could not insert into database",addtional_info);
                }
            },
            1=> {
                let already_called_amount = current_document.get_i32("CallingAmount").unwrap_or_else(|_| {0});
                addtional_info = format!("<br> already called {} times",already_called_amount+current_count);

                current_count=current_count+already_called_amount;
                
                let query_info = doc!{
                    "RouteName" : current_route
                };

                let update_info = doc!{
                    "$set": { "CallingAmount": current_count }
                };
                let update_result = DbHandlerCouchbase::update_document_in_table(&db_connection,&DbHandlerCouchbase::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),query_info, update_info);
                if update_result.is_err(){
                    addtional_info = format!("{}<br>could not update database",addtional_info);
                }
            },
            _ => {
                addtional_info = "<br> multiple records found".to_string();
            },
        };
    }
    else {
        addtional_info = "<br> Could not get calling information".to_string();
    }



    Html(format!("<h1>Hello, World!</h1><br>running on port {}{}",local_settings.web_server_port, addtional_info))
}