use async_mongodb_session::MongodbSessionStore;
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
pub struct SessionData {
    pub session_user_id: UserId,
    pub session_option: Option<Session>,
    pub session_store: MongodbSessionStore,
}

impl SessionData {
    pub(crate) fn from_session_data_result(result_obj: SessionDataResult) -> SessionData {
        let session_data = match result_obj {
            SessionDataResult::FoundSessionData(result_obj) => result_obj,
            SessionDataResult::CreatedSessionData(result_obj) => {
                return SessionData {
                    session_user_id: result_obj.session_user_id,
                    session_option: result_obj.session_option,
                    session_store: result_obj.session_store,
                }
            }
        };

        return session_data;
    }
}

pub struct FreshSessionData {
    pub session_user_id: UserId,
    pub session_option: Option<Session>,
    pub session_store: MongodbSessionStore,
    pub cookie: HeaderValue,
}

pub enum SessionDataResult {
    FoundSessionData(SessionData),
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
            return Ok(Self::CreatedSessionData(create_new_session(store).await));
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

            remove_axum_session_cookie(&mut headers);

            let request_uri_path = parts.uri.path();
            if request_uri_path.eq_ignore_ascii_case("/") {
                return Ok(Self::CreatedSessionData(create_new_session(store).await));
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

        Ok(Self::FoundSessionData(SessionData {
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
        cookie: HeaderValue::from_str(format!("{}={}", AXUM_SESSION_COOKIE_NAME, cookie).as_str())
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
