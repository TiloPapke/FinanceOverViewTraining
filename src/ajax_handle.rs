use std::{
    borrow::Borrow,
    env,
    ffi::OsString,
    fs::{self, read_dir},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use async_session::{
    chrono::{DateTime, NaiveDateTime, Utc},
    serde_json::json,
    SessionStore,
};
use axum::{
    body::{self, Empty, Full},
    extract::Form,
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use log::debug;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::{
    datatypes::{PasswordResetRequest, PasswordResetTokenRequest},
    frontend_functions::send_password_reset_email,
    password_handle::{self, validate_credentials, UserCredentials},
    session_data_handle::{SessionData, SessionDataResult},
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
            .body(body::boxed(body::Full::from(
                project_root_result.unwrap_err().to_string(),
            )))
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
            .body(body::boxed(Full::from(file_content)))
            .unwrap();
    } else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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

            match validate_credentials(&credentials).await {
                Ok(user_id) => {
                    debug!(target: "app::FinanceOverView","trying to change password for user {}", user_id);

                    let credentials_new = UserCredentials {
                        username: username.clone(),
                        password: password_new_1.clone(),
                    };

                    let update_result =
                        password_handle::update_user_password(&credentials_new).await;

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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );

        let register_result_2 = crate::frontend_functions::register_user_with_email_verfication(
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );

        let password_reset_request_result =
            crate::password_handle::request_password_reset_token(input.borrow()).await;

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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
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
                .unwrap_or(&DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(0, 0),
                    Utc
                ))
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );

        let password_change_result =
            crate::password_handle::reset_password_with_token(input.borrow()).await;

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
