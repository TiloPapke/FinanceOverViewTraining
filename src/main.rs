use axum::{response::Html, routing::get, Router};
use ini::Ini;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    //get configuration from ini file
    //define default content
    let web_server_ip_part1:u8 = 127;
    let web_server_ip_part2:u8 = 0;
    let web_server_ip_part3:u8 = 0;
    let web_server_ip_part4:u8 = 1;
    let web_server_port:u16=3000;
    let mut conf:Ini = Ini::new();

    let working_dir = env::current_dir().unwrap();
    let config_dir:PathBuf = Path::new(&working_dir).join("config");
    if !config_dir.exists()
    {
        fs::create_dir_all(&config_dir).ok();
    }
    let server_settings_file = Path::new(&config_dir).join("ServerSettings.ini");
    if !server_settings_file.exists()
    {
        conf.with_section(Some("[WARNING]"))
            .set("GITWARNING", "This is a default config file, when entering own value be sure to add this file to ignore")
            .set("GITWARNING2", "Only push to repository if this file does not contain any private information");
        conf.with_section(Some("WebServer"))
            .set("ip_part1", web_server_ip_part1.to_string())
            .set("ip_part2", web_server_ip_part2.to_string())
            .set("ip_part3", web_server_ip_part3.to_string())
            .set("ip_part4", web_server_ip_part4.to_string())
            .set("port", web_server_port.to_string());
            conf.write_to_file(&server_settings_file).unwrap();
    }
    let conf:Ini = Ini::load_from_file(&server_settings_file).unwrap();
    let web_server_ip_part1:u8 = conf.get_from_or(Some("WebServer"),"ip_part1","127").parse().unwrap();
    let web_server_ip_part2:u8 = conf.get_from_or(Some("WebServer"),"ip_part2","0").parse().unwrap();
    let web_server_ip_part3:u8 = conf.get_from_or(Some("WebServer"),"ip_part3","0").parse().unwrap();
    let web_server_ip_part4:u8 = conf.get_from_or(Some("WebServer"),"ip_part4","1").parse().unwrap();
    let web_server_port:u16 = conf.get_from_or(Some("WebServer"),"port","3000").parse().unwrap();


    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let addr = SocketAddr::from(([web_server_ip_part1, web_server_ip_part2, web_server_ip_part3, web_server_ip_part4], web_server_port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}