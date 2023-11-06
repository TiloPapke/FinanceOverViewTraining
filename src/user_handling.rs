use secrecy::Secret;

use crate::{
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB, EmailVerificationStatus},
    setting_struct::SettingStruct,
};

pub(crate) async fn validate_user_email(
    user_name: &String,
    email_secret: &Secret<String>,
) -> Result<EmailVerificationStatus, String> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let validate_result =
        DbHandlerMongoDB::verify_email_by_name(&db_connection, user_name, email_secret).await;
    if validate_result.is_err() {
        return Err(format!(
            "Error during verfiy_email_by_name: {}",
            validate_result.unwrap_err()
        ));
    } else {
        return Ok(validate_result.unwrap());
    }
}
