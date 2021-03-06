mod database_handler_mongodb;
mod setting_struct;
mod mdb_convert_tools;

use axum::response::Redirect;
use axum::routing::get;
use axum::{response::Html, Router};
use axum::http::uri::Uri;
use axum_server;
use axum_server::tls_rustls::RustlsConfig;
use database_handler_mongodb::DbConnectionSetting;
use database_handler_mongodb::DbHandlerMongoDB;
use log::{error, warn, debug, trace, info, LevelFilter};
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::json::JsonEncoder,
};
use mdb_convert_tools::MdbConvertTools;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use mongodb::bson::doc;
use setting_struct::SettingStruct;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
 
    //default logger for startup
    let stdout: ConsoleAppender = ConsoleAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build();
    let config = log4rs::config::Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .build(Root::builder().appender("stdout").build(LevelFilter::Info))
    .unwrap();

    let log4rs_create_result= log4rs::init_config(config);
    if log4rs_create_result.is_err(){
        println!("Could not instatiate log mechanismn, {}",log4rs_create_result.unwrap_err());
        return  
    }
    let log4rs_handle = log4rs_create_result.unwrap(); 

    //get configuration from ini file
    let working_dir = env::current_dir().unwrap();
    let config_dir:PathBuf = Path::new(&working_dir).join("config");
    if !config_dir.exists()
    {
        warn!(target: "app::FinanceOverView","setting folder not found, will be created at {}",config_dir.to_string_lossy());
        fs::create_dir_all(&config_dir).ok();
    }
    let server_settings_file = Path::new(&config_dir).join("ServerSettings.ini");
    let dummy_server_settings_file = Path::new(&config_dir).join("DUMMY_ServerSettings.ini");
    if !dummy_server_settings_file.exists()
    {
        debug!(target: "app::FinanceOverView","Dummy setting file not found, will be created at {}",dummy_server_settings_file.to_string_lossy());
        SettingStruct::create_dummy_setting(&dummy_server_settings_file);
    }
    if !server_settings_file.exists()
    {
        error!(target: "app::FinanceOverView","setting folder not found, will be created at {}",server_settings_file.to_string_lossy());
        println!("No ServerSettings.ini file found, exiting");
        return
    }

    let local_setting = SettingStruct::load_from_file(&server_settings_file);

    setting_struct::GLOBAL_SETTING.set(local_setting.clone()).ok();

    //real logger configuration from settings
    let log4rs_update_result= log4rs::config::load_config_file(local_setting.log_config_path, Default::default());
    if log4rs_update_result.is_err(){
        println!("Could not load log settings, {}",log4rs_update_result.unwrap_err());
        return  
    }
    log4rs_handle.set_config(log4rs_update_result.unwrap());


    error!(target:"app::FinanceOverView","checking ERROR log");
    warn!(target:"app::FinanceOverView","checking WARN log");
    info!(target:"app::FinanceOverView","checking INFO log");
    debug!(target:"app::FinanceOverView","checking DEBUG log");
    trace!(target:"app::FinanceOverView","checking TRACE log");

    //check database
    let db_connection=DbConnectionSetting{
        url: String::from(local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password) ,
        instance: String::from(local_setting.backend_database_instance)
    };

    if !DbHandlerMongoDB::validate_db_structure(&db_connection){
        error!(target: "app::FinanceOverView","Could not validate backend structure, quitting");
        println!("Could not validate backend structure, quitting");
        return;
    }
    
    let http = tokio::spawn(http_server());
    let https = tokio::spawn(https_server());

    // Ignore errors.
    let _ = tokio::join!(http, https);


}   

async fn http_server() {

    let local_setting:SettingStruct = SettingStruct::global().clone();
    let app = Router::new().route("/", get(http_handler));

    let addr = SocketAddr::from(([local_setting.web_server_ip_part1, local_setting.web_server_ip_part2, local_setting.web_server_ip_part3, local_setting.web_server_ip_part4], local_setting.web_server_port_http));
    info!(target: "app::FinanceOverView","http listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn https_server() {

    let local_setting:SettingStruct = SettingStruct::global().clone();
    let app = Router::new().route("/", get(https_handler));

    let config_result = RustlsConfig::from_pem_file(
        local_setting.web_server_cert_cert_path,
        local_setting.web_server_cert_key_path,
    )
    .await;
    
    if config_result.is_err()
    {
        println!("Error loading TLS configuration: {}",config_result.as_ref().unwrap_err());
        error!(target: "app::FinanceOverView","Error loading TLS configuration: {}",config_result.unwrap_err());
        return;
    }

    let config = config_result.unwrap();


    let addr = SocketAddr::from(([local_setting.web_server_ip_part1, local_setting.web_server_ip_part2, local_setting.web_server_ip_part3, local_setting.web_server_ip_part4], local_setting.web_server_port_https));
    info!(target: "app::FinanceOverView","https listening on {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn http_handler(uri: Uri) -> Redirect {
    let local_setting:SettingStruct = SettingStruct::global().clone();
    let host_check = uri.host();
    let host_info;//see https://github.com/rust-lang/rust/issues/49171
    if host_check.is_some(){
        host_info=format!("{}:{}",host_check.unwrap(),local_setting.web_server_port_https);
    }
    else {
        let addr = SocketAddr::from(([local_setting.web_server_ip_part1, local_setting.web_server_ip_part2, local_setting.web_server_ip_part3, local_setting.web_server_ip_part4], local_setting.web_server_port_https));
        host_info=addr.to_string();
    }
    let new_uri = format!("https://{}{}",host_info,uri.path());
    
    trace!(target:"app::FinanceOverView","Redirecting to {}",new_uri);

    Redirect::temporary(&new_uri)

}

async fn https_handler() -> Html<String> {
    
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
    

    let mut addtional_info; //see https://github.com/rust-lang/rust/issues/49171

    let query_filter = doc!{"RouteName":current_route};
 
    let query_site_result_cursor = DbHandlerMongoDB::query_table_with_filter(&db_connection, &DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),query_filter);
    if query_site_result_cursor.is_ok(){
        let document_list = MdbConvertTools::get_vector_from_cursor(query_site_result_cursor.unwrap());
        for document_entry in document_list {
            if let Some(&Bson::String(ref route_value)) = document_entry.get("RouteName") {
                if let Some(&Bson::Int32(ref calling_amount_value)) = document_entry.get("CallingAmount") {
                    
                    trace!(target:"app::FinanceOverView","route: {}, called: {}",route_value,calling_amount_value);

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

                let insert_result = DbHandlerMongoDB::insert_document_in_table(&db_connection,&DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),&current_document);
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
                let update_result = DbHandlerMongoDB::update_document_in_table(&db_connection,&DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),query_info, update_info);
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
        warn!("Error querying database: {}",query_site_result_cursor.unwrap_err());
        addtional_info = "<br> Could not get calling information".to_string();
    }

    Html(format!("<h1>Hello, World!</h1><br>running on port {}{}",local_settings.web_server_port_https, addtional_info))
}