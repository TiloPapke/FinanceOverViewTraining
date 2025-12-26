use axum::{http::HeaderMap, response::IntoResponse, Form};
use axum_session_mongo::SessionMongoSession;
use secrecy::SecretBox;

use crate::{
    ajax_handle::SimpleAjaxRequestResult,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB, EmailVerificationStatus},
    datatypes::GenerallUserData,
    frontend_functions::save_general_userdata,
    //    session_data_handle::{SessionData, SessionDataResult},
    setting_struct::SettingStruct,
};

use async_session::chrono::Utc;

pub(crate) async fn validate_user_email(
    db_connection: &DbConnectionSetting,
    user_name: &String,
    email_secret: &SecretBox<String>,
) -> Result<EmailVerificationStatus, String> {
    let validate_result =
        DbHandlerMongoDB::verify_email_by_name(&db_connection, user_name, email_secret).await;
    if validate_result.is_err() {
        return Err(format!(
            "Error during verfiy_email_by_name: {}",
            validate_result.unwrap_err()
        ));
    } else {
        return Ok(validate_result.unwrap());
    }
}

pub async fn do_update_general_user_data(
    session: SessionMongoSession,
    Form(input): Form<GenerallUserData>,
) -> impl IntoResponse {
    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let session_expire_timestamp = format!(
            "{} UTC",
            Utc::now().naive_local().format("%Y-%m-%d %H:%M:%S")
        );
        let return_value = SimpleAjaxRequestResult {
            result: "not logged in".to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return (headers, return_value);
    }

    //let username: String = session.get("user_name").unwrap();

    let local_settings: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(local_settings.backend_database_url),
        user: String::from(local_settings.backend_database_user),
        password: String::from(local_settings.backend_database_password),
        instance: String::from(local_settings.backend_database_instance),
    };

    let ajax_return_result: String;
    let username: String = session.get("user_name").unwrap();

    let update_result_async = save_general_userdata(&db_connection, &username, &input);
    let update_result = update_result_async.await;

    if update_result.is_err() {
        ajax_return_result = update_result.unwrap_err().to_string();
    } else {
        ajax_return_result = "information stored".to_string();
    }

    session.update();
    let session_expire_timestamp = format!(
        "{} UTC",
        Utc::now().naive_local().format("%Y-%m-%d %H:%M:%S")
    );

    let return_value = crate::ajax_handle::SimpleAjaxRequestResult {
        result: ajax_return_result,
        new_expire_timestamp: session_expire_timestamp,
    };

    (headers, return_value)
}
