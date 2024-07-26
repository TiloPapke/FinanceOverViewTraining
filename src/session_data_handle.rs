use std::time::Duration;

use async_mongodb_session::MongodbSessionStore;
use async_session::chrono::{DateTime, Utc};
use async_session::{Session, SessionStore};
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::Extension;
use axum::RequestPartsExt;
use axum_extra::{headers::Cookie, TypedHeader};
use log::trace;
use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};

const AXUM_SESSION_COOKIE_NAME: &str = "axum_session";
pub struct SessionDataHandler {
    pub session_user_id: UserId, //hint: this is not an user id of an user account in the final database, this is used to identify themachine client that tries to access the system
    session_option: Option<Session>,
    session_store: MongodbSessionStore,
}

impl SessionDataHandler {
    pub(crate) fn from_session_data_result(result_obj: SessionDataResult) -> Self {
        let session_data = match result_obj {
            SessionDataResult::FoundSessionData(result_obj) => result_obj,
            SessionDataResult::CreatedSessionData(result_obj) => {
                return Self {
                    session_user_id: result_obj.session_user_id,
                    session_option: result_obj.session_option,
                    session_store: result_obj.session_store,
                }
            }
        };

        return session_data;
    }

    pub fn is_logged_in(&self) -> bool {
        if self.session_option.is_none() {
            return false;
        }
        let session = &self.session_option.as_ref().unwrap();
        return session.get("logged_in").unwrap_or(false);
    }

    pub fn set_loggin(&mut self, status: bool) -> Result<(), async_session::serde_json::Error> {
        let session = self.session_option.as_mut().unwrap();
        let return_obj = session.insert("logged_in", status);
        return return_obj;
    }

    pub fn is_expired(&self) -> bool {
        if self.session_option.is_none() {
            return true;
        }
        let session = &self.session_option.as_ref().unwrap();
        return session.is_expired();
    }

    pub fn user_name(&self) -> String {
        if self.session_option.is_none() {
            return "".into();
        }
        let session = &self.session_option.as_ref().unwrap();
        return session.get("user_name").unwrap_or_default();
    }

    pub fn set_user_name(
        &mut self,
        user_name: &String,
    ) -> Result<(), async_session::serde_json::Error> {
        let session = self.session_option.as_mut().unwrap();
        let return_obj = session.insert("user_name", user_name);
        return return_obj;
    }

    pub fn user_id(&self) -> Uuid {
        let session = &self.session_option.as_ref().unwrap();
        return session.get("user_account_id").unwrap();
    }

    pub fn set_user_id(&mut self, user_id: &Uuid) -> Result<(), async_session::serde_json::Error> {
        let session = self.session_option.as_mut().unwrap();
        let return_obj = session.insert("user_account_id", user_id);
        return return_obj;
    }

    pub fn remove_user_id(&mut self) -> () {
        let session = self.session_option.as_mut().unwrap();
        let return_obj = session.remove("user_account_id");
        return return_obj;
    }

    pub fn get_utc_expire_timestamp(&self) -> String {
        let timestamp_value = match &self.session_option.as_ref() {
            Some(session) => session.expiry().unwrap_or(&DateTime::<Utc>::MIN_UTC),
            None => &DateTime::<Utc>::MIN_UTC,
        };

        let session_expire_timestamp = format!(
            "{} UTC",
            (timestamp_value.naive_local().format("%Y-%m-%d %H:%M:%S"))
        );

        return session_expire_timestamp;
    }

    pub fn set_expire(&mut self, expire_in: Option<Duration>) {
        let expire_in_value = match expire_in {
            Some(expire_in_some) => expire_in_some,
            None => std::time::Duration::from_secs(60 * 1),
        };
        let session = self.session_option.as_mut().unwrap();
        session.expire_in(expire_in_value);
    }

    pub async fn update_cookie(&mut self) -> Result<Option<String>, anyhow::Error> {
        let session = self.session_option.as_mut().unwrap().to_owned();

        let new_cookie = self.session_store.store_session(session).await;

        return new_cookie;
    }

    pub async fn destroy_session(&mut self) -> Result<(), anyhow::Error> {
        let session = self.session_option.as_mut().unwrap().to_owned();
        let destroy_result = self.session_store.destroy_session(session).await;
        return destroy_result;
    }

    pub fn valid_logged_in(&self) -> Result<(), String> {
        if self.is_expired() {
            return Err("Session expired".into());
        }
        if !self.is_logged_in() {
            return Err("not logged in".into());
        }
        return Ok(());
    }

    async fn create_new_session(store: MongodbSessionStore) -> FreshSessionData {
        let session_user_id = UserId::new();
        let mut session = Session::new();
        session.insert("session_user_id", session_user_id).unwrap();
        //session.expire_in(std::time::Duration::from_secs(60*5));
        session.expire_in(std::time::Duration::from_secs(60 * 1));
        let cookie = store.store_session(session).await.unwrap().unwrap();
        let cookie_copy = cookie.to_owned();
        let reload_result = store.load_session(cookie_copy).await;
        let reloaded_session = reload_result.unwrap();

        return FreshSessionData {
            session_user_id,
            session_option: reloaded_session,
            session_store: store,
            cookie: HeaderValue::from_str(
                format!("{}={}", AXUM_SESSION_COOKIE_NAME, cookie).as_str(),
            )
            .unwrap(),
        };
    }

    fn remove_axum_session_cookie(headers: &mut HeaderMap) {
        let mut temp_vec: Vec<HeaderValue> = Vec::new();
        let mut current_axum_cookie: String = String::new();
        let all_cookies = headers.get_all(axum::http::header::COOKIE);
        let cookie_iter = all_cookies.iter();
        for cookie_entry in cookie_iter {
            let header_value_text = cookie_entry.to_str().unwrap();
            let check_text = AXUM_SESSION_COOKIE_NAME;
            if header_value_text.starts_with(check_text) {
                current_axum_cookie = format!("{}", header_value_text);
            } else {
                temp_vec.push(cookie_entry.to_owned());
            }
        }
        let _header_value_option_delete = headers.remove(axum::http::header::COOKIE);

        for header_value in temp_vec {
            headers.append(axum::http::header::COOKIE, header_value);
        }
        if current_axum_cookie != "" {
            let reset_axum_cookie = format!(
                "{};expires=Tue, 01-Jan-2000 00:00:01 GMT",
                current_axum_cookie
            );
            let reset_cookie_header = HeaderValue::from_str(&reset_axum_cookie).unwrap();
            headers.insert(http::header::SET_COOKIE, reset_cookie_header);
        }
    }
}

pub struct FreshSessionData {
    pub session_user_id: UserId,
    pub session_option: Option<Session>,
    pub session_store: MongodbSessionStore,
    pub cookie: HeaderValue,
}

pub enum SessionDataResult {
    FoundSessionData(SessionDataHandler),
    CreatedSessionData(FreshSessionData),
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionDataResult
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, HeaderMap, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<MongodbSessionStore>::from_request_parts(parts, state)
            .await
            .expect("`MongoDBSessionStore` extension missing");

        let cookie = parts.extract::<TypedHeader<Cookie>>().await.unwrap();
        let session_cookie = cookie.get(AXUM_SESSION_COOKIE_NAME);

        let mut headers = HeaderMap::from_request_parts(parts, state).await.unwrap();

        // return the new created session cookie for client
        if session_cookie.is_none() {
            return Ok(Self::CreatedSessionData(
                SessionDataHandler::create_new_session(store).await,
            ));
        }

        trace!(
            "UserIdFromSession: got session cookie from user agent, {}={}",
            AXUM_SESSION_COOKIE_NAME,
            session_cookie.unwrap()
        );

        let reloaded_session = store
            .load_session(session_cookie.unwrap().to_owned())
            .await
            .unwrap();

        if reloaded_session.is_none() {
            trace!(
                "SessionDataResult: err session not exists in store, {}={}",
                AXUM_SESSION_COOKIE_NAME,
                session_cookie.unwrap()
            );

            SessionDataHandler::remove_axum_session_cookie(&mut headers);

            let request_uri_path = parts.uri.path();
            if request_uri_path.eq_ignore_ascii_case("/") {
                return Ok(Self::CreatedSessionData(
                    SessionDataHandler::create_new_session(store).await,
                ));
            } else {
                headers.insert(
                    http::header::REFRESH,
                    HeaderValue::from_str("5; url = /").unwrap(),
                );
                return Err((
                    StatusCode::BAD_REQUEST,
                    headers,
                    "No session found for cookie",
                ));
            }
        }

        // continue to decode the session cookie
        let session_user_id = if let Some(user_id) = reloaded_session
            .clone()
            .unwrap()
            .get::<UserId>("session_user_id")
        {
            trace!(
                "SessionUserIdFromSession: session decoded success, user_id={:?}",
                user_id
            );
            user_id
        } else {
            headers.insert(
                http::header::REFRESH,
                HeaderValue::from_str("5; url = /").unwrap(),
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                "No `session_user_id` found in session",
            ));
        };

        Ok(Self::FoundSessionData(SessionDataHandler {
            session_user_id,
            session_option: reloaded_session,
            session_store: store,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct UserId(Uuid);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl UserId {
    fn new() -> Self {
        Self(Uuid::new())
    }
}
