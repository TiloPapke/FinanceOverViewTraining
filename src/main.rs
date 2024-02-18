mod ajax_handle;
mod convert_tools;
mod database_handler_mongodb;
pub mod datatypes;
mod frontend_functions;
mod html_render;
mod mail_handle;
mod mdb_convert_tools;
mod password_handle;
mod session_data_handle;
pub mod setting_struct;
mod user_handling;
mod tests {
    mod testing_convert_tools;
    mod testing_email_smtp;
    mod testing_email_validation;
}

use async_mongodb_session::MongodbSessionStore;
use axum::{http::{self, HeaderMap, Uri}, response::{IntoResponse, Redirect}, routing::{get, post}, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use log::{debug, error, info, trace, warn, LevelFilter};
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::json::JsonEncoder,
};
use mongodb::bson::{doc, Bson, Document};
use session_data_handle::SessionDataResult;
use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::{database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB}, html_render::{invalid_handler, registration_incomplete_handler, HtmlTemplate, MainPageTemplate}, mdb_convert_tools::MdbConvertTools, setting_struct::SettingStruct};

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

    let log4rs_create_result = log4rs::init_config(config);
    if log4rs_create_result.is_err() {
        println!(
            "Could not instatiate log mechanismn, {}",
            log4rs_create_result.unwrap_err()
        );
        return;
    }
    let log4rs_handle = log4rs_create_result.unwrap();

    //get configuration from ini file
    let working_dir = env::current_dir().unwrap();
    let config_dir: PathBuf = Path::new(&working_dir).join("config");
    if !config_dir.exists() {
        warn!(target: "app::FinanceOverView","setting folder not found, will be created at {}",config_dir.to_string_lossy());
        fs::create_dir_all(&config_dir).ok();
    }
    let server_settings_file = Path::new(&config_dir).join("ServerSettings.ini");
    let dummy_server_settings_file = Path::new(&config_dir).join("DUMMY_ServerSettings.ini");
    if !dummy_server_settings_file.exists() {
        debug!(target: "app::FinanceOverView","Dummy setting file not found, will be created at {}",dummy_server_settings_file.to_string_lossy());
        SettingStruct::create_dummy_setting(&dummy_server_settings_file);
    }
    if !server_settings_file.exists() {
        error!(target: "app::FinanceOverView","setting folder not found, will be created at {}",server_settings_file.to_string_lossy());
        println!("No ServerSettings.ini file found, exiting");
        return;
    }

    let local_setting = SettingStruct::load_from_file(&server_settings_file);

    setting_struct::GLOBAL_SETTING
        .set(local_setting.clone())
        .ok();

    //real logger configuration from settings
    let log4rs_update_result =
        log4rs::config::load_config_file(local_setting.log_config_path, Default::default());
    if log4rs_update_result.is_err() {
        println!(
            "Could not load log settings, {}",
            log4rs_update_result.unwrap_err()
        );
        return;
    }
    log4rs_handle.set_config(log4rs_update_result.unwrap());

    error!(target:"app::FinanceOverView","checking ERROR log");
    warn!(target:"app::FinanceOverView","checking WARN log");
    info!(target:"app::FinanceOverView","checking INFO log");
    debug!(target:"app::FinanceOverView","checking DEBUG log");
    trace!(target:"app::FinanceOverView","checking TRACE log");

    //check database
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    if !DbHandlerMongoDB::validate_db_structure(&db_connection) {
        error!(target: "app::FinanceOverView","Could not validate backend structure, quitting");
        println!("Could not validate backend structure, quitting");
        return;
    }

    let http = tokio::spawn(http_server());
    let https = tokio::spawn(https_server());

    // Ignore errors.
    let _ = tokio::join!(http, https);

}

#[allow(dead_code)]
async fn http_server_simple() {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[allow(dead_code)]
async fn https_server_simple() {
    let app = Router::new().route("/", get(|| async { "Hello, https world!" }));

    // configure certificate and private key used by https
    let config_async = RustlsConfig::from_pem_file(
        "config/self-signed-certs/cert.pem",
        "config/self-signed-certs/key.pem",
    )
    .await;

    let config = config_async.unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3333));

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn http_server() {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let app = Router::new().route("/", get(http_handler));
    
    let addr = SocketAddr::from((
        [
            local_setting.web_server_ip_part1,
            local_setting.web_server_ip_part2,
            local_setting.web_server_ip_part3,
            local_setting.web_server_ip_part4,
        ],
        local_setting.web_server_port_http,
    ));
    info!(target: "app::FinanceOverView","http listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn https_server() {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    // Get a handle to the deployment.
    let mgdb_client_create_result = DbHandlerMongoDB::create_client_connection(&db_connection);
    if mgdb_client_create_result.is_err() {
        warn!(target:"app::FinanceOverView","Could not create mongo DB Client {}",mgdb_client_create_result.unwrap_err());
        return;
    }
    let mgdb_client = mgdb_client_create_result.unwrap();

    let server_session_store = MongodbSessionStore::from_client(
        mgdb_client,
        &db_connection.instance,
        DbHandlerMongoDB::COLLECTION_NAME_SESSION_INFO,
    );

    let initilize_result = server_session_store.initialize().await;
    if initilize_result.is_err() {
        let error_info = initilize_result.unwrap_err();
        error!(target: "app::FinanceOverView","Could not initialize session store: {}", error_info);
        println!(
            "Could not initialize session store, quitting: {}",
            error_info
        );
        return;
    }

    let app = Router::new()
        .route("/", get(https_handler))
        .route("/invalid", get(invalid_handler))
        .route(
            "/registration_incomplete",
            get(registration_incomplete_handler),
        )
        .route("/do_login", post(html_render::accept_login_form))
        .route("/do_create", post(html_render::create_login_handler))
        .route("/user_home", get(html_render::user_home_handler))
        .route("/do_logout", post(html_render::do_logout_handler))
        .route("/do_changePasswort", post(ajax_handle::do_change_passwort))
        .route(
            "/do_changeUserInfo",
            post(user_handling::do_update_general_user_data),
        )
        .route("/registerUser", get(html_render::register_user_handler))
        .route(
            "/do_register_via_email",
            post(ajax_handle::do_register_user_via_email),
        )
        .route(
            "/verify_email",
            get(html_render::validate_user_email_handler),
        )
        .route(
            "/getPasswordResetToken",
            get(html_render::display_paswword_reset_token_request_page),
        )
        .route(
            "/do_RequestPasswordResetToken",
            post(ajax_handle::do_request_password_reset),
        )
        .route(
            "/reset_password",
            get(html_render::display_paswword_reset_with_token_page),
        )
        .route("/do_reset_password", post(ajax_handle::do_change_password))
        .route("/js_code/*path", get(ajax_handle::get_js_files))
        .layer(Extension(server_session_store));

    let config_result = RustlsConfig::from_pem_file(
        local_setting.web_server_cert_cert_path,
        local_setting.web_server_cert_key_path,
    )
    .await;

    if config_result.is_err() {
        println!(
            "Error loading TLS configuration: {}",
            config_result.as_ref().unwrap_err()
        );
        error!(target: "app::FinanceOverView","Error loading TLS configuration: {}",config_result.unwrap_err());
        return;
    }

    let config = config_result.unwrap();

    let addr = SocketAddr::from((
        [
            local_setting.web_server_ip_part1,
            local_setting.web_server_ip_part2,
            local_setting.web_server_ip_part3,
            local_setting.web_server_ip_part4,
        ],
        local_setting.web_server_port_https,
    ));
    info!(target: "app::FinanceOverView","https listening on {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn http_handler(uri: Uri) -> Redirect {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let host_check = uri.host();
    let host_info; //see https://github.com/rust-lang/rust/issues/49171
    if host_check.is_some() {
        host_info = format!(
            "{}:{}",
            host_check.unwrap(),
            local_setting.web_server_port_https
        );
    } else {
        let addr = SocketAddr::from((
            [
                local_setting.web_server_ip_part1,
                local_setting.web_server_ip_part2,
                local_setting.web_server_ip_part3,
                local_setting.web_server_ip_part4,
            ],
            local_setting.web_server_port_https,
        ));
        host_info = addr.to_string();
    }
    let new_uri = format!("https://{}{}", host_info, uri.path());

    trace!(target:"app::FinanceOverView","Redirecting to {}",new_uri);

    Redirect::temporary(&new_uri)
}

async fn https_handler(session_data: SessionDataResult) -> impl IntoResponse {
    let (headers, user_id, create_cookie) = match session_data {
        SessionDataResult::FoundSessionData(session_data) => {
            (HeaderMap::new(), session_data.user_id, false)
        }
        SessionDataResult::CreatedSessionData(new_session_data) => {
            let mut headers = HeaderMap::new();
            headers.insert(http::header::SET_COOKIE, new_session_data.cookie);
            (headers, new_session_data.user_id, true)
        }
    };
    debug!(
        "user_id is: {}\rcreate:cookie is {}",
        &user_id, &create_cookie
    );

    let local_settings: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(local_settings.backend_database_url),
        user: String::from(local_settings.backend_database_user),
        password: String::from(local_settings.backend_database_password),
        instance: String::from(local_settings.backend_database_instance),
    };
    let current_route = "main";
    let mut current_count = 1;
    let mut current_document = Document::new();
    let mut documents_found = 0;

    let mut addtional_info; //see https://github.com/rust-lang/rust/issues/49171

    let query_filter = doc! {"RouteName":current_route};

    let query_site_result_cursor = DbHandlerMongoDB::query_table_with_filter(
        &db_connection,
        &DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),
        query_filter,
    );
    if query_site_result_cursor.is_ok() {
        let document_list =
            MdbConvertTools::get_vector_from_cursor(query_site_result_cursor.unwrap());
        for document_entry in document_list {
            if let Some(&Bson::String(ref route_value)) = document_entry.get("RouteName") {
                if let Some(&Bson::Int32(ref calling_amount_value)) =
                    document_entry.get("CallingAmount")
                {
                    trace!(target:"app::FinanceOverView","route: {}, called: {}",route_value,calling_amount_value);

                    if route_value.eq(current_route) {
                        current_document = document_entry.clone();
                        documents_found = documents_found + 1;
                    }
                }
            }
        }

        match documents_found {
            0 => {
                addtional_info = "<br> first documented call".to_string();

                current_document = doc! {
                    "RouteName" : current_route,
                    "CallingAmount": Bson::Int32(current_count)
                };

                let insert_result = DbHandlerMongoDB::insert_document_in_table(
                    &db_connection,
                    &DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),
                    &current_document,
                );
                if insert_result.is_err() {
                    addtional_info =
                        format!("{}<br>could not insert into database", addtional_info);
                }
            }
            1 => {
                let already_called_amount = current_document
                    .get_i32("CallingAmount")
                    .unwrap_or_else(|_| 0);
                addtional_info = format!(
                    "<br> already called {} times",
                    already_called_amount + current_count
                );

                current_count = current_count + already_called_amount;

                let query_info = doc! {
                    "RouteName" : current_route
                };

                let update_info = doc! {
                    "$set": { "CallingAmount": current_count }
                };
                let update_result = DbHandlerMongoDB::update_document_in_table(
                    &db_connection,
                    &DbHandlerMongoDB::COLLECTION_NAME_WEBSITE_TRAFFIC.to_string(),
                    query_info,
                    update_info,
                );
                if update_result.is_err() {
                    addtional_info = format!("{}<br>could not update database", addtional_info);
                }
            }
            _ => {
                addtional_info = "<br> multiple records found".to_string();
            }
        };
    } else {
        warn!(
            "Error querying database: {}",
            query_site_result_cursor.unwrap_err()
        );
        addtional_info = "<br> Could not get calling information".to_string();
    }

    //Html(format!("<h1>Hello, World!</h1><br>running on port {}{}",local_settings.web_server_port_https, addtional_info))

    let template = MainPageTemplate {
        web_running_port: local_settings.web_server_port_https,
        additional_info: addtional_info,
        called_times: current_count,
    };

    (headers, HtmlTemplate(template))
}