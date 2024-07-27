#[cfg(test)]

pub(crate) mod test_accounting_handle {
    use std::path::{Path, PathBuf};

    use mongodb::bson::Uuid;

    use crate::{
        accounting_config_logic::FinanceAccountingConfigHandle,
        database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
        datatypes::{FinanceAccount, FinanceAccountType},
        password_handle::{validate_credentials, UserCredentials},
        setting_struct::{self, SettingStruct, TestSettingStruct},
        tests::{
            mocking_database::{InMemoryDatabaseData, InMemoryDatabaseHandler},
            testing_accounting_config::test_accounting_handle,
        },
    };

    #[tokio::test]
    async fn test_accounting_type_config_handling_with_mock() {
        let user_id_1 = Uuid::new();
        let user_id_2 = Uuid::new();
        let user_id_3 = Uuid::new();
        let user_id_4 = Uuid::new();

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

        let account_handle_1 = FinanceAccountingConfigHandle::new(&user_id_1, &in_memory_db);
        let mut account_handle_2 = FinanceAccountingConfigHandle::new(&user_id_2, &in_memory_db);
        let mut account_handle_3 = FinanceAccountingConfigHandle::new(&user_id_3, &in_memory_db);
        let mut account_handle_4 = FinanceAccountingConfigHandle::new(&user_id_4, &in_memory_db);

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

        //using account type id from another user must fail
        /*let finance_account_type_1 = FinanceAccountType {
            description: "SomeTypeDescription".to_string(),
            title: "SomeType".to_string(),
            id: Uuid::new(),
        };
        */
        let mut finance_account_type_5 = finance_account_type_1.clone();
        finance_account_type_5.description = "ERRORDescription".to_string();
        finance_account_type_5.title = "ERRORType".to_string();
        let insert_result_5 =
            account_handle_3.finance_account_type_upsert(&mut finance_account_type_5.clone());
        assert!(
            insert_result_5.is_err(),
            "Using account type from different user must fail"
        );
        let errmsg = insert_result_5.unwrap_err();
        assert!(errmsg.contains("account type id not available for current user"));
    }

    #[tokio::test]
    async fn test_accounting_type_config_handling_with_mongodb() {
        init();
        //let local_setting: SettingStruct = SettingStruct::global().clone();
        let test_setting = TestSettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(test_setting.backend_database_url),
            user: String::from(test_setting.backend_database_user),
            password: String::from(test_setting.backend_database_password),
            instance: String::from(test_setting.backend_database_instance),
        };
        let mongo_db = DbHandlerMongoDB::new(&db_connection);

        //first user
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

        let mut account_handle_1 = FinanceAccountingConfigHandle::new(&user_id_1, &mongo_db);

        //second user
        let credentials_2 = UserCredentials {
            username: test_setting.test_user_2_account_user_login,
            password: test_setting.test_user_2_account_user_password.into(),
        };

        let validate_result_2 = validate_credentials(&db_connection, &credentials_2).await;
        if validate_result_2.is_err() {
            panic!(
                "test user 2 {} not valid: {}",
                credentials.username,
                validate_result_2.unwrap_err()
            );
        }

        let user_id_2 = validate_result_2.unwrap();

        let mut account_handle_2 = FinanceAccountingConfigHandle::new(&user_id_2, &mongo_db);

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

        let finance_account_type_4 = FinanceAccountType {
            description: "SomeTypeDescription4_".to_string() + &Uuid::new().to_string(),
            title: "SomeType4_".to_string() + &Uuid::new().to_string(),
            id: Uuid::new(),
        };
        let list_4_result = account_handle_2.finance_account_type_list();
        let insert_result_4 =
            account_handle_2.finance_account_type_upsert(&mut finance_account_type_4.clone());
        let list_5_result = account_handle_2.finance_account_type_list();
        assert!(list_4_result.is_ok(), "{}", list_4_result.unwrap_err());
        assert!(list_5_result.is_ok(), "{}", list_5_result.unwrap_err());
        assert!(insert_result_4.is_ok(), "{}", insert_result_4.unwrap_err());

        let list_4 = list_4_result.unwrap();
        let list_5 = list_5_result.unwrap();

        assert_eq!(list_4.len() + 1, list_5.len());
        assert!(test_accounting_handle::account_type_list_contains_element(
            &list_5,
            &finance_account_type_4
        ));

        //trying to update account type using another user, musst fail
        let mut finance_account_type_2_update = finance_account_type_2.clone();
        finance_account_type_2_update.description =
            "ERRORDescription2_Update".to_string() + &Uuid::new().to_string();
        finance_account_type_2_update.title =
            "ERRORType2_Update_".to_string() + &Uuid::new().to_string();
        let update_result_2 =
            account_handle_2.finance_account_type_upsert(&mut finance_account_type_3);
        assert!(
            update_result_2.is_err(),
            "Updating with wrong user must fail"
        );
        let errmsg = update_result_2.unwrap_err();
        assert!(errmsg.contains("account type id not available for current user"));
    }

    #[tokio::test]
    async fn test_accounting_config_handle_with_mock() {
        /* test preparations:
           3 users a, b and c
           user a has:
           - Account type a_1
           - Account type a_2
           user b has:
           - Account type b_1
           user c: not inserted in database
        */
        let in_memory_db = InMemoryDatabaseHandler {};
        let user_id_1 = Uuid::new();
        let user_id_2 = Uuid::new();
        let user_id_3 = Uuid::new();

        let mut account_handle_1: FinanceAccountingConfigHandle =
            FinanceAccountingConfigHandle::new(&user_id_1, &in_memory_db);
        let mut account_handle_2 = FinanceAccountingConfigHandle::new(&user_id_2, &in_memory_db);
        let mut account_handle_3 = FinanceAccountingConfigHandle::new(&user_id_3, &in_memory_db);

        let entry_object1 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_1);
        let entry_object2 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_2);
        let init_db_result = InMemoryDatabaseData::insert_in_memory_database(Vec::from([
            entry_object1,
            entry_object2,
        ]));
        assert!(
            init_db_result.is_ok(),
            "Could not prepare database for test"
        );

        let finance_account_type_a_1 = FinanceAccountType {
            description: "SomeTypeDescription_a_1".to_string(),
            title: "SomeType_a_1".to_string(),
            id: Uuid::new(),
        };
        let finance_account_type_a_2 = FinanceAccountType {
            description: "SomeTypeDescription_a_2".to_string(),
            title: "SomeType_a_2".to_string(),
            id: Uuid::new(),
        };
        let finance_account_type_b_1 = FinanceAccountType {
            description: "SomeTypeDescription_b_1".to_string(),
            title: "SomeType_b_1".to_string(),
            id: Uuid::new(),
        };
        let insert_result_fat_a1 =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_a_1.clone());
        let insert_result_fat_a2 =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_a_2.clone());
        let insert_result_fat_b1 =
            account_handle_2.finance_account_type_upsert(&mut finance_account_type_b_1.clone());
        assert!(
            insert_result_fat_a1.is_ok()
                && insert_result_fat_a2.is_ok()
                && insert_result_fat_b1.is_ok(),
            "Could not prepare database for testing"
        );

        /* Testcase 1
           adding 3 new accounts for user a:
           account 1: type 1
           account 2: type 2
           account 3: type 1

           check:
           # listing size should increase with each insert
           # each listing should contains the correct accounts
        */
        let finance_account_1_1 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_a_1.id,
            title: "account_1_1".into(),
            description: "description_1_1".into(),
        };
        let finance_account_1_2 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_a_2.id,
            title: "account_1_2".into(),
            description: "description_1_2".into(),
        };
        let finance_account_1_3 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_a_1.id,
            title: "account_1_3".into(),
            description: "description_1_3".into(),
        };
        let list_0_result = account_handle_1.finance_account_list(None);
        let insert_1_result = account_handle_1.finance_account_upsert(&finance_account_1_1);
        let list_1_result = account_handle_1.finance_account_list(None);
        let insert_2_result = account_handle_1.finance_account_upsert(&finance_account_1_2);
        let list_2_result = account_handle_1.finance_account_list(None);
        let insert_3_result = account_handle_1.finance_account_upsert(&finance_account_1_3);
        let list_3_result = account_handle_1.finance_account_list(None);

        assert!(list_0_result.is_ok(), "{}", list_0_result.unwrap_err());
        assert!(list_1_result.is_ok(), "{}", list_1_result.unwrap_err());
        assert!(list_2_result.is_ok(), "{}", list_2_result.unwrap_err());
        assert!(list_3_result.is_ok(), "{}", list_3_result.unwrap_err());
        assert!(insert_1_result.is_ok(), "{}", insert_1_result.unwrap_err());
        assert!(insert_2_result.is_ok(), "{}", insert_2_result.unwrap_err());
        assert!(insert_3_result.is_ok(), "{}", insert_3_result.unwrap_err());

        let list_length_base = list_0_result.unwrap().len();
        let list1 = list_1_result.unwrap();
        let list2 = list_2_result.unwrap();
        let list3 = list_3_result.unwrap();
        assert!(
            list1.len().eq(&(list_length_base + 1))
                && list2.len().eq(&(list_length_base + 2))
                && list3.len().eq(&(list_length_base + 3)),
            "returned list length do not match"
        );
        assert!(account_list_contains_element(&list1, &finance_account_1_1));
        assert!(account_list_contains_element(&list2, &finance_account_1_1));
        assert!(account_list_contains_element(&list2, &finance_account_1_2));
        assert!(account_list_contains_element(&list3, &finance_account_1_1));
        assert!(account_list_contains_element(&list3, &finance_account_1_2));
        assert!(account_list_contains_element(&list3, &finance_account_1_3));

        /* Testcase 2
            adding 2 new accounts for user b:
            account 1: type 1
            account 2: new account type, not stored in database

            check:
            adding account 1 ok
            adding account 2 fails
        */
        let finance_account_2_1 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_b_1.id,
            title: "account_2_1".into(),
            description: "description_2_1".into(),
        };
        let finance_account_type_b_2 = FinanceAccountType {
            description: "SomeTypeDescription_b_2".to_string(),
            title: "SomeType_b_2".to_string(),
            id: Uuid::new(),
        };
        let finance_account_2_2 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_b_2.id,
            title: "account_2_2".into(),
            description: "description_2_2".into(),
        };
        let list_2_0_result = account_handle_2.finance_account_list(None);
        let insert_2_1_result = account_handle_2.finance_account_upsert(&finance_account_2_1);
        let list_2_1_result = account_handle_2.finance_account_list(None);
        let insert_2_2_result = account_handle_2.finance_account_upsert(&finance_account_2_2);
        assert!(list_2_0_result.is_ok(), "{}", list_2_0_result.unwrap_err());
        assert!(list_2_1_result.is_ok(), "{}", list_2_1_result.unwrap_err());
        assert!(
            insert_2_1_result.is_ok(),
            "{}",
            list_2_1_result.unwrap_err()
        );
        assert!(
            insert_2_2_result.is_err(),
            "inserting with unkown account type did not failed"
        );
        let list_length_base_2 = list_2_0_result.unwrap().len();
        let list_2_1 = list_2_1_result.unwrap();
        assert!(
            list_2_1.len().eq(&(list_length_base_2 + 1)),
            "return list length does not match"
        );
        assert!(account_list_contains_element(
            &list_2_1,
            &finance_account_2_1
        ));

        /* Testcase 3
           rename account 2 from user 1

           check:
           # new list contains the updated entry
        */
        let finance_account_1_2_update = FinanceAccount {
            id: finance_account_1_2.id,
            finance_account_type_id: finance_account_1_2.finance_account_type_id,
            title: "Update_1_2".into(),
            description: "Another description for 1_2".into(),
        };
        let upsert_result = account_handle_1.finance_account_upsert(&finance_account_1_2_update);
        let list_update_result = account_handle_1.finance_account_list(None);
        assert!(upsert_result.is_ok(), "{}", upsert_result.unwrap_err());
        assert!(
            list_update_result.is_ok(),
            "{}",
            list_update_result.unwrap_err()
        );

        let list_update = list_update_result.unwrap();
        assert!(
            list3.len().eq(&list_update.len()),
            "return list length does not match"
        );

        assert!(account_list_contains_element(
            &list_update,
            &finance_account_1_1
        ));
        assert!(account_list_contains_element(
            &list_update,
            &finance_account_1_2_update
        ));
        assert!(account_list_contains_element(
            &list_update,
            &finance_account_1_3
        ));
        assert!(!account_list_contains_element(
            &list_update,
            &finance_account_1_2
        ));

        /* Testcase 4 checking limiting query for accounts */
        let list_4_result = account_handle_1
            .finance_account_list_async(Some(&vec![
                finance_account_1_1.id,
                finance_account_1_2_update.id,
            ]))
            .await;
        assert!(list_4_result.is_ok(), "{}", list_4_result.unwrap_err());
        let list4 = list_4_result.unwrap();
        assert_eq!(list4.len(), 2);
        assert!(account_list_contains_element(&list4, &finance_account_1_1));
        assert!(account_list_contains_element(
            &list4,
            &finance_account_1_2_update
        ));
        let list_5_result = account_handle_1
            .finance_account_list_async(Some(&vec![finance_account_1_3.id]))
            .await;
        assert!(list_5_result.is_ok(), "{}", list_5_result.unwrap_err());
        let list5 = list_5_result.unwrap();
        assert_eq!(list5.len(), 1);
        assert!(account_list_contains_element(&list5, &finance_account_1_3));

        //listing an account id from a different user
        let list_6_result = account_handle_2
            .finance_account_list_async(Some(&vec![finance_account_1_3.id]))
            .await;
        assert!(list_6_result.is_ok(), "{}", list_6_result.unwrap_err());
        let list6 = list_6_result.unwrap();
        assert_eq!(list6.len(), 0);

        /* Testcase 5
        trying to list and insert value for a new user that does not exist

        Check:
            all operation have to fail
         */

        let list_e1_result = account_handle_3.finance_account_list(None);
        let insert_e1_result = account_handle_3.finance_account_upsert(&finance_account_1_1);
        assert!(
            list_e1_result.is_err(),
            "listing for unknown user has to fail"
        );
        assert!(
            insert_e1_result.is_err(),
            "inserting for unknown user has to fail"
        );

        /* Testcase 6
        trying to use IDs from another user must fail

        Check:
            all operations have to fail
         */

        let mut finance_account_2_3 = finance_account_1_1.clone();
        finance_account_2_3.id = Uuid::new();
        finance_account_2_3.title = "ERROR".into();
        finance_account_2_3.description = "ERROR".into();
        let insert_e2_result = account_handle_2.finance_account_upsert(&finance_account_2_3);
        assert!(
            insert_e2_result.is_err(),
            "using account type from another user must fail"
        );
        let errmsg_1 = insert_e2_result.unwrap_err();
        assert!(errmsg_1
            .contains("could not upsert finance account because account type is not available"));
        let mut finance_account_2_4 = finance_account_1_1.clone();
        finance_account_2_4.finance_account_type_id = finance_account_2_1.finance_account_type_id;
        finance_account_2_4.title = "ERROR".into();
        finance_account_2_4.description = "ERROR".into();
        let insert_e3_result = account_handle_2.finance_account_upsert(&finance_account_2_4);
        assert!(
            insert_e3_result.is_err(),
            "using account id from another user must fail"
        );
        let errmsg_2 = insert_e3_result.unwrap_err();
        assert!(errmsg_2.contains("account id not accessible for current user"));
    }

    #[tokio::test]
    async fn test_accounting_config_handling_with_mongodb() {
        init();
        //let local_setting: SettingStruct = SettingStruct::global().clone();
        let test_setting = TestSettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(test_setting.backend_database_url),
            user: String::from(test_setting.backend_database_user),
            password: String::from(test_setting.backend_database_password),
            instance: String::from(test_setting.backend_database_instance),
        };

        let mongo_db = DbHandlerMongoDB::new(&db_connection);

        //first user
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

        let mut account_handle_1 = FinanceAccountingConfigHandle::new(&user_id_1, &mongo_db);

        //second user
        let credentials_2 = UserCredentials {
            username: test_setting.test_user_2_account_user_login,
            password: test_setting.test_user_2_account_user_password.into(),
        };

        let validate_result_2 = validate_credentials(&db_connection, &credentials_2).await;
        if validate_result_2.is_err() {
            panic!(
                "test user 2 {} not valid: {}",
                credentials.username,
                validate_result_2.unwrap_err()
            );
        }

        let user_id_2 = validate_result_2.unwrap();

        let mut account_handle_2 = FinanceAccountingConfigHandle::new(&user_id_2, &mongo_db);

        //check if there are any finance account type available
        let list_fat_result = account_handle_1.finance_account_type_list();
        assert!(
            list_fat_result.is_ok(),
            "Could not load finance account type list: {}",
            list_fat_result.unwrap_err().to_string()
        );
        let available_finance_account_type = list_fat_result.unwrap();
        assert!(
            available_finance_account_type.len() > 1,
            "not enough finance account types available"
        );

        let user_2_types_result = account_handle_2.finance_account_type_list();
        assert!(
            user_2_types_result.is_ok(),
            "Could not load finance account type list: {}",
            user_2_types_result.unwrap_err().to_string()
        );

        /* Testcase 1
        insert 2 diffenert finance accounts

        check:
        * inserts are okay
        * list of finance accounts increases and contains the newly inserted accounts
         */
        let id1 = Uuid::new();
        let id2 = Uuid::new();
        let account_1 = FinanceAccount {
            id: id1,
            finance_account_type_id: available_finance_account_type[0].id,
            title: "SomeTitle".to_string() + &id1.to_string(),
            description: "some Decription for ".to_string() + &id1.to_string(),
        };
        let account_2 = FinanceAccount {
            id: id2,
            finance_account_type_id: available_finance_account_type[0].id,
            title: "SomeTitle".to_string() + &id2.to_string(),
            description: "some Decription for ".to_string() + &id2.to_string(),
        };
        let list_accounts_0_result = account_handle_1.finance_account_list(None);
        let insert_1_result = account_handle_1.finance_account_upsert(&account_1);
        let list_accounts_1_result = account_handle_1.finance_account_list(None);
        let insert_2_result = account_handle_1.finance_account_upsert(&account_2);
        let list_accounts_2_result = account_handle_1.finance_account_list(None);

        assert!(
            list_accounts_0_result.is_ok(),
            "{}",
            list_accounts_0_result.unwrap_err()
        );
        assert!(
            list_accounts_1_result.is_ok(),
            "{}",
            list_accounts_1_result.unwrap_err()
        );
        assert!(
            list_accounts_1_result.is_ok(),
            "{}",
            list_accounts_2_result.unwrap_err()
        );
        assert!(insert_1_result.is_ok(), "{}", insert_1_result.unwrap_err());
        assert!(insert_2_result.is_ok(), "{}", insert_2_result.unwrap_err());
        let list0 = list_accounts_0_result.unwrap();
        let list1 = list_accounts_1_result.unwrap();
        let list2 = list_accounts_2_result.unwrap();
        let base_length = list0.len();

        assert!(
            list1.len().eq(&(base_length + 1)) & list2.len().eq(&(base_length + 2)),
            "Length of list do not match"
        );
        assert!(account_list_contains_element(&list1, &account_1));
        assert!(account_list_contains_element(&list2, &account_1));
        assert!(account_list_contains_element(&list2, &account_2));

        /* Testcase 2
        update an account

        Checks:
        listing of account has same size
        old value not in listing anymore, it is replaced by new value
         */
        let account_updated = FinanceAccount {
            id: account_2.id,
            finance_account_type_id: account_2.finance_account_type_id,
            title: "Updated".to_string() + &account_2.id.to_string(),
            description: "changed description".to_string() + &account_2.id.to_string(),
        };
        let insert_updated_result = account_handle_1.finance_account_upsert(&account_updated);
        let list_updated_result = account_handle_1.finance_account_list(None);
        assert!(
            insert_updated_result.is_ok(),
            "{}",
            insert_updated_result.unwrap_err()
        );
        assert!(
            list_updated_result.is_ok(),
            "{}",
            list_updated_result.unwrap_err()
        );

        let list_updated = list_updated_result.unwrap();

        assert!(
            list2.len().eq(&list_updated.len()),
            "length after updating does not match"
        );
        assert!(account_list_contains_element(&list_updated, &account_1));
        assert!(account_list_contains_element(
            &list_updated,
            &account_updated
        ));
        assert!(!account_list_contains_element(&list_updated, &account_2));

        /* Testcase 3 limited query */
        let list_fat_2_result = account_handle_1.finance_account_list_async(None).await;
        let list_fat_2 = list_fat_2_result.unwrap();
        let index_start = 0;
        let index_end = list_fat_2.len() - 1;
        let index_middle = index_end / 2;
        let sub_list_1: Vec<FinanceAccount> = list_fat_2[index_start..index_middle]
            .iter()
            .cloned()
            .collect();
        let sub_list_2: Vec<FinanceAccount> = list_fat_2[index_middle + 1..index_end]
            .iter()
            .cloned()
            .collect();
        let sub_ids_1 = sub_list_1.iter().map(|elem| elem.id).collect::<Vec<Uuid>>();
        let sub_ids_2 = sub_list_2.iter().map(|elem| elem.id).collect::<Vec<Uuid>>();

        let limit_list_1_result = account_handle_1
            .finance_account_list_async(Some(&sub_ids_1))
            .await;
        let limit_list_2_result = account_handle_1
            .finance_account_list_async(Some(&sub_ids_2))
            .await;

        assert!(
            limit_list_1_result.is_ok(),
            "{}",
            limit_list_1_result.unwrap_err()
        );
        assert!(
            limit_list_2_result.is_ok(),
            "{}",
            limit_list_2_result.unwrap_err()
        );
        let limit_list_1 = limit_list_1_result.unwrap();
        let limit_list_2 = limit_list_2_result.unwrap();
        assert_eq!(limit_list_1.len(), sub_ids_1.len());
        assert_eq!(limit_list_2.len(), sub_ids_2.len());

        for account in sub_list_1 {
            assert!(account_list_contains_element(&limit_list_1, &account))
        }
        for account in sub_list_2 {
            assert!(account_list_contains_element(&limit_list_2, &account))
        }
        //listing an account id from a different user
        let limit_list_3_result = account_handle_2
            .finance_account_list_async(Some(&sub_ids_1))
            .await;
        assert!(
            limit_list_3_result.is_ok(),
            "{}",
            limit_list_3_result.unwrap_err()
        );
        let limit_list_3 = limit_list_3_result.unwrap();
        assert_eq!(limit_list_3.len(), 0);

        //try to create an account with an account type to another user => must fail
        let mut account_3 = account_1.clone();
        account_3.id = Uuid::new();
        account_3.title = "ERROR".to_string() + &account_3.id.to_string();
        account_3.description = "ERROR".to_string() + &account_3.id.to_string();
        let insert_3_result = account_handle_2.finance_account_upsert(&account_3);
        assert!(
            insert_3_result.is_err(),
            "Using account type from different user must fail"
        );
        let errmsg_1 = insert_3_result.unwrap_err();
        assert!(errmsg_1
            .contains("could not upsert finance account because account type is not available"));

        //try to use an account ID from another user => must fail
        let user_2_types = user_2_types_result.unwrap();
        let mut account_4 = account_1.clone();
        account_4.finance_account_type_id = user_2_types[0].id;
        account_4.title = "ERROR".to_string() + &account_4.id.to_string();
        account_4.description = "ERROR".to_string() + &account_4.id.to_string();
        let insert_4_result = account_handle_2.finance_account_upsert(&account_4);
        assert!(
            insert_4_result.is_err(),
            "Using account from different user must fail"
        );
        let errmsg_2 = insert_4_result.unwrap_err();
        assert!(errmsg_2.contains("account id not accessible for current user"));
    }

    //see https://stackoverflow.com/questions/58006033/how-to-run-setup-code-before-any-tests-run-in-rust
    pub static TEST_INIT: std::sync::Once = std::sync::Once::new();

    pub fn init() {
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

    fn account_list_contains_element(
        list_to_check: &Vec<FinanceAccount>,
        element_to_check: &FinanceAccount,
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
