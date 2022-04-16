mod setting_struct;

use axum::{response::Html, routing::get, Router};
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
    if !server_settings_file.exists()
    {
        SettingStruct::create_dummy_setting(&server_settings_file);
    }

        let local_setting = SettingStruct::load_from_file(&server_settings_file);

        setting_struct::GLOBAL_SETTING.set(local_setting).ok();

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
    
    let local_settings = SettingStruct::global();

    Html(format!("<h1>Hello, World!</h1><br>running on port {}",local_settings.web_server_port))
}