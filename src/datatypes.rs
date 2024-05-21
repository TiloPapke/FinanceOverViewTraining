use async_session::chrono::{DateTime, Utc};
use mongodb::bson::Uuid;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GenerallUserData {
    pub first_name: String,
    pub last_name: String,
    pub reset_secret: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordResetTokenRequest {
    pub user_name: String,
    pub reset_secret: String,
}

#[derive(Debug)]
pub struct PasswordResetTokenRequestResult {
    pub reset_token: String,
    pub expires_at: DateTime<Utc>,
    pub user_email: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordResetRequest {
    pub username: String,
    pub reset_token: String,
    pub new_password: secrecy::Secret<String>,
}

#[derive(Deserialize, Debug)]
pub struct FinanceAccountType {
    pub id: Uuid,
    pub title: String,
    pub descriptiom: String,
}
