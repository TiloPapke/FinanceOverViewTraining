use async_mongodb_session::MongodbSessionStore;
use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},  
};

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
