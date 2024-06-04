#[cfg(test)]

mod test_accounting_handle {
    use std::path::{Path, PathBuf};

    use mongodb::bson::Uuid;

    use crate::{
        accounting_config_logic::FinanceAccounttingHandle,
        database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
        datatypes::FinanceAccountType,
        password_handle::{validate_credentials, UserCredentials},
        setting_struct::{self, SettingStruct, TestSettingStruct},
        tests::{
            mocking_database::{InMemoryDatabaseData, InMemoryDatabaseHandler},
            testing_accounting_config::test_accounting_handle,
        },
    };

    #[tokio::test]
    async fn test_acounting_type_config_handling_with_mock() {
        let dummy_connection_settings = DbConnectionSetting {
            instance: "".into(),
            password: "".into(),
            url: "".into(),
            user: "".into(),
        };
        let user_id_1 = Uuid::new();
        let user_id_2 = Uuid::new();
        let user_id_3 = Uuid::new();
        let user_id_4 = Uuid::new();
        let mut id_list: Vec<Uuid> = Vec::new();

        id_list.push(user_id_1.clone());
        id_list.push(user_id_2.clone());
        id_list.push(user_id_3.clone());

        let entry_object1 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_1);
        let entry_object2 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_2);
        let entry_object3 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_3);

        let _insert_result = InMemoryDatabaseData::insert_in_memory_database(Vec::from([
            entry_object1,
            entry_object2,
            entry_object3,
        ]));

        let in_memory_db = InMemoryDatabaseHandler {};

        let account_handle_1 =
            FinanceAccounttingHandle::new(&dummy_connection_settings, &user_id_1, &in_memory_db);
        let mut account_handle_2 =
            FinanceAccounttingHandle::new(&dummy_connection_settings, &user_id_2, &in_memory_db);
        let mut account_handle_3 =
            FinanceAccounttingHandle::new(&dummy_connection_settings, &user_id_3, &in_memory_db);
        let mut account_handle_4 =
            FinanceAccounttingHandle::new(&dummy_connection_settings, &user_id_4, &in_memory_db);

        //prepare data
        //empty list
        let list_1_result = account_handle_1.finance_account_type_list();
        //list with one element
        let finance_account_type_1 = FinanceAccountType {
            description: "SomeTypeDescription".to_string(),
            title: "SomeType".to_string(),
            id: Uuid::new(),
        };
        let insert_result_1 =
            account_handle_2.finance_account_type_upsert(&mut finance_account_type_1.clone());
        //list with two elements where one is updated
        let mut finance_account_type_2 = FinanceAccountType {
            description: "SomeTypeDescription2".to_string(),
            title: "SomeType2".to_string(),
            id: Uuid::new(),
        };
        let finance_account_type_3 = FinanceAccountType {
            description: "SomeTypeDescription3".to_string(),
            title: "SomeType3".to_string(),
            id: Uuid::new(),
        };
        let insert_result_2 =
            account_handle_3.finance_account_type_upsert(&mut finance_account_type_2.clone());
        let insert_result_3 =
            account_handle_3.finance_account_type_upsert(&mut finance_account_type_3.clone());
        finance_account_type_2.description = "UpdatedDescription".to_string();
        finance_account_type_2.title = "UpdatedTitle".to_string();
        let update_result_1 =
            account_handle_3.finance_account_type_upsert(&mut finance_account_type_2);
        //listing that returns an error because user not existing
        let finance_account_type_4 = FinanceAccountType {
            description: "SomeTypeDescription4".to_string(),
            title: "SomeType4".to_string(),
            id: Uuid::new(),
        };
        let insert_result_4 =
            account_handle_4.finance_account_type_upsert(&mut finance_account_type_4.clone());

        //test data
        if list_1_result.is_ok() {
            assert_eq!(list_1_result.unwrap().len(), 0)
        } else {
            panic!("{}", list_1_result.unwrap_err())
        }

        assert!(insert_result_1.is_ok());
        let list_2_result = account_handle_2.finance_account_type_list();
        if list_2_result.is_ok() {
            let returned_list = list_2_result.unwrap();
            assert_eq!(returned_list.len(), 1);
            assert_eq!(returned_list[0], finance_account_type_1);
        } else {
            panic!("{}", list_2_result.unwrap_err())
        }

        assert!(insert_result_2.is_ok());
        assert!(insert_result_3.is_ok());
        assert!(update_result_1.is_ok());
        let list_3_result = account_handle_3.finance_account_type_list();
        if list_3_result.is_ok() {
            let returned_list = list_3_result.unwrap();
            assert_eq!(returned_list.len(), 2);
            assert!(test_accounting_handle::account_type_list_contains_element(
                &returned_list,
                &finance_account_type_2
            ));
            assert!(test_accounting_handle::account_type_list_contains_element(
                &returned_list,
                &finance_account_type_3
            ));
        } else {
            panic!("{}", list_3_result.unwrap_err())
        }

        assert!(insert_result_4.is_err());
    }

    #[tokio::test]
    async fn test_acounting_type_config_handling_with_mongodb() {
        init();
        //let local_setting: SettingStruct = SettingStruct::global().clone();
        let test_setting = TestSettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(test_setting.backend_database_url),
            user: String::from(test_setting.backend_database_user),
            password: String::from(test_setting.backend_database_password),
            instance: String::from(test_setting.backend_database_instance),
        };
        //
        let credentials = UserCredentials {
            username: test_setting.test_user_account_user_login,
            password: test_setting.test_user_account_user_password.into(),
        };

        let validate_result = validate_credentials(&db_connection, &credentials).await;
        if validate_result.is_err() {
            panic!(
                "test user {} not valid: {}",
                credentials.username,
                validate_result.unwrap_err()
            );
        }

        let user_id_1 = validate_result.unwrap();
        let mongo_db = DbHandlerMongoDB {};

        let mut account_handle_1 =
            FinanceAccounttingHandle::new(&db_connection, &user_id_1, &mongo_db);

        //prepare data
        //First lilst
        let list_1_result = account_handle_1.finance_account_type_list();
        //inserting 2 Elements
        let finance_account_type_1 = FinanceAccountType {
            description: "SomeTypeDescription_".to_string() + &Uuid::new().to_string(),
            title: "SomeType_".to_string() + &Uuid::new().to_string(),
            id: Uuid::new(),
        };
        let insert_result_1 =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_1.clone());
        let finance_account_type_2 = FinanceAccountType {
            description: "SomeTypeDescription2_".to_string() + &Uuid::new().to_string(),
            title: "SomeType2_".to_string() + &Uuid::new().to_string(),
            id: Uuid::new(),
        };
        let insert_result_2 =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_2.clone());
        let list_2_result = account_handle_1.finance_account_type_list();
        let mut finance_account_type_3 = finance_account_type_2.clone();
        finance_account_type_3.description =
            "UpdatedDescription_".to_string() + &Uuid::new().to_string();
        finance_account_type_3.title = "UpdatedTitle_".to_string() + &Uuid::new().to_string();
        let update_result_1 =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_3);
        let list_3_result = account_handle_1.finance_account_type_list();

        //test data

        assert!(list_1_result.is_ok(), "{}", list_1_result.unwrap_err());
        assert!(list_2_result.is_ok(), "{}", list_2_result.unwrap_err());
        assert!(list_3_result.is_ok(), "{}", list_3_result.unwrap_err());
        assert!(insert_result_1.is_ok(), "{}", insert_result_1.unwrap_err());
        assert!(insert_result_2.is_ok(), "{}", insert_result_2.unwrap_err());
        assert!(update_result_1.is_ok(), "{}", update_result_1.unwrap_err());

        let list1 = list_1_result.unwrap();
        let list2 = list_2_result.unwrap();
        let list3 = list_3_result.unwrap();

        assert_eq!(list1.len() + 2, list2.len());
        assert_eq!(list2.len(), list3.len());

        assert!(test_accounting_handle::account_type_list_contains_element(
            &list2,
            &finance_account_type_1
        ));
        assert!(test_accounting_handle::account_type_list_contains_element(
            &list2,
            &finance_account_type_2
        ));
        assert!(test_accounting_handle::account_type_list_contains_element(
            &list3,
            &finance_account_type_1
        ));
        assert!(test_accounting_handle::account_type_list_contains_element(
            &list3,
            &finance_account_type_3
        ));
    }

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
    async fn test_acounting_config_handle_with_mock() {
        let dummy_connection_settings = DbConnectionSetting {
            instance: "".into(),
            password: "".into(),
            url: "".into(),
            user: "".into(),
        };

        /* test preparations:
           2 users a and b
           user a has:
           - Account type a_1
           - Account type a_2
           user b has:
           - Account type b_1
        */
        let user_id_1 = Uuid::new();
        let mut id_list: Vec<Uuid> = Vec::new();

        id_list.push(user_id_1.clone());

        let account_types_user_1: Vec<FinanceAccountType> = Vec::new();

        let entry_object1 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_1);
        let insert_result =
            InMemoryDatabaseData::insert_in_memory_database(Vec::from([entry_object1]));
    }

    fn account_type_list_contains_element(
        list_to_check: &Vec<FinanceAccountType>,
        element_to_check: &FinanceAccountType,
    ) -> bool {
        let position_option = list_to_check
            .iter()
            .position(|elem| elem.id.eq(&element_to_check.id));
        if let Some(position) = position_option {
            return list_to_check[position].eq(element_to_check);
        }
        return false;
    }
}
