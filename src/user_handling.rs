use axum::{http::HeaderMap, response::IntoResponse, Form};
use secrecy::Secret;

use crate::{
    ajax_handle::SimpleAjaxRequestResult,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB, EmailVerificationStatus},
    datatypes::GenerallUserData,
    frontend_functions::save_general_userdata,
    session_data_handle::{SessionData, SessionDataResult},
    setting_struct::SettingStruct,
};

use async_session::{
    chrono::{DateTime, Utc},
    SessionStore,
};

pub(crate) async fn validate_user_email(
    db_connection: &DbConnectionSetting,
    user_name: &String,
    email_secret: &Secret<String>,
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
    session_data: SessionDataResult,
    Form(input): Form<GenerallUserData>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
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

    if session.is_expired() {
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );
        let return_value = SimpleAjaxRequestResult {
            result: "Session expired".to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        (headers, return_value)
    } else {
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

        session.expire_in(std::time::Duration::from_secs(60 * 1));
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );

        let return_value = crate::ajax_handle::SimpleAjaxRequestResult {
            result: ajax_return_result,
            new_expire_timestamp: session_expire_timestamp,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (headers, return_value)
    }
}