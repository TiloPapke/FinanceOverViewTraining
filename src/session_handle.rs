use async_mongodb_session::MongodbSessionStore;
use async_session::{Session, SessionStore};
use axum::TypedHeader;
use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    headers::Cookie,   
};
use log::trace;

const AXUM_SESSION_COOKIE_NAME: &str = "axum_session";

pub enum SessionRefOptionFromStore {
    FoundSession(Option<Session>),
}

#[async_trait]
impl<B> FromRequest<B> for SessionRefOptionFromStore
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
            trace!(
                "no Session found for cookie {}",AXUM_SESSION_COOKIE_NAME
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "No session found",
            ));
        }

    
        trace!(
            "Session: got session cookie from user agent, {}={}",
            AXUM_SESSION_COOKIE_NAME,
            session_cookie.unwrap()
        );
        // continue to decode the session cookie
        let local_session = store
            .load_session(session_cookie.unwrap().to_owned())
            .await
            .unwrap();

        Ok(Self::FoundSession(local_session))
    }
}
