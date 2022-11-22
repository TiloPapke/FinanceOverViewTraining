use mail_send::{mail_builder::MessageBuilder, Transport};

pub struct SimpleMailData {
    pub receiver: String,
    pub sender: String,
    pub subject: String,
    pub body: String,
}

pub struct SmtpMailSetting {
    pub host: String,
    pub client_name: String,
    pub client_password: String,
}

pub async fn send_smtp_mail(
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
    let transport_create = Transport::new(mail_server_setting.host)
        .credentials(
            mail_server_setting.client_name,
            mail_server_setting.client_password,
        )
        
        .connect_tls()
        .await;

    if transport_create.is_err() {
        Err("Could not connect to mail server".to_string()
        )
    } else {
        let send_result = transport_create.unwrap().send(message).await;

        if send_result.is_ok() {
            Ok(())
        } else {
            Err(format!("Could not send email: {:?}", send_result))
        }
    }
}
