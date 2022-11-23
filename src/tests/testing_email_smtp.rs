#[cfg(test)]
mod test_email_smtp {

    use std::path::{Path, PathBuf};

    use crate::{
        mail_handle::{self, SimpleMailData, SmtpMailSetting},
        setting_struct::{self, SettingStruct, TestSettingStruct},
    };

    //see https://stackoverflow.com/questions/58006033/how-to-run-setup-code-before-any-tests-run-in-rust
    static TEST_INIT: std::sync::Once = std::sync::Once::new();

    fn init() {
        TEST_INIT.call_once(|| {
            //get configuration from ini file
            let working_dir = std::env::current_dir().unwrap();
            let config_dir: PathBuf = Path::new(&working_dir).join("config");
            if !config_dir.exists() {
                panic!("Testing: Config folder not present, aborting tests");
            }

            let server_settings_file = Path::new(&config_dir).join("ServerSettings.ini");
            if !server_settings_file.exists() {
                panic!("Testing: Server Setting file not present, aborting tests");
            }

            let test_settings_file = Path::new(&config_dir).join("TestSettings.ini");
            if !test_settings_file.exists() {
                panic!("Testing: Test Setting file not present, aborting tests");
            }

            let dummy_server_settings_file = Path::new(&config_dir).join("DUMMY_ServerSettings.ini");
            if !dummy_server_settings_file.exists()
            {
                log::debug!(target: "app::FinanceOverView","Dummy setting file not found, will be created at {}",dummy_server_settings_file.to_string_lossy());
                SettingStruct::create_dummy_setting(&dummy_server_settings_file);
            }

            let dummy_test_settings_file = Path::new(&config_dir).join("DUMMY_TestSettings.ini");
            if !dummy_test_settings_file.exists()
            {
                log::debug!(target: "app::FinanceOverView","Dummy test setting file not found, will be created at {}",dummy_test_settings_file.to_string_lossy());
                TestSettingStruct::create_dummy_setting(&dummy_test_settings_file);
            }

            let local_setting = SettingStruct::load_from_file(&server_settings_file);
            setting_struct::GLOBAL_SETTING
                .set(local_setting.clone())
                .ok();

            let test_setting = TestSettingStruct::load_from_file(&test_settings_file);
            setting_struct::GLOBAL_TEST_SETTING
                .set(test_setting.clone())
                .ok();
        });
    }
    #[tokio::test]
    async fn test_mail_sending() {
        init();
        let local_setting: SettingStruct = SettingStruct::global().clone();
        let test_setting = TestSettingStruct::global().clone();

        let mail_content = SimpleMailData {
            receiver: test_setting.outgoing_mail_receiver,
            sender: local_setting.backend_mail_smtp_mail_address,
            subject: test_setting.outgoing_mail_title,
            body: test_setting.outgoing_mail_simple_body,
        };

        let mail_config = SmtpMailSetting {
            host: local_setting.backend_mail_smtp_host,
            client_name: local_setting.backend_mail_smtp_user,
            client_password: local_setting.backend_mail_smtp_password,
        };

        let result_async = mail_handle::send_smtp_mail(mail_content, mail_config);
        //let result =futures::executor::block_on(result_async);
        let result = result_async.await;

        assert!(
            result.is_ok(),
            "Error sending mail: {}",
            result.unwrap_err()
        );
    }
}
