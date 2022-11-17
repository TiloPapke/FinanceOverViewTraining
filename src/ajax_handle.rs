use std::{
    env,
    ffi::OsString,
    fs::{read_dir, self},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use async_session::{
    chrono::{DateTime, NaiveDateTime, Utc},
    serde_json::json,
    SessionStore,
};
use axum::{
    body::{self, Full, Empty},
    extract::Form,
    http::{HeaderMap, StatusCode, header, HeaderValue, Uri},
    response::{IntoResponse, Response},
    Json,
};
use log::debug;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::{
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
    let mut js_path = js_uri.path().to_string().trim_start_matches('/').to_string();
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

    let total_js_path = Path::new(&project_root_result.unwrap()).join("js_code").join(js_path);
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
pub struct ChangePasswordRequestResult {
    result: String,
    new_expire_timestamp: String,
}

impl IntoResponse for ChangePasswordRequestResult {
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
        let return_value = ChangePasswordRequestResult {
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
        let return_value = ChangePasswordRequestResult {
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

                    let update_result = password_handle::update_user_password(&credentials_new).await;

                    if update_result.is_err() {
                        change_result = format!("error updating password: {}",update_result.unwrap_err().to_string());
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

        let return_value = ChangePasswordRequestResult {
            result: change_result,
            new_expire_timestamp: session_expire_timestamp,
        };

        let _new_cookie = session_data.session_store.store_session(session).await;

        (headers, return_value)
    }
}
