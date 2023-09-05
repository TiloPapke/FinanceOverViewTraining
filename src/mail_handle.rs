use mail_send::mail_builder::MessageBuilder;
use regex_automata::meta::Regex;

use crate::setting_struct::SettingStruct;

pub(crate) struct SimpleMailData {
    pub receiver: String,
    pub sender: String,
    pub subject: String,
    pub body: String,
}

pub(crate) struct SmtpMailSetting {
    pub host: String,
    pub client_name: String,
    pub client_password: String,
}

pub(crate) async fn send_smtp_mail(
    mail_content: SimpleMailData,
    mail_server_setting: SmtpMailSetting,
) -> Result<(), String> {
    // Build a simple multipart message
    let message = MessageBuilder::new()
        .from((mail_content.sender.clone(), mail_content.sender))
        .to(vec![(mail_content.receiver.clone(), mail_content.receiver)])
        .subject(mail_content.subject)
        .text_body(mail_content.body);

    // Connect to an SMTP relay server over TLS and
    // authenticate using the provided credentials.
    let smpt_connect_result = mail_send::SmtpClientBuilder::new(mail_server_setting.host, 587)
        .implicit_tls(false)
        .credentials((mail_server_setting.client_name,mail_server_setting.client_password))
        .connect()
        .await;

    if smpt_connect_result.is_err() {
        Err("Could not connect to mail server".to_string())
    } else {
        let send_result = smpt_connect_result.unwrap().send(message).await;

        if send_result.is_ok() {
            Ok(())
        } else {
            Err(format!("Could not send email: {:?}", send_result))
        }
    }
}

pub(crate) fn validate_email(email_address_to_check: &String) -> Result<bool, String> {
    let local_setting = SettingStruct::global();

    let regexfile = &local_setting.frontend_register_user_mail_validation_regex_path;

    let file = std::fs::File::open(regexfile);
    if file.is_err() {
        Err(format!(
            "Error accessing file with email regex: {}",
            file.unwrap_err()
        ))
    } else {
        let mut regex_contents = String::new();
        let read_result = std::io::Read::read_to_string(&mut file.unwrap(), &mut regex_contents);
        if read_result.is_err() {
            Err(format!(
                "Error readin file with email regex: {}",
                read_result.unwrap_err()
            ))
        } else {
            let regex_obj_build = Regex::new(&regex_contents);
            if regex_obj_build.is_err() {
                Err(format!(
                    "Error parsing regex rule for email validation: {}",
                    regex_obj_build.unwrap_err()
                ))
            } else {
                let regex_obj = regex_obj_build.unwrap();
                let check_result = regex_obj.is_match(email_address_to_check.as_bytes());

                return Ok(check_result);
            }
        }
    }
}
