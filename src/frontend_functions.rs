use std::borrow::Borrow;

use anyhow::{Error, Ok};
use log::error;
use secrecy::Secret;

use crate::{
    convert_tools::ConvertTools,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
    datatypes::{GenerallUserData, PasswordResetTokenRequestResult},
    mail_handle::{self, validate_email_format, SimpleMailData, SmtpMailSetting},
    setting_struct::SettingStruct,
};

pub async fn register_user_with_email_verfication(
    user_name: &String,
    user_password: &Secret<String>,
    user_email: &String,
) -> Result<String, Error> {
    let check_mail_result = validate_email_format(user_email);
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

    let reg_subject = local_setting
        .frontend_register_user_mail_info_subject
        .replace("{{username}}", &user_name);
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
            "Error reading email registration template: {}",
            reg_body_read_result.unwrap_err()
        ));
    }

    let validation_token_masked = ConvertTools::escape_htmltext(validation_token);
    let reg_body = reg_body_read_result
        .unwrap()
        .replace("{{username}}", &user_name)
        .replace(
            "{{serveraddress}}",
            &local_setting.frontend_register_user_mail_server_address,
        )
        .replace("{{hashedToken}}", &validation_token_masked);
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

    let result: Result<(), String> = result_async.await;

    if result.is_err() {
        return Err(anyhow::anyhow!(
            "Error sending registration mail: {}",
            result.unwrap_err()
        ));
    }

    return Ok(true);
}

pub async fn get_general_userdata_fromdatabase(
    user_name: &String,
) -> Result<GenerallUserData, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let get_result_async =
        DbHandlerMongoDB::get_user_general_data_by_user_name(&db_connection, user_name);

    let get_result: Result<GenerallUserData, String> = get_result_async.await;

    if get_result.is_err() {
        return Err(anyhow::anyhow!(
            "Error get data: {}",
            get_result.unwrap_err()
        ));
    }

    return Ok(get_result.unwrap());
}

pub async fn save_general_userdata(
    user_name: &String,
    general_user_data: &GenerallUserData,
) -> Result<String, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };

    let save_data_result_async = DbHandlerMongoDB::update_general_user_data_by_name(
        &db_connection,
        user_name,
        general_user_data,
    );
    let save_data_result = save_data_result_async.await;
    if save_data_result.is_err() {
        return Err(anyhow::anyhow!(
            "Error while saving data {}",
            save_data_result.unwrap_err()
        ));
    }

    return Ok(save_data_result.unwrap());
}

pub async fn send_password_reset_email(
    user_name: &String,
    password_reset_token: &PasswordResetTokenRequestResult,
) -> Result<bool, Error> {
    let local_setting: SettingStruct = SettingStruct::global().clone();

    let password_reset_subject = local_setting
        .frontend_password_reset_mail_info_subject
        .replace("{{username}}", &user_name);
    let working_dir = std::env::current_dir().unwrap();
    let password_reset_body_template_file = std::path::Path::new(&working_dir)
        .join(local_setting.frontend_password_reset_mail_info_body_path);

    if !password_reset_body_template_file.exists() {
        error!(target: "app::FinanceOverView","email template for password reset not found");
        return Err(anyhow::anyhow!(
            "email template for password reset not found"
        ));
    }
    let password_reset_body_read_result =
        crate::convert_tools::ConvertTools::load_text_from_file(&password_reset_body_template_file);
    if password_reset_body_read_result.is_err() {
        return Err(anyhow::anyhow!(
            "Error reading email password reset template: {}",
            password_reset_body_read_result.unwrap_err()
        ));
    }

    let reset_token_masked =
        ConvertTools::escape_htmltext(password_reset_token.reset_token.borrow());
    let password_reset_body = password_reset_body_read_result
        .unwrap()
        .replace("{{username}}", &user_name)
        .replace(
            "{{serveraddress}}",
            &local_setting.frontend_password_reset_mail_server_address,
        )
        .replace("{{resettoken}}", &reset_token_masked)
        .replace(
            "{{timelimit_minutes}}",
            &local_setting
                .frontend_password_reset_token_time_limit_minutes
                .to_string(),
        )
        .replace(
            "{{tokenexpriredatetime}}",
            &password_reset_token.expires_at.to_string(),
        );
    //

    let mail_content = SimpleMailData {
        receiver: password_reset_token.user_email.clone(),
        sender: local_setting.backend_mail_smtp_mail_address,
        subject: password_reset_subject,
        body: password_reset_body,
    };

    let mail_config = SmtpMailSetting {
        host: local_setting.backend_mail_smtp_host,
        client_name: local_setting.backend_mail_smtp_user,
        client_password: local_setting.backend_mail_smtp_password,
    };

    let result_async = mail_handle::send_smtp_mail(mail_content, mail_config);

    let result: Result<(), String> = result_async.await;

    if result.is_err() {
        return Err(anyhow::anyhow!(
            "Error sending password reset mail: {}",
            result.unwrap_err()
        ));
    }

    return Ok(true);
}
