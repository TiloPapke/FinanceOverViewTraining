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
pub struct SessionData {
    pub user_id: UserId,
    pub session_option:Option<Session>,
    pub session_store:MongodbSessionStore,
}

impl SessionData {
    pub(crate)  fn from_session_data_result(result_obj:SessionDataResult)->SessionData{
        let session_data = match result_obj {
            SessionDataResult::FoundSessionData(result_obj) => ( result_obj),
            SessionDataResult::CreatedSessionData(result_obj) =>
                return SessionData { user_id: result_obj.user_id,
                              session_option: result_obj.session_option,
                              session_store: result_obj.session_store }
                
            ,
        };

        return session_data;
    }
}

pub struct FreshSessionData {
    pub user_id: UserId,
    pub session_option:Option<Session>,
    pub session_store:MongodbSessionStore,
    pub cookie: HeaderValue,
}

pub enum SessionDataResult {
    FoundSessionData(SessionData),
    CreatedSessionData(FreshSessionData),
}

#[async_trait]
impl<B> FromRequest<B> for SessionDataResult
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
            let cookie = store.store_session(session).await.unwrap().unwrap();
            let cookie_copy = cookie.to_owned();
            let reload_result = store.load_session(cookie_copy)
                                                                    .await;
            let reloaded_session=reload_result.unwrap();

            return Ok(Self::CreatedSessionData(FreshSessionData {
                user_id,
                session_option: reloaded_session,
                session_store: store,
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

        let reloaded_session  = store
            .load_session(session_cookie.unwrap().to_owned())
            .await
            .unwrap();
        
        if reloaded_session.is_none(){
            trace!(
                "SessionDataResult: err session not exists in store, {}={}",
                AXUM_SESSION_COOKIE_NAME,
                session_cookie.unwrap()
            );
            return Err((StatusCode::BAD_REQUEST, "No session found for cookie"));
        }
        
        // continue to decode the session cookie
            let user_id = 
                if let Some(user_id) = reloaded_session.clone().unwrap().get::<UserId>("user_id") {
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
            };


        Ok(Self::FoundSessionData(SessionData{
            user_id,
            session_option: reloaded_session,
            session_store: store
        }
        ))
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