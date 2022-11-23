#[cfg(test)]
mod test_email_validation {
    use std::path::{Path, PathBuf};

    use crate::{
        mail_handle,
        setting_struct::{self, SettingStruct, TestSettingStruct},
    };

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

    #[test]
    fn test_mail_validation() {
        init();

        let testmail = "someUser@website.org";
        let mut result = mail_handle::validate_email(&testmail.to_string());
        if result.is_err() {
            panic!(
                "Panic validation email {}: {}",
                &testmail,
                result.unwrap_err()
            );
        }
        assert!(
            result.unwrap(),
            "Error validation e-mail address: {}",
            &testmail
        );

        let invalidmail = "nopemail@";
        result = mail_handle::validate_email(&invalidmail.to_string());
        if result.is_err() {
            panic!(
                "Panic validation invalid email {}: {}",
                &invalidmail,
                result.unwrap_err()
            );
        }
        assert!(
            !result.unwrap(),
            "Error invalid email marked as valid: {}",
            &invalidmail
        );

        let emptymail = "";
        result = mail_handle::validate_email(&emptymail.to_string());
        if result.is_err() {
            panic!(
                "Panic validation empty email {}: {}",
                &emptymail,
                result.unwrap_err()
            );
        }
        assert!(
            !result.unwrap(),
            "Error empty email marked as valid{}",
            &emptymail
        );
    }
}
