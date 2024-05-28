#[cfg(test)]

mod test_accounting_handle {
    use mongodb::bson::Uuid;

    use crate::{
        accounting_config_logic::FinanceAccounttingHandle,
        datatypes::FinanceAccountType,
        tests::mocking_database::{InMemoryDatabaseData, InMemoryDatabaseHandler},
    };

    #[tokio::test]
    async fn test_acounting_with_mock() {
        let user_id_1 = Uuid::new();
        let user_id_2 = Uuid::new();
        let user_id_3 = Uuid::new();
        let user_id_4 = Uuid::new();
        let mut id_list: Vec<Uuid> = Vec::new();
        //let user_id_1_2 = user_id_1.clone();
        //let mut user_id_2_2 = user_id_2.clone();
        //let mut user_id_3_2 = user_id_3.clone();
        //let mut user_id_4_2 = user_id_4.clone();
        id_list.push(user_id_1.clone());
        id_list.push(user_id_2.clone());
        id_list.push(user_id_3.clone());
        id_list.push(user_id_4.clone());

        let account_types_user_1: Vec<FinanceAccountType> = Vec::new();
        let account_types_user_2: Vec<FinanceAccountType> = Vec::new();
        let account_types_user_3: Vec<FinanceAccountType> = Vec::new();
        let account_types_user_4: Vec<FinanceAccountType> = Vec::new();
        let mut account_types_all_users = Vec::new();
        account_types_all_users.push(account_types_user_1);
        account_types_all_users.push(account_types_user_2);
        account_types_all_users.push(account_types_user_3);
        account_types_all_users.push(account_types_user_4);

        let data_obj = InMemoryDatabaseData::create_in_memory_database_data_object(
            id_list,
            account_types_all_users,
        );
        let mutex_obj = std::sync::Mutex::new(data_obj);

        let _ = crate::tests::mocking_database::GLOBAL_IN_MEMORY_DATA.set(mutex_obj);

        let in_memory_db = InMemoryDatabaseHandler {};

        let account_handle_1 = FinanceAccounttingHandle::new(&user_id_1, &in_memory_db);
        let mut account_handle_2 = FinanceAccounttingHandle::new(&user_id_2, &in_memory_db);
        let mut account_handle_3 = FinanceAccounttingHandle::new(&user_id_3, &in_memory_db);
        let mut account_handle_4 = FinanceAccounttingHandle::new(&user_id_4, &in_memory_db);

        //prepare data
        //empty list
        let list_1_result = account_handle_1.finance_account_type_list().await;
        //list with one element
        let finance_account_type_1 = FinanceAccountType {
            description: "SomeTypeDescription".to_string(),
            title: "SomeType".to_string(),
            id: Uuid::new(),
        };
        let insert_result_1 = account_handle_2
            .finance_account_type_upsert(&mut finance_account_type_1.clone())
            .await;
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
        let insert_result_2 = account_handle_3
            .finance_account_type_upsert(&mut finance_account_type_2.clone())
            .await;
        let insert_result_3 = account_handle_3
            .finance_account_type_upsert(&mut finance_account_type_3.clone())
            .await;
        finance_account_type_2.description = "UpdatedDescription".to_string();
        finance_account_type_2.title = "UpdatedTitle".to_string();
        let update_result_1 = account_handle_3
            .finance_account_type_upsert(&mut finance_account_type_2)
            .await;
        //listing that returns an error
        let finance_account_type_4 = FinanceAccountType {
            description: "SomeTypeDescription4".to_string(),
            title: "SomeType4".to_string(),
            id: Uuid::new(),
        };
        let insert_result_4 = account_handle_4
            .finance_account_type_upsert(&mut finance_account_type_4.clone())
            .await;

        //test data
        if list_1_result.is_ok() {
            assert_eq!(list_1_result.unwrap().len(), 0)
        } else {
            panic!("{}", list_1_result.unwrap_err())
        }

        assert!(insert_result_1.is_ok());
        let list_2_result = account_handle_2.finance_account_type_list().await;
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
        let list_3_result = account_handle_3.finance_account_type_list().await;
        if list_3_result.is_ok() {
            let returned_list = list_3_result.unwrap();
            assert_eq!(returned_list.len(), 1);
            assert_eq!(returned_list[0], finance_account_type_2);
            assert_eq!(returned_list[0], finance_account_type_3);
        } else {
            panic!("{}", list_3_result.unwrap_err())
        }

        assert!(insert_result_4.is_err());
    }
}
