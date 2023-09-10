use anyhow::Error;
use log::error;
use secrecy::Secret;

use crate::{
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
    mail_handle::{self, validate_email, SimpleMailData, SmtpMailSetting},
    setting_struct::SettingStruct,
};

pub async fn register_user_with_email_verfication(
    user_name: &String,
    user_password: &Secret<String>,
    user_email: &String,
) -> Result<String, Error> {
    let check_mail_result = validate_email(user_email);
    if check_mail_result.is_err() {
        return Err(anyhow::anyhow!(
            "Error while validating email {}: {}",
            user_email,
            check_mail_result.unwrap_err()
        ));
    }
    if !check_mail_result.unwrap() {
        return Err(anyhow::anyhow!("email {} is not valid", user_email));
    }

    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let new_user_credentials = crate::password_handle::UserCredentials {
        username: user_name.to_string(),
        password: user_password.clone(),
    };

    //create_credentials checks if user is already there
    let create_result = crate::password_handle::create_credentials(&new_user_credentials).await;
    if create_result.is_err() {
        return Err(anyhow::anyhow!(
            "error creating user: {}",
            create_result.unwrap_err()
        ));
    }

    let update_result =
        DbHandlerMongoDB::update_user_email(&db_connection, &user_name, user_email).await;
    if update_result.is_err() {
        return Err(anyhow::anyhow!(
            "error setting email: {}",
            update_result.unwrap_err()
        ));
    }

    let send_email_result =
        send_email_verification_mail(user_name, user_email, &update_result.as_ref().unwrap()).await;

    if send_email_result.is_err() {
        return Err(anyhow::anyhow!(
            "error sending verifiation email: {}",
            send_email_result.unwrap_err()
        ));
    }

    return Ok(update_result.unwrap());
}

async fn send_email_verification_mail(
    user_name: &String,
    user_email: &String,
    validation_token: &String,
) -> Result<bool, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();

    let reg_subject = local_setting.frontend_register_user_mail_info_subject;
    let working_dir = std::env::current_dir().unwrap();
    let reg_body_template_file = std::path::Path::new(&working_dir)
        .join(local_setting.frontend_register_user_mail_info_body_path);

    if !reg_body_template_file.exists() {
        error!(target: "app::FinanceOverView","email template for registration not found");
        return Err(anyhow::anyhow!("email template for registration not found"));
    }
    let reg_body_read_result =
        crate::convert_tools::ConvertTools::load_text_from_file(&reg_body_template_file);
    if reg_body_read_result.is_err() {
        return Err(anyhow::anyhow!(
            "Error readin email registration template: {}",
            reg_body_read_result.unwrap_err()
        ));
    }

    let reg_body = reg_body_read_result
        .unwrap()
        .replace("{{username}}", &user_name)
        .replace(
            "{{serveraddress}}",
            &local_setting.frontend_register_user_mail_server_address,
        )
        .replace("{{hashedToken}}", validation_token);
    //

    let mail_content = SimpleMailData {
        receiver: user_email.clone(),
        sender: local_setting.backend_mail_smtp_mail_address,
        subject: reg_subject,
        body: reg_body,
    };

    let mail_config = SmtpMailSetting {
        host: local_setting.backend_mail_smtp_host,
        client_name: local_setting.backend_mail_smtp_user,
        client_password: local_setting.backend_mail_smtp_password,
    };

    let result_async = mail_handle::send_smtp_mail(mail_content, mail_config);

    let result = result_async.await;

    if result.is_err() {
        return Err(anyhow::anyhow!(
            "Error sending registration mail: {}",
            result.unwrap_err()
        ));
    }

    return Ok(true);
}
