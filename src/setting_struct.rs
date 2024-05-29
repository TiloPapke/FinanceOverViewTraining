use std::path::PathBuf;

use ini::Ini;
use once_cell::sync::OnceCell;

#[derive(Clone)]
pub struct SettingStruct {
    pub web_server_ip_part1: u8,
    pub web_server_ip_part2: u8,
    pub web_server_ip_part3: u8,
    pub web_server_ip_part4: u8,
    pub web_server_port_http: u16,
    pub web_server_port_https: u16,
    pub web_server_cert_cert_path: String,
    pub web_server_cert_key_path: String,
    pub backend_database_url: String,
    pub backend_database_user: String,
    pub backend_database_password: String,
    pub backend_database_instance: String,
    pub log_config_path: String,
    pub backend_mail_smtp_host: String,
    pub backend_mail_smtp_user: String,
    pub backend_mail_smtp_password: String,
    pub backend_mail_smtp_mail_address: String,
    pub frontend_register_user_mail_validation_regex_path: String,
    pub frontend_register_user_mail_info_subject: String,
    pub frontend_register_user_mail_info_body_path: String,
    pub frontend_register_user_mail_server_address: String,
    pub frontend_password_reset_mail_validation_regex_path: String,
    pub frontend_password_reset_mail_info_subject: String,
    pub frontend_password_reset_mail_info_body_path: String,
    pub frontend_password_reset_mail_server_address: String,
    pub frontend_password_reset_token_time_limit_minutes: u16,
}

#[derive(Clone)]
pub struct TestSettingStruct {
    pub outgoing_mail_receiver: String,
    pub outgoing_mail_title: String,
    pub outgoing_mail_simple_body: String,
    pub test_database_db_url: String,
    pub test_database_db_user: String,
    pub test_database_db_password: String,
    pub test_database_db_instance: String,
}

pub static GLOBAL_SETTING: OnceCell<SettingStruct> = OnceCell::new();

pub static GLOBAL_TEST_SETTING: OnceCell<TestSettingStruct> = OnceCell::new();

impl SettingStruct {
    pub fn global() -> &'static SettingStruct {
        GLOBAL_SETTING
            .get()
            .expect("GLOBAL_SETTING is not initialized")
    }

    pub fn create_dummy_setting(settingpath: &PathBuf) {
        let mut conf: Ini = Ini::new();
        conf.with_section(Some("[WARNING]"))
            .set("GITWARNING", "This is a default config file, when entering own value be sure to add this file to ignore")
            .set("GITWARNING2", "Only push to repository if this file does not contain any private information")
            .set("Renaming", "Use this file as tempalte to create your own ServerSettings.ini file");
        conf.with_section(Some("WebServer"))
            .set("ip_part1", "127")
            .set("ip_part2", "0")
            .set("ip_part3", "0")
            .set("ip_part4", "1")
            .set("port_http", "3000")
            .set("port_https", "3300")
            .set("cert_cert_path", "config/self-signed-certs/cert.pem")
            .set("cert_key_path", "config/self-signed-certs/key.pem");
        conf.with_section(Some("BackendDatabase"))
            .set("DB_URL", "mongodb://localhost:27017")
            .set("DB_User", "Administrator")
            .set("DB_Password", "password")
            .set("DB_Instance", "StructureName");
        conf.with_section(Some("Logging"))
            .set("config_path", "config/default_log_settings.yaml");
        conf.with_section(Some("BackendMail"))
            .set("Mail_SMTP_Host", "smtp.server.org")
            .set("Mail_SMTP_User", "username")
            .set("Mail_SMTP_Password", "secret")
            .set("Mail_SMTP_MailAddress", "username@server.org");
        conf.with_section(Some("Frontend_RegisterUser"))
            .set(
                "RegisterUser_Mail_Validation_Regex_Path",
                "config/default_email_validation_regex.info",
            )
            .set(
                "RegisterUser_Mail_Info_Subject",
                "Welcome to the FinanceTrainingPortal {{username}}",
            )
            .set(
                "RegisterUser_Mail_Info_Body_Path",
                "config/default_RegisterMailBody.html",
            )
            .set("RegisterUser_Mail_Server_Address", "https:127.0.0.1:3300");
        conf.with_section(Some("Frontend_PasswordReset"))
            .set(
                "PasswordReset_Mail_Validation_Regex_Path",
                "config/default_email_validation_regex.info",
            )
            .set(
                "PasswordReset_Mail_Info_Subject",
                "A password Reset was requested for FinanceTrainingPortal {{username}}",
            )
            .set(
                "PasswordReset_Mail_Info_Body_Path",
                "config/default_ResetPasswordBody.html",
            )
            .set("PasswordReset_Mail_Server_Address", "https:127.0.0.1:3300")
            .set("PasswordReset_Token_Time_Limit_Minutes", "5");
        conf.write_to_file(&settingpath).unwrap();
    }

    pub fn load_from_file(settingpath: &PathBuf) -> Self {
        let conf: Ini = Ini::load_from_file(&settingpath).unwrap();
        let _web_server_ip_part1: u8 = conf
            .get_from_or(Some("WebServer"), "ip_part1", "127")
            .parse()
            .unwrap();
        let _web_server_ip_part2: u8 = conf
            .get_from_or(Some("WebServer"), "ip_part2", "0")
            .parse()
            .unwrap();
        let _web_server_ip_part3: u8 = conf
            .get_from_or(Some("WebServer"), "ip_part3", "0")
            .parse()
            .unwrap();
        let _web_server_ip_part4: u8 = conf
            .get_from_or(Some("WebServer"), "ip_part4", "1")
            .parse()
            .unwrap();
        let _web_server_port_http: u16 = conf
            .get_from_or(Some("WebServer"), "port_http", "3000")
            .parse()
            .unwrap();
        let _web_server_port_https: u16 = conf
            .get_from_or(Some("WebServer"), "port_https", "3300")
            .parse()
            .unwrap();
        let _web_server_cert_cert_path: String = conf
            .get_from_or(Some("WebServer"), "cert_cert_path", "")
            .to_string();
        let _web_server_cert_key_path: String = conf
            .get_from_or(Some("WebServer"), "cert_key_path", "")
            .to_string();
        let _db_url: String = conf
            .get_from_or(Some("BackendDatabase"), "DB_URL", "")
            .to_string();
        let _db_user: String = conf
            .get_from_or(Some("BackendDatabase"), "DB_User", "")
            .to_string();
        let _db_password: String = conf
            .get_from_or(Some("BackendDatabase"), "DB_Password", "")
            .to_string();
        let _db_instance: String = conf
            .get_from_or(Some("BackendDatabase"), "DB_Instance", "")
            .to_string();
        let _log_config_path: String = conf
            .get_from_or(
                Some("Logging"),
                "file_config_path",
                "config/default_log_settings.yaml",
            )
            .to_string();
        let _backend_mail_smtp_host: String = conf
            .get_from_or(Some("BackendMail"), "Mail_SMTP_Host", "")
            .to_string();
        let _backend_mail_smtp_user: String = conf
            .get_from_or(Some("BackendMail"), "Mail_SMTP_User", "")
            .to_string();
        let _backend_mail_smtp_password: String = conf
            .get_from_or(Some("BackendMail"), "Mail_SMTP_Password", "")
            .to_string();
        let _backend_mail_smtp_mail_address: String = conf
            .get_from_or(Some("BackendMail"), "Mail_SMTP_MailAddress", "")
            .to_string();
        let _frontend_register_user_mail_validation_regex_path: String = conf
            .get_from_or(
                Some("Frontend_RegisterUser"),
                "RegisterUser_Mail_Validation_Regex_Path",
                "",
            )
            .to_string();
        let _frontend_register_user_mail_info_subject: String = conf
            .get_from_or(
                Some("Frontend_RegisterUser"),
                "RegisterUser_Mail_Info_Subject",
                "",
            )
            .to_string();
        let _frontend_register_user_mail_info_body_path: String = conf
            .get_from_or(
                Some("Frontend_RegisterUser"),
                "RegisterUser_Mail_Info_Body_Path",
                "",
            )
            .to_string();
        let _frontend_register_user_mail_server_address: String = conf
            .get_from_or(
                Some("Frontend_RegisterUser"),
                "RegisterUser_Mail_Server_Address",
                "",
            )
            .to_string();
        let _frontend_password_reset_mail_validation_regex_path: String = conf
            .get_from_or(
                Some("Frontend_PasswordReset"),
                "PasswordReset_Mail_Validation_Regex_Path",
                "",
            )
            .to_string();
        let _frontend_password_reset_mail_info_subject: String = conf
            .get_from_or(
                Some("Frontend_PasswordReset"),
                "PasswordReset_Mail_Info_Subject",
                "",
            )
            .to_string();
        let _frontend_password_reset_mail_body_path: String = conf
            .get_from_or(
                Some("Frontend_PasswordReset"),
                "PasswordReset_Mail_Info_Body_Path",
                "",
            )
            .to_string();
        let _frontend_password_reset_mail_server_address: String = conf
            .get_from_or(
                Some("Frontend_PasswordReset"),
                "PasswordReset_Mail_Server_Address",
                "",
            )
            .to_string();
        let _frontend_password_reset_token_time_limit_minutes: u16 = conf
            .get_from_or(
                Some("Frontend_PasswordReset"),
                "PasswordReset_Token_Time_Limit_Minutes",
                "5",
            )
            .parse()
            .unwrap();

        return SettingStruct {
            web_server_ip_part1: _web_server_ip_part1,
            web_server_ip_part2: _web_server_ip_part2,
            web_server_ip_part3: _web_server_ip_part3,
            web_server_ip_part4: _web_server_ip_part4,
            web_server_port_http: _web_server_port_http,
            web_server_port_https: _web_server_port_https,
            web_server_cert_cert_path: _web_server_cert_cert_path,
            web_server_cert_key_path: _web_server_cert_key_path,
            backend_database_url: _db_url,
            backend_database_user: _db_user,
            backend_database_password: _db_password,
            backend_database_instance: _db_instance,
            log_config_path: _log_config_path,
            backend_mail_smtp_host: _backend_mail_smtp_host,
            backend_mail_smtp_user: _backend_mail_smtp_user,
            backend_mail_smtp_password: _backend_mail_smtp_password,
            backend_mail_smtp_mail_address: _backend_mail_smtp_mail_address,
            frontend_register_user_mail_validation_regex_path:
                _frontend_register_user_mail_validation_regex_path,
            frontend_register_user_mail_info_subject: _frontend_register_user_mail_info_subject,
            frontend_register_user_mail_info_body_path: _frontend_register_user_mail_info_body_path,
            frontend_register_user_mail_server_address: _frontend_register_user_mail_server_address,
            frontend_password_reset_mail_validation_regex_path:
                _frontend_password_reset_mail_validation_regex_path,
            frontend_password_reset_mail_info_subject: _frontend_password_reset_mail_info_subject,
            frontend_password_reset_mail_info_body_path: _frontend_password_reset_mail_body_path,
            frontend_password_reset_mail_server_address:
                _frontend_password_reset_mail_server_address,
            frontend_password_reset_token_time_limit_minutes:
                _frontend_password_reset_token_time_limit_minutes,
        };
    }
}

impl TestSettingStruct {
    pub fn global() -> &'static TestSettingStruct {
        GLOBAL_TEST_SETTING
            .get()
            .expect("GLOBAL_TEST_SETTING is not initialized")
    }

    pub fn create_dummy_setting(settingpath: &PathBuf) {
        let mut conf: Ini = Ini::new();
        conf.with_section(Some("[WARNING]"))
            .set("GITWARNING", "This is a default config file, when entering own value be sure to add this file to ignore")
            .set("GITWARNING2", "Only push to repository if this file does not contain any private information")
            .set("Renaming", "Use this file as tempalte to create your own ServerSettings.ini file");
        conf.with_section(Some("OutgoingMail"))
            .set("OutgoingMail_Receiver", "someUser@Server.org")
            .set("OutgoingMail_Title", "Testmessage")
            .set(
                "OutgoingMail_SimpleBody",
                "This is just some placeholder text \r please your own",
            );
        conf.with_section(Some("TestDatabase"))
            .set("DB_URL", "mongodb://localhost:27017")
            .set("DB_User", "TestUser")
            .set("DB_Password", "password")
            .set("DB_Instance", "StructureName");
        conf.write_to_file(&settingpath).unwrap();
    }

    pub fn load_from_file(settingpath: &PathBuf) -> Self {
        let conf: Ini = Ini::load_from_file(&settingpath).unwrap();
        let _outgoing_mail_receiver: String = conf
            .get_from_or(Some("OutgoingMail"), "OutgoingMail_Receiver", "")
            .parse()
            .unwrap();
        let _outgoing_mail_title: String = conf
            .get_from_or(Some("OutgoingMail"), "OutgoingMail_Title", "")
            .parse()
            .unwrap();
        let _outgoing_mail_simple_body: String = conf
            .get_from_or(Some("OutgoingMail"), "OutgoingMail_SimpleBody", "")
            .parse()
            .unwrap();
        let _test_database_db_url: String = conf
            .get_from_or(Some("TestDatabase"), "DB_URL", "")
            .parse()
            .unwrap();
        let _test_database_db_user: String = conf
            .get_from_or(Some("TestDatabase"), "DB_User", "")
            .parse()
            .unwrap();
        let _test_database_db_password: String = conf
            .get_from_or(Some("TestDatabase"), "DB_Password", "")
            .parse()
            .unwrap();
        let _test_database_db_instance: String = conf
            .get_from_or(Some("TestDatabase"), "DB_Instance", "")
            .parse()
            .unwrap();

        return TestSettingStruct {
            outgoing_mail_receiver: _outgoing_mail_receiver,
            outgoing_mail_title: _outgoing_mail_title,
            outgoing_mail_simple_body: _outgoing_mail_simple_body,
            test_database_db_instance: _test_database_db_instance,
            test_database_db_password: _test_database_db_password,
            test_database_db_url: _test_database_db_url,
            test_database_db_user: _test_database_db_user,
        };
    }
}
