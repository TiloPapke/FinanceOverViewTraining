use std::{
    borrow::Borrow,
    env,
    ffi::OsString,
    fs::{self, read_dir},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use askama::Template;
use async_session::{
    chrono::{DateTime, Utc},
    serde_json::json,
    SessionStore,
};
use axum::{
    body::Body,
    extract::Form,
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use log::debug;
use mongodb::bson::Uuid;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::{
    accounting_config_logic::FinanceAccountingConfigHandle,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
    datatypes::{
        FinanceAccount, FinanceAccountType, PasswordResetRequest, PasswordResetTokenRequest,
    },
    frontend_functions::send_password_reset_email,
    html_render::{AccountTemplate, AccountTypeTemplate, HtmlTemplate},
    password_handle::{self, validate_credentials, UserCredentials},
    session_data_handle::{SessionData, SessionDataResult},
    setting_struct::SettingStruct,
};

//from https://github.com/neilwashere/rust-project-root/blob/main/src/lib.rs
fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(p) = path_ancestors.next() {
        let has_cargo = read_dir(p)?
            .into_iter()
            .any(|p| p.unwrap().file_name() == OsString::from("Cargo.lock"));
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}

pub async fn get_js_files(js_uri: Uri) -> impl IntoResponse {
    let mut js_path = js_uri
        .path()
        .to_string()
        .trim_start_matches('/')
        .to_string();
    js_path = js_path.replace('/', &std::path::MAIN_SEPARATOR.to_string());

    let project_root_result = get_project_root();
    if project_root_result.is_err() {
        return Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .body(Body::from(project_root_result.unwrap_err().to_string()))
            .unwrap();
    }

    let total_js_path = Path::new(&project_root_result.unwrap()).join(js_path);
    if total_js_path.is_file() {
        let file_content = fs::read_to_string(total_js_path).unwrap_or_default();
        return Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str("text/javascript").unwrap(),
            )
            .body(Body::from(file_content))
            .unwrap();
    } else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap();
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ChangePasswortFormInput {
    pub password_new_1: Secret<String>,
    pub password_new_2: Secret<String>,
    pub password_old: Secret<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SimpleAjaxRequestResult {
    pub result: String,
    pub new_expire_timestamp: String,
}

impl IntoResponse for SimpleAjaxRequestResult {
    fn into_response(self) -> Response {
        return Json(json!(self)).into_response();
    }
}

pub async fn do_change_passwort(
    session_data: SessionDataResult,
    Form(input): Form<ChangePasswortFormInput>,
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

    let username: String = session.get("user_name").unwrap();

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
        let change_result: String;
        let password_new_1 = input.password_new_1.clone();
        let password_new_2 = input.password_new_2.clone();

        let compare_result = password_handle::compare_password(&password_new_1, &password_new_2);

        if compare_result.is_err() {
            change_result = compare_result.unwrap_err().to_string();
        } else {
            let credentials = UserCredentials {
                username: username.clone(),
                password: input.password_old.clone(),
            };
            let local_settings: SettingStruct = SettingStruct::global().clone();
            let db_connection = DbConnectionSetting {
                url: String::from(local_settings.backend_database_url),
                user: String::from(local_settings.backend_database_user),
                password: String::from(local_settings.backend_database_password),
                instance: String::from(local_settings.backend_database_instance),
            };

            match validate_credentials(&db_connection, &credentials).await {
                Ok(user_id) => {
                    debug!(target: "app::FinanceOverView","trying to change password for user {}", user_id);

                    let credentials_new = UserCredentials {
                        username: username.clone(),
                        password: password_new_1.clone(),
                    };

                    let update_result =
                        password_handle::update_user_password(&db_connection, &credentials_new)
                            .await;

                    if update_result.is_err() {
                        change_result = format!(
                            "error updating password: {}",
                            update_result.unwrap_err().to_string()
                        );
                    } else {
                        change_result = "password change successfull".to_string();
                    }
                }
                Err(_) => {
                    debug!(target: "app::FinanceOverView","no old password not valid");

                    change_result = "old password did not match".to_string();
                }
            }
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

        let return_value = SimpleAjaxRequestResult {
            result: change_result,
            new_expire_timestamp: session_expire_timestamp,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (headers, return_value)
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ChangeResetSecretFormInput {
    pub new_reset_secret: Secret<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct ChangeResetSecretResponse {
    pub result: String,
}

impl IntoResponse for ChangeResetSecretResponse {
    fn into_response(self) -> Response {
        return Json(json!(self)).into_response();
    }
}

pub async fn do_change_reset_secret(
    session_data: SessionDataResult,
    Form(input): Form<ChangeResetSecretFormInput>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let return_value = ChangeResetSecretResponse {
            result: "not logged in".to_string(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return (headers, return_value);
    }

    let user_id: Uuid = session.get("user_account_id").unwrap();

    if session.is_expired() {
        let return_value = ChangeResetSecretResponse {
            result: "Session expired".to_string(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        (headers, return_value)
    } else {
        let change_result: String;

        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };

        debug!(target: "app::FinanceOverView","trying to change reset secret for user {}", user_id);

        let update_result = password_handle::update_user_reset_secret(
            &db_connection,
            &user_id,
            &input.new_reset_secret,
        )
        .await;

        if update_result.is_err() {
            change_result = format!(
                "error updating reset secret: {}",
                update_result.unwrap_err().to_string()
            );
        } else {
            change_result = "reset secret change successfull".to_string();
        }

        session.expire_in(std::time::Duration::from_secs(60 * 1));

        let return_value = ChangeResetSecretResponse {
            result: change_result,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (headers, return_value)
    }
}

#[derive(Deserialize, Debug)]
pub struct RegisterUserViaEmailFormInput {
    pub username: String,
    pub password: Secret<String>,
    pub email: String,
}

pub async fn do_register_user_via_email(
    session_data: SessionDataResult,
    Form(input): Form<RegisterUserViaEmailFormInput>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let headers = HeaderMap::new();

    if is_logged_in {
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );
        let return_value = SimpleAjaxRequestResult {
            result: "You are still logged in, please log out before registering new accounts"
                .to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };

        return (headers, return_value);
    }

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
            result: "Session expired, please try again".to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };

        (headers, return_value)
    } else {
        let register_result: String;
        let _new_user_name = &input.username;
        let _new_password = &input.password;
        let _new_email = &input.email;

        session.expire_in(std::time::Duration::from_secs(60 * 1));
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );

        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let register_result_2 = crate::frontend_functions::register_user_with_email_verfication(
            &db_connection,
            _new_user_name,
            _new_password,
            _new_email,
        )
        .await;

        if register_result_2.is_err() {
            register_result = register_result_2.unwrap_err().to_string()
        } else {
            register_result = "OK, please check your E-Mail".to_string();
        };

        let return_value = SimpleAjaxRequestResult {
            result: register_result,
            new_expire_timestamp: session_expire_timestamp,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (headers, return_value)
    }
}

pub async fn do_request_password_reset(
    session_data: SessionDataResult,
    Form(input): Form<PasswordResetTokenRequest>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let headers = HeaderMap::new();

    if is_logged_in {
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );
        let return_value = SimpleAjaxRequestResult {
            result: "You are logged in, please use normal password change function".to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };

        return (headers, return_value);
    }

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
            result: "Session expired, please try again".to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };

        (headers, return_value)
    } else {
        let request_result: String;
        session.expire_in(std::time::Duration::from_secs(60 * 1));
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );

        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let password_reset_request_result =
            crate::password_handle::request_password_reset_token(&db_connection, input.borrow())
                .await;

        if password_reset_request_result.is_err() {
            request_result = password_reset_request_result.unwrap_err().to_string();
        } else {
            //send Email
            let unwrapped_password_reset_request_result = password_reset_request_result.unwrap();
            let send_result = send_password_reset_email(
                input.user_name.borrow(),
                unwrapped_password_reset_request_result.borrow(),
            )
            .await;
            if send_result.is_err() {
                request_result = send_result.unwrap_err().to_string();
            } else {
                request_result =
                    "Password Request successful, please check your e-mail".to_string();
            }
        };

        let return_value = SimpleAjaxRequestResult {
            result: request_result,
            new_expire_timestamp: session_expire_timestamp,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (headers, return_value)
    }
}

pub async fn do_change_password(
    session_data: SessionDataResult,
    Form(input): Form<PasswordResetRequest>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let headers = HeaderMap::new();

    if is_logged_in {
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );
        let return_value = SimpleAjaxRequestResult {
            result: "You are logged in, please use normal password change function".to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };

        return (headers, return_value);
    }

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
            result: "Session expired, please try again".to_string(),
            new_expire_timestamp: session_expire_timestamp,
        };

        (headers, return_value)
    } else {
        let request_result: String;
        session.expire_in(std::time::Duration::from_secs(60 * 1));
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );
        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let password_change_result =
            crate::password_handle::reset_password_with_token(&db_connection, input.borrow()).await;

        if password_change_result.is_err() {
            request_result = password_change_result.unwrap_err().to_string();
        } else {
            //send Email
            let unwrapped_password_change_result = password_change_result.unwrap();
            if unwrapped_password_change_result {
                request_result =
                    "Password Change successful, please login via normal page".to_string();
            } else {
                request_result = "Password Change unsuccessful".to_string();
            }
        };

        let return_value = SimpleAjaxRequestResult {
            result: request_result,
            new_expire_timestamp: session_expire_timestamp,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (headers, return_value)
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateNewFinanceAccountTypeFormInput {
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CreateNewFinanceAccountTypeResponse {
    pub result: String,
    pub new_id: String,
    pub subpage: String,
}

#[derive(Template)]
#[template(path = "AccountingConfig/accountTypeRow.html")]
pub struct AccountTypeCreateResponseTemplate {
    account_type: AccountTypeTemplate,
}

impl IntoResponse for CreateNewFinanceAccountTypeResponse {
    fn into_response(self) -> Response {
        return Json(json!(self)).into_response();
    }
}

pub async fn do_create_new_finance_account_type(
    session_data: SessionDataResult,
    Form(input): Form<CreateNewFinanceAccountTypeFormInput>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let return_value = CreateNewFinanceAccountTypeResponse {
            result: "not logged in".to_string(),
            new_id: "".into(),
            subpage: "".into(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return (StatusCode::BAD_REQUEST, headers, return_value);
    }

    if session.is_expired() {
        let return_value = CreateNewFinanceAccountTypeResponse {
            result: "Session expired, please try again".to_string(),
            new_id: "".into(),
            subpage: "".into(),
        };

        (StatusCode::BAD_REQUEST, headers, return_value)
    } else {
        let create_result: String;
        let new_title = &input.title;
        let new_description = &input.description;
        let new_uuid = Uuid::new();
        let mut new_account_type = FinanceAccountType {
            id: new_uuid,
            title: new_title.into(),
            description: new_description.into(),
        };

        session.expire_in(std::time::Duration::from_secs(60 * 10));

        let db_handler = DbHandlerMongoDB {};
        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let user_id: Uuid = session.get("user_account_id").unwrap();
        let mut return_status_code = StatusCode::OK;
        {
            let mut accounting_config_handle =
                FinanceAccountingConfigHandle::new(&db_connection, &user_id, &db_handler);

            let register_result_2 =
                accounting_config_handle.finance_account_type_upsert(&mut new_account_type);
            {
                if register_result_2.is_err() {
                    return_status_code = StatusCode::BAD_REQUEST;
                    create_result = register_result_2.unwrap_err().to_string()
                } else {
                    create_result = "OK, created".to_string();
                };
            }
        }

        let new_account_type_template = AccountTypeTemplate {
            id: new_account_type.id.to_string(),
            name: new_account_type.title,
            description: new_account_type.description,
        };
        let response_html_result = HtmlTemplate(AccountTypeCreateResponseTemplate {
            account_type: new_account_type_template,
        })
        .0
        .render();
        let return_html = response_html_result.unwrap();

        let return_value = CreateNewFinanceAccountTypeResponse {
            result: create_result,
            new_id: new_account_type.id.to_string(),
            subpage: return_html,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (return_status_code, headers, return_value)
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateFinanceAccountTypeFormInput {
    pub account_type_id: String,
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct UpdateFinanceAccountTypeResponse {
    pub result: String,
}

impl IntoResponse for UpdateFinanceAccountTypeResponse {
    fn into_response(self) -> Response {
        return Json(json!(self)).into_response();
    }
}

pub async fn do_update_finance_account_type(
    session_data: SessionDataResult,
    Form(input): Form<UpdateFinanceAccountTypeFormInput>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let return_value = UpdateFinanceAccountTypeResponse {
            result: "not logged in".to_string(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return (StatusCode::BAD_REQUEST, headers, return_value);
    }

    if session.is_expired() {
        let return_value = UpdateFinanceAccountTypeResponse {
            result: "Session expired, please try again".to_string(),
        };

        (StatusCode::BAD_REQUEST, headers, return_value)
    } else {
        let upsert_result: String;
        let new_title = &input.title;
        let new_description = &input.description;
        let old_uuid = Uuid::parse_str(&input.account_type_id);
        if old_uuid.is_err() {
            debug!(target: "app::FinanceOverView","error in function do_update_finance_account_type, could not parse UUID from input: {}",&input.account_type_id);
            let return_value = UpdateFinanceAccountTypeResponse {
                result: "Error reading data".to_string(),
            };

            return (StatusCode::BAD_REQUEST, headers, return_value);
        }

        let mut old_account_type = FinanceAccountType {
            id: old_uuid.unwrap(),
            title: new_title.into(),
            description: new_description.into(),
        };

        session.expire_in(std::time::Duration::from_secs(60 * 10));

        let db_handler = DbHandlerMongoDB {};
        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let user_id: Uuid = session.get("user_account_id").unwrap();
        let mut return_status_code = StatusCode::OK;
        {
            let mut accounting_config_handle =
                FinanceAccountingConfigHandle::new(&db_connection, &user_id, &db_handler);

            let upsert_result_2 =
                accounting_config_handle.finance_account_type_upsert(&mut old_account_type);
            {
                if upsert_result_2.is_err() {
                    return_status_code = StatusCode::BAD_REQUEST;
                    upsert_result = upsert_result_2.unwrap_err().to_string()
                } else {
                    upsert_result = "OK, aktualisiert".to_string();
                };
            }
        }

        let return_value = UpdateFinanceAccountTypeResponse {
            result: upsert_result,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (return_status_code, headers, return_value)
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateNewFinanceAccountFormInput {
    pub title: String,
    pub description: String,
    pub account_type_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CreateNewFinanceAccountResponse {
    pub result: String,
    pub new_id: String,
    pub subpage: String,
}

#[derive(Template)]
#[template(path = "AccountingConfig/accountRow.html")]
pub struct AccountCreateResponseTemplate {
    account: AccountTemplate,
}

impl IntoResponse for CreateNewFinanceAccountResponse {
    fn into_response(self) -> Response {
        return Json(json!(self)).into_response();
    }
}

pub async fn do_create_new_finance_account(
    session_data: SessionDataResult,
    Form(input): Form<CreateNewFinanceAccountFormInput>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let return_value = CreateNewFinanceAccountResponse {
            result: "not logged in".to_string(),
            new_id: "".into(),
            subpage: "".into(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return (StatusCode::BAD_REQUEST, headers, return_value);
    }

    if session.is_expired() {
        let return_value = CreateNewFinanceAccountResponse {
            result: "Session expired, please try again".to_string(),
            new_id: "".into(),
            subpage: "".into(),
        };

        (StatusCode::BAD_REQUEST, headers, return_value)
    } else {
        let mut create_result: String;
        let new_title = &input.title;
        let new_description = &input.description;
        let new_uuid = Uuid::new();
        let new_finance_account_type_id_result = Uuid::parse_str(&input.account_type_id);
        if new_finance_account_type_id_result.is_err() {
            let return_value = CreateNewFinanceAccountResponse {
                result: new_finance_account_type_id_result.unwrap_err().to_string(),
                new_id: "".into(),
                subpage: "".into(),
            };
            headers.insert(
                axum::http::header::REFRESH,
                axum::http::HeaderValue::from_str("5; url = /").unwrap(),
            );
            return (StatusCode::BAD_REQUEST, headers, return_value);
        }
        let mut new_account = FinanceAccount {
            id: new_uuid,
            title: new_title.into(),
            description: new_description.into(),
            finance_account_type_id: new_finance_account_type_id_result.unwrap(),
        };
        let mut available_types = Vec::new();

        session.expire_in(std::time::Duration::from_secs(60 * 10));

        let db_handler = DbHandlerMongoDB {};
        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let user_id: Uuid = session.get("user_account_id").unwrap();
        let mut return_status_code = StatusCode::OK;
        {
            let mut accounting_config_handle =
                FinanceAccountingConfigHandle::new(&db_connection, &user_id, &db_handler);

            let register_result_2 =
                accounting_config_handle.finance_account_upsert(&mut new_account);
            {
                if register_result_2.is_err() {
                    return_status_code = StatusCode::BAD_REQUEST;
                    create_result = register_result_2.unwrap_err().to_string()
                } else {
                    create_result = "OK, created".to_string();
                };
            }

            let list_types_result = accounting_config_handle.finance_account_type_list();
            if list_types_result.is_err() {
                return_status_code = StatusCode::BAD_REQUEST;
                create_result = list_types_result.unwrap_err().to_string()
            } else {
                available_types = list_types_result.unwrap();
            };
        }

        let type_position_result = available_types
            .iter()
            .position(|elem| elem.id.eq(&new_account.finance_account_type_id));
        let type_title = match type_position_result {
            Some(position) => &available_types[position].title,
            _ => "Type not found",
        };

        let new_account_template = AccountTemplate {
            id: new_account.id.to_string(),
            name: new_account.title,
            description: new_account.description,
            type_title: type_title.into(),
        };
        let response_html_result = HtmlTemplate(AccountCreateResponseTemplate {
            account: new_account_template,
        })
        .0
        .render();
        let return_html = response_html_result.unwrap();

        let return_value = CreateNewFinanceAccountResponse {
            result: create_result,
            new_id: new_account.id.to_string(),
            subpage: return_html,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (return_status_code, headers, return_value)
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateFinanceAccountFormInput {
    pub account_id: String,
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct UpdateFinanceAccountResponse {
    pub result: String,
}

impl IntoResponse for UpdateFinanceAccountResponse {
    fn into_response(self) -> Response {
        return Json(json!(self)).into_response();
    }
}

pub async fn do_update_finance_account(
    session_data: SessionDataResult,
    Form(input): Form<UpdateFinanceAccountFormInput>,
) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let return_value = UpdateFinanceAccountResponse {
            result: "not logged in".to_string(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return (StatusCode::BAD_REQUEST, headers, return_value);
    }

    if session.is_expired() {
        let return_value = UpdateFinanceAccountResponse {
            result: "Session expired, please try again".to_string(),
        };

        (StatusCode::BAD_REQUEST, headers, return_value)
    } else {
        let upsert_result: String;
        let new_title = &input.title;
        let new_description = &input.description;
        let old_uuid_result = Uuid::parse_str(&input.account_id);
        if old_uuid_result.is_err() {
            debug!(target: "app::FinanceOverView","error in function do_update_finance_account, could not parse UUID from input: {}",&input.account_id);
            let return_value = UpdateFinanceAccountResponse {
                result: "Error reading data".to_string(),
            };

            return (StatusCode::BAD_REQUEST, headers, return_value);
        }

        session.expire_in(std::time::Duration::from_secs(60 * 10));

        let db_handler = DbHandlerMongoDB {};
        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let user_id: Uuid = session.get("user_account_id").unwrap();
        let mut return_status_code = StatusCode::OK;
        {
            let mut accounting_config_handle =
                FinanceAccountingConfigHandle::new(&db_connection, &user_id, &db_handler);

            let available_accounts_result = accounting_config_handle.finance_account_list();
            if available_accounts_result.is_err() {
                debug!(target: "app::FinanceOverView","error in function do_update_finance_account, could not load available accounts for user {}",&user_id);
                let return_value = UpdateFinanceAccountResponse {
                    result: "Error reading database".to_string(),
                };

                return (StatusCode::BAD_REQUEST, headers, return_value);
            }
            let old_uuid = old_uuid_result.unwrap();
            let available_accounts = available_accounts_result.unwrap();
            let position_result = available_accounts
                .iter()
                .position(|elem| elem.id.eq(&old_uuid));
            if position_result.is_none() {
                debug!(target: "app::FinanceOverView","error in function do_update_finance_account, could not load find account {} for user {}",&old_uuid, &user_id);
                let return_value = UpdateFinanceAccountResponse {
                    result: "Error reading database".to_string(),
                };

                return (StatusCode::BAD_REQUEST, headers, return_value);
            }

            let mut old_account_type = FinanceAccount {
                id: old_uuid,
                finance_account_type_id: available_accounts[position_result.unwrap()]
                    .finance_account_type_id,
                title: new_title.into(),
                description: new_description.into(),
            };

            let upsert_result_2 =
                accounting_config_handle.finance_account_upsert(&mut old_account_type);
            {
                if upsert_result_2.is_err() {
                    return_status_code = StatusCode::BAD_REQUEST;
                    upsert_result = upsert_result_2.unwrap_err().to_string()
                } else {
                    upsert_result = "OK, aktualisiert".to_string();
                };
            }
        }

        let return_value = UpdateFinanceAccountResponse {
            result: upsert_result,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (return_status_code, headers, return_value)
    }
}
