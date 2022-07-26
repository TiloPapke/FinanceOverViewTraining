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
/*
pub struct FreshUserId {
    pub user_id: UserId,
    pub cookie: HeaderValue,
}
*/
pub enum MdbSessionStoreLoadResult {
    FoundSessionStore(MongodbSessionStore),
}

#[async_trait]
impl<B> FromRequest<B> for MdbSessionStoreLoadResult
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<MongodbSessionStore>::from_request(req)
            .await
            .expect("`MongoDBSessionsStore` extension missing");


        Ok(Self::FoundSessionStore(store))
    }
}

//#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
//pub struct UserId(Uuid);

/*
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
*/