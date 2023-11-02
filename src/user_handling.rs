use secrecy::Secret;

use crate::{database_handler_mongodb::{DbHandlerMongoDB, EmailVerificationStatus, DbConnectionSetting}, setting_struct::SettingStruct};

pub(crate) fn validate_user_email(user_name:&String, email_secret: &Secret<String>) -> Result<EmailVerificationStatus, String> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let _validate_result = DbHandlerMongoDB::verify_email_by_name(&db_connection,user_name,email_secret);

    return Err("Not implemented yet".to_string());
}