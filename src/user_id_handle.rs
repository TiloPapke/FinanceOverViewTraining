use async_mongodb_session::MongodbSessionStore;
use async_session::{Session, SessionStore};
use axum::TypedHeader;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    headers::Cookie,   
};
use log::trace;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const AXUM_SESSION_COOKIE_NAME: &str = "axum_session";
pub struct FreshUserId {
    pub user_id: UserId,
    pub cookie: HeaderValue,
}

pub(crate) enum UserIdFromSession {
    FoundUserId(UserId),
    CreatedFreshUserId(FreshUserId),
}

#[async_trait]
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<MongodbSessionStore>::from_request(req)
            .await
            .expect("`MongoDBSessionsStore` extension missing");

        let cookie = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap();

        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(AXUM_SESSION_COOKIE_NAME));

        // return the new created session cookie for client
        if session_cookie.is_none() {
            let user_id = UserId::new();
            let mut session = Session::new();
            session.insert("user_id", user_id).unwrap();
            session.expire_in(std::time::Duration::from_secs(60*5));
            session.insert("testvar1","Testvalue1").unwrap();
            session.insert("testvar2","Testvalue2").unwrap();
            let cookie = store.store_session(session).await.unwrap().unwrap();
            let mut session2 = store.load_session(cookie.clone()).await.unwrap().unwrap();
            session2.insert("testvar3","Testvalue3").unwrap();
            let cookie2 = store.store_session(session2).await;
            return Ok(Self::CreatedFreshUserId(FreshUserId {
                user_id,
                cookie: HeaderValue::from_str(
                    format!("{}={}", AXUM_SESSION_COOKIE_NAME, cookie).as_str(),
                )
                .unwrap(),
            }));
        }

    
        trace!(
            "UserIdFromSession: got session cookie from user agent, {}={}",
            AXUM_SESSION_COOKIE_NAME,
            session_cookie.unwrap()
        );
        // continue to decode the session cookie
        let user_id = if let Some(session) = store
            .load_session(session_cookie.unwrap().to_owned())
            .await
            .unwrap()
        {
            if let Some(user_id) = session.get::<UserId>("user_id") {
                trace!(
                    "UserIdFromSession: session decoded success, user_id={:?}",
                    user_id
                );
                user_id
            } else {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "No `user_id` found in session",
                ));
            }
        } else {
            trace!(
                "UserIdFromSession: err session not exists in store, {}={}",
                AXUM_SESSION_COOKIE_NAME,
                session_cookie.unwrap()
            );
            return Err((StatusCode::BAD_REQUEST, "No session found for cookie"));
        };

        Ok(Self::FoundUserId(user_id))
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
        Self(Uuid::new_v4())
    }

}