use anyhow::Error;

use crate::password_handle::{check_user_exsits, insert_user, UserCredentials};

pub async fn register_user_with_email_verfication(
    credentials: &UserCredentials,
    user_email: &String,
) -> Result<uuid::Uuid, Error> {
    let check_result = check_user_exsits(&credentials.username).await;
    if check_result.is_err() {
        return Err(check_result.unwrap_err());
    }

    let user_exsists = check_result.unwrap();
    if user_exsists {
        return Err(anyhow::anyhow!("User already exsists, can not recreate"));
    }

    let insert_result = insert_user(&credentials).await;
    if insert_result.is_err() {
        return Err(insert_result.unwrap_err());
    }

    return Ok(insert_result.unwrap());
}
