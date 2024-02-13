use std::borrow::Borrow;

use anyhow::Error;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use bson::uuid;
use secrecy::{ExposeSecret, Secret};

use crate::database_handler_mongodb::EmailVerificationStatus;
use crate::datatypes::{
    PasswordResetRequest, PasswordResetTokenRequest, PasswordResetTokenRequestResult,
};
use crate::{
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
    setting_struct::SettingStruct,
};

pub struct UserCredentials {
    // These two fields were not marked as `pub` before!
    pub username: String,
    pub password: Secret<String>,
}

pub struct UserCredentialsHashed {
    // These two fields were not marked as `pub` before!
    pub username: String,
    pub password_hash: Secret<String>,
}

#[derive(Debug)]
pub struct StoredCredentials {
    // These two fields were not marked as `pub` before!
    pub user_id: uuid::Uuid,
    pub password_hash: Secret<String>,
}

pub async fn validate_credentials(credentials: &UserCredentials) -> Result<uuid::Uuid, Error> {
    let get_result = get_stored_credentials(&credentials.username).await;
    if get_result.is_err() {
        return Err(anyhow::anyhow!("Problem getting credentials"));
    }

    let stored_credentials = get_result.unwrap();
    let user_id = Some(stored_credentials.user_id);

    let expected_password_hash = stored_credentials.password_hash;

    let verify_result = verify_password_hash(&expected_password_hash, &credentials.password);
    if verify_result.is_err() {
        return Err(verify_result.unwrap_err());
    }

    if user_id.is_some() {
        return Ok(user_id.unwrap());
    }
    Err(anyhow::anyhow!("Unknown username."))
}

async fn get_stored_credentials(_user_name: &str) -> Result<StoredCredentials, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let _db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let query_credentials =
        DbHandlerMongoDB::get_stored_credentials_by_name(&_db_connection, &_user_name.to_string())
            .await;

    if query_credentials.is_err() {
        return Err(anyhow::anyhow!(query_credentials.unwrap_err()));
    }

    let some_credential = query_credentials.unwrap();

    Ok(some_credential)
}

pub(crate) fn verify_password_hash(
    expected_password_hash: &Secret<String>,
    password_candidate: &Secret<String>,
) -> Result<(), Error> {
    let expected_password_hash_2 = PasswordHash::new(&expected_password_hash.expose_secret());
    if expected_password_hash_2.is_err() {
        return Err(anyhow::anyhow!(
            "Failed to parse hash in PHC string format."
        ));
    }

    let check_result = Argon2::default().verify_password(
        password_candidate.expose_secret().as_bytes(),
        &expected_password_hash_2.unwrap(),
    );
    if check_result.is_err() {
        return Err(anyhow::anyhow!("AUTH ERROR: {}", check_result.unwrap_err()));
    }

    Ok(())
}

pub fn compare_password(
    password_1: &Secret<String>,
    password_2: &Secret<String>,
) -> Result<(), Error> {
    if password_1.expose_secret() != password_2.expose_secret() {
        return Err(anyhow::anyhow!("new passwords do not match"));
    }

    Ok(())
}

pub async fn create_credentials(credentials: &UserCredentials) -> Result<uuid::Uuid, Error> {
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

pub(crate) async fn check_user_exsits(user_name: &str) -> Result<bool, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let check_result =
        DbHandlerMongoDB::check_user_exsists_by_name(&db_connection, &user_name.to_string()).await;

    if check_result.is_err() {
        return Err(anyhow::anyhow!(check_result.unwrap_err()));
    }

    Ok(check_result.unwrap())
}

pub(crate) async fn insert_user(some_credentials: &UserCredentials) -> Result<uuid::Uuid, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let salt = SaltString::generate(&mut rand::thread_rng());
    let user_password_hashed = Argon2::default()
        .hash_password(some_credentials.password.expose_secret().as_bytes(), &salt)
        .unwrap()
        .to_string();

    let some_credentials_hashed = UserCredentialsHashed {
        username: some_credentials.username.clone(),
        password_hash: Secret::new(user_password_hashed),
    };

    let insert_result =
        DbHandlerMongoDB::insert_user(&db_connection, &some_credentials_hashed).await;

    if insert_result.is_err() {
        return Err(anyhow::anyhow!(insert_result.unwrap_err()));
    }

    Ok(insert_result.unwrap())
}

pub async fn update_user_password(some_credentials: &UserCredentials) -> Result<bool, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let salt = SaltString::generate(&mut rand::thread_rng());
    let user_password_hashed = Argon2::default()
        .hash_password(some_credentials.password.expose_secret().as_bytes(), &salt)
        .unwrap()
        .to_string();

    let some_credentials_hashed: UserCredentialsHashed = UserCredentialsHashed {
        username: some_credentials.username.clone(),
        password_hash: Secret::new(user_password_hashed),
    };

    let update_result =
        DbHandlerMongoDB::update_user_password(&db_connection, &some_credentials_hashed).await;

    if update_result.is_err() {
        return Err(anyhow::anyhow!(update_result.unwrap_err()));
    }

    Ok(update_result.unwrap())
}

pub async fn check_email_status_by_name(user_name: &str) -> Result<EmailVerificationStatus, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let check_result =
        DbHandlerMongoDB::check_email_verfification_by_name(&db_connection, &user_name.to_string())
            .await;

    if check_result.is_err() {
        return Err(anyhow::anyhow!(check_result.unwrap_err()));
    }
    return Ok(check_result.unwrap());
}

pub async fn request_password_reset_token(
    request_data: &PasswordResetTokenRequest,
) -> Result<PasswordResetTokenRequestResult, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let generate_token_result_async = DbHandlerMongoDB::generate_passwort_reset_token(
        &db_connection,
        request_data.user_name.borrow(),
        request_data.reset_secret.borrow(),
        &local_setting.frontend_password_reset_token_time_limit_minutes,
    );

    let generate_token_result = generate_token_result_async.await;

    if generate_token_result.is_err() {
        return Err(anyhow::anyhow!(
            "Error get data: {}",
            generate_token_result.unwrap_err()
        ));
    }

    return Ok(generate_token_result.unwrap());
}

pub async fn reset_password_with_token(request_data: &PasswordResetRequest) -> Result<bool, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let salt = SaltString::generate(&mut rand::thread_rng());
    let user_password_hashed = Argon2::default()
        .hash_password(request_data.new_password.expose_secret().as_bytes(), &salt)
        .unwrap()
        .to_string();

    let passwort_reset_result = DbHandlerMongoDB::change_password_with_token(
        &db_connection,
        &request_data.username,
        &request_data.reset_token,
        &Secret::new(user_password_hashed),
    )
    .await;

    if passwort_reset_result.is_err() {
        return Err(anyhow::anyhow!(
            "Error get data: {}",
            passwort_reset_result.unwrap_err()
        ));
    }

    return Ok(passwort_reset_result.unwrap());
}
