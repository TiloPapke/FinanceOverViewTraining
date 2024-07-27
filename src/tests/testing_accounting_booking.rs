#[cfg(test)]
pub static GLOBAL_PREPARED_MONGODB: std::sync::Once = std::sync::Once::new();

#[cfg(test)]
mod test_accounting_handle {
    use std::collections::HashMap;

    use async_session::chrono::{Datelike, Duration, TimeZone, Utc};
    use mongodb::bson::Uuid;

    use crate::{
        accounting_config_logic::FinanceAccountingConfigHandle,
        accounting_database::FinanceAccountBookingEntryListSearchOption,
        accounting_logic::FinanceBookingHandle,
        database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB},
        datatypes::{
            AccountBalanceInfo, AccountBalanceType, BookingEntryType, FinanceAccount,
            FinanceAccountBookingEntry, FinanceAccountType, FinanceBookingRequest,
            FinanceBookingResult, FinanceJournalEntry,
        },
        password_handle::{validate_credentials, UserCredentials},
        setting_struct::TestSettingStruct,
        tests::{
            mocking_database::{InMemoryDatabaseData, InMemoryDatabaseHandler},
            testing_accounting_config,
        },
    };

    #[tokio::test]
    async fn test_accounting_booking_creating_with_mock() {
        /* this test is "just" for testing the creation of entries, for validation of calculation please see other tests */
        let dummy_connection_settings = DbConnectionSetting {
            instance: "".into(),
            password: "".into(),
            url: "".into(),
            user: "".into(),
        };
        let user_id_1 = Uuid::new();
        let user_id_2 = Uuid::new();
        let user_id_3 = Uuid::new();

        let entry_object1 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_1);
        let entry_object2 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_2);

        let _insert_result = InMemoryDatabaseData::insert_in_memory_database(Vec::from([
            entry_object1,
            entry_object2,
        ]));

        let in_memory_db = InMemoryDatabaseHandler {};

        let mut account_handle_1 = FinanceAccountingConfigHandle::new(&user_id_1, &in_memory_db);
        let mut account_handle_2 = FinanceAccountingConfigHandle::new(&user_id_2, &in_memory_db);
        let mut account_handle_3 = FinanceAccountingConfigHandle::new(&user_id_3, &in_memory_db);

        let booking_handle_1 =
            FinanceBookingHandle::new(&dummy_connection_settings, &user_id_1, &in_memory_db);
        let booking_handle_2 =
            FinanceBookingHandle::new(&dummy_connection_settings, &user_id_2, &in_memory_db);
        let booking_handle_3 =
            FinanceBookingHandle::new(&dummy_connection_settings, &user_id_3, &in_memory_db);

        let mut finance_account_type_1_1 = FinanceAccountType {
            description: "SomeTypeDescription_1_1".to_string(),
            title: "SomeType_1_1".to_string(),
            id: Uuid::new(),
        };
        let mut finance_account_type_1_2 = FinanceAccountType {
            description: "SomeTypeDescription_1_1".to_string(),
            title: "SomeType_1_2".to_string(),
            id: Uuid::new(),
        };
        let mut finance_account_type_2_1 = FinanceAccountType {
            description: "SomeTypeDescription_1_1".to_string(),
            title: "SomeType_2_1".to_string(),
            id: Uuid::new(),
        };
        let mut finance_account_type_2_2 = FinanceAccountType {
            description: "SomeTypeDescription_1_1".to_string(),
            title: "SomeType_1_2".to_string(),
            id: Uuid::new(),
        };
        let mut finance_account_type_3_1 = FinanceAccountType {
            description: "SomeTypeDescription_3_1".to_string(),
            title: "SomeType_3_1".to_string(),
            id: Uuid::new(),
        };
        let finance_account_1_1 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_1_1.id,
            title: "account_1_1".into(),
            description: "description_1_1".into(),
        };
        let finance_account_1_2 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_1_2.id,
            title: "account_1_2".into(),
            description: "description_1_2".into(),
        };
        let finance_account_2_1 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_2_1.id,
            title: "account_2_1".into(),
            description: "description_2_1".into(),
        };
        let finance_account_2_2 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_2_2.id,
            title: "account_2_2".into(),
            description: "description_2_2".into(),
        };
        let finance_account_2_3 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_2_2.id,
            title: "account_2_3".into(),
            description: "description_2_3".into(),
        };
        let finance_account_3_1 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_3_1.id,
            title: "account_3_1".into(),
            description: "description_3_1".into(),
        };
        let finance_account_3_2 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_3_1.id,
            title: "account_3_2".into(),
            description: "description_3_2".into(),
        };
        let insert_finance_account_type_1_1_result =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_1_1);
        let insert_finance_account_type_1_2_result =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_1_2);
        let insert_finance_account_type_2_1_result =
            account_handle_2.finance_account_type_upsert(&mut finance_account_type_2_1);
        let insert_finance_account_type_2_2_result =
            account_handle_2.finance_account_type_upsert(&mut finance_account_type_2_2);
        let insert_finance_account_type_3_1_result =
            account_handle_3.finance_account_type_upsert(&mut finance_account_type_3_1);
        let insert_finance_account_1_1_result =
            account_handle_1.finance_account_upsert(&finance_account_1_1);
        let insert_finance_account_1_2_result =
            account_handle_1.finance_account_upsert(&finance_account_1_2);
        let insert_finance_account_2_1_result =
            account_handle_2.finance_account_upsert(&finance_account_2_1);
        let insert_finance_account_2_2_result =
            account_handle_2.finance_account_upsert(&finance_account_2_2);
        let insert_finance_account_2_3_result =
            account_handle_2.finance_account_upsert(&finance_account_2_3);
        let insert_finance_account_3_1_result =
            account_handle_3.finance_account_upsert(&finance_account_3_1);
        let insert_finance_account_3_2_result =
            account_handle_3.finance_account_upsert(&finance_account_3_2);
        assert!(
            insert_finance_account_type_1_1_result.is_ok(),
            "{}",
            insert_finance_account_type_1_1_result.unwrap_err()
        );
        assert!(
            insert_finance_account_type_1_2_result.is_ok(),
            "{}",
            insert_finance_account_type_1_2_result.unwrap_err()
        );
        assert!(
            insert_finance_account_type_2_1_result.is_ok(),
            "{}",
            insert_finance_account_type_2_1_result.unwrap_err()
        );
        assert!(
            insert_finance_account_type_2_2_result.is_ok(),
            "{}",
            insert_finance_account_type_2_2_result.unwrap_err()
        );
        assert!(
            insert_finance_account_1_1_result.is_ok(),
            "{}",
            insert_finance_account_1_1_result.unwrap_err()
        );
        assert!(
            insert_finance_account_1_2_result.is_ok(),
            "{}",
            insert_finance_account_1_2_result.unwrap_err()
        );
        assert!(
            insert_finance_account_2_1_result.is_ok(),
            "{}",
            insert_finance_account_2_1_result.unwrap_err()
        );
        assert!(
            insert_finance_account_2_2_result.is_ok(),
            "{}",
            insert_finance_account_2_2_result.unwrap_err()
        );
        assert!(
            insert_finance_account_2_3_result.is_ok(),
            "{}",
            insert_finance_account_2_3_result.unwrap_err()
        );
        assert!(
            insert_finance_account_type_3_1_result.is_err(),
            "command should have failed"
        );
        assert!(
            insert_finance_account_3_1_result.is_err(),
            "command should have failed"
        );
        assert!(
            insert_finance_account_3_2_result.is_err(),
            "command should have failed"
        );

        /* Test 1 create booking entries for 3 users:
        user 1: 2 entires for two account => a to b and b to a with different amounts
        user 2: 3 entries for three account => a to b, b to c, c to b
        user 3: 1 entry for two accounts => a to b,

        checks:
        * creating for user 1 and 2 are successfult
        * creating for user 3 has to fail
        * full listings for user 1 gets the data created for user 1 but not data created for user 2
        * full listings for user 2 gets the data created for user 2 but not data created for user 1
         */

        let booking_time_1 = Utc
            .with_ymd_and_hms(Utc::now().year(), 1, 1, 10, 15, 25)
            .unwrap();
        let booking_time_2 = booking_time_1 + Duration::days(1);
        let booking_time_3 = booking_time_2 + Duration::days(1);
        let booking_time_4 = booking_time_3 + Duration::days(1);
        let booking_time_5 = booking_time_4 + Duration::days(1);
        let booking_time_6 = booking_time_5 + Duration::days(1);
        let finance_booking_request_1_1 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_1_1.id,
            credit_finance_account_id: finance_account_1_2.id,
            booking_time: booking_time_1,
            amount: 100,
            title: "f_b_r_1_1".into(),
            description: "description_f_b_r_1_1".into(),
        };
        let finance_booking_request_1_2 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_booking_request_1_1.credit_finance_account_id,
            credit_finance_account_id: finance_booking_request_1_1.debit_finance_account_id,
            booking_time: booking_time_2,
            amount: finance_booking_request_1_1.amount + 1,
            title: "f_b_r_1_2".into(),
            description: "description_f_b_r_1_2".into(),
        };
        let finance_booking_request_2_1 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_1.id,
            credit_finance_account_id: finance_account_2_2.id,
            booking_time: booking_time_3,
            amount: 100,
            title: "f_b_r_2_1".into(),
            description: "description_f_b_r_2_1".into(),
        };
        let finance_booking_request_2_2 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_2.id,
            credit_finance_account_id: finance_account_2_3.id,
            booking_time: booking_time_4,
            amount: finance_booking_request_2_1.amount + 1,
            title: "f_b_r_2_2".into(),
            description: "description_f_b_r_2_2".into(),
        };
        let finance_booking_request_2_3 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_3.id,
            credit_finance_account_id: finance_account_2_1.id,
            booking_time: booking_time_2,
            amount: finance_booking_request_2_2.amount + 1,
            title: "f_b_r_2_3".into(),
            description: "description_f_b_r_2_3".into(),
        };
        let finance_booking_request_3_1 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_3_1.id,
            credit_finance_account_id: finance_account_3_2.id,
            booking_time: booking_time_6,
            amount: 100,
            title: "f_b_r_3_1".into(),
            description: "description_f_b_r_3_1".into(),
        };
        let insert_finance_booking_request_1_1_result = booking_handle_1
            .finance_insert_booking_entry(&finance_booking_request_1_1)
            .await;
        let insert_finance_booking_request_1_2_result = booking_handle_1
            .finance_insert_booking_entry(&finance_booking_request_1_2)
            .await;
        let insert_finance_booking_request_2_1_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_1)
            .await;
        let insert_finance_booking_request_2_2_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_2)
            .await;
        let insert_finance_booking_request_2_3_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_3)
            .await;
        let insert_finance_booking_request_3_1_result = booking_handle_3
            .finance_insert_booking_entry(&finance_booking_request_3_1)
            .await;

        assert!(
            insert_finance_booking_request_1_1_result.is_ok(),
            "{}",
            insert_finance_booking_request_1_1_result.unwrap_err()
        );
        assert!(
            insert_finance_booking_request_1_2_result.is_ok(),
            "{}",
            insert_finance_booking_request_1_2_result.unwrap_err()
        );
        assert!(
            insert_finance_booking_request_2_1_result.is_ok(),
            "{}",
            insert_finance_booking_request_2_1_result.unwrap_err()
        );
        assert!(
            insert_finance_booking_request_2_2_result.is_ok(),
            "{}",
            insert_finance_booking_request_2_2_result.unwrap_err()
        );
        assert!(
            insert_finance_booking_request_2_3_result.is_ok(),
            "{}",
            insert_finance_booking_request_2_3_result.unwrap_err()
        );
        assert!(
            insert_finance_booking_request_3_1_result.is_err(),
            "inserting booking request for unkown userdid not fail"
        );

        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_1_1,
                &insert_finance_booking_request_1_1_result.unwrap()
            ),
            ""
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_1_2,
                &insert_finance_booking_request_1_2_result.unwrap()
            ),
            ""
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_2_1,
                &insert_finance_booking_request_2_1_result.unwrap()
            ),
            ""
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_2_2,
                &insert_finance_booking_request_2_2_result.unwrap()
            ),
            ""
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_2_3,
                &insert_finance_booking_request_2_3_result.unwrap()
            ),
            ""
        );

        let full_listing_user_1_result = booking_handle_1.list_journal_entries(None, None).await;
        let full_listing_user_2_result = booking_handle_2.list_journal_entries(None, None).await;
        assert!(
            full_listing_user_1_result.is_ok(),
            "{}",
            full_listing_user_1_result.unwrap_err()
        );
        assert!(
            full_listing_user_2_result.is_ok(),
            "{}",
            full_listing_user_2_result.unwrap_err()
        );
        let full_listing_user_1 = full_listing_user_1_result.unwrap();
        let full_listing_user_2 = full_listing_user_2_result.unwrap();
        assert_eq!(full_listing_user_1.len(), 2);
        assert_eq!(full_listing_user_2.len(), 3);
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1,
            &finance_booking_request_1_1
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1,
            &finance_booking_request_1_2
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_2,
            &finance_booking_request_2_1
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_2,
            &finance_booking_request_2_2
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_2,
            &finance_booking_request_2_3
        ));

        /* Test 2 increasing running number
        insert another entry for user 1

        checks:
        in full listing before new entry the biggest running booking number is lower than biggest running number in full listing after insert new booking entry
        */
        let booking_time_7 = booking_time_6 + Duration::days(1);
        let finance_booking_request_1_3 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_booking_request_1_1.credit_finance_account_id,
            credit_finance_account_id: finance_booking_request_1_1.debit_finance_account_id,
            booking_time: booking_time_7,
            amount: finance_booking_request_1_2.amount + 1,
            title: "f_b_r_1_3".into(),
            description: "description_f_b_r_1_3".into(),
        };
        let insert_finance_booking_request_1_3_result = booking_handle_1
            .finance_insert_booking_entry(&finance_booking_request_1_3)
            .await;
        assert!(
            insert_finance_booking_request_1_3_result.is_ok(),
            "{}",
            insert_finance_booking_request_1_3_result.unwrap_err()
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_1_3,
                &insert_finance_booking_request_1_3_result.unwrap()
            ),
            ""
        );
        let full_listing_user_1_2_result = booking_handle_1.list_journal_entries(None, None).await;
        let full_listing_user_1_2 = full_listing_user_1_2_result.unwrap();
        assert_eq!(full_listing_user_1_2.len(), 3);
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_2,
            &finance_booking_request_1_1
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_2,
            &finance_booking_request_1_2
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_2,
            &finance_booking_request_1_3
        ));

        let running_number_1 = get_max_running_number_from_journal_list(&full_listing_user_1);
        let running_number_2 = get_max_running_number_from_journal_list(&full_listing_user_1_2);
        assert!(
            running_number_2 > running_number_1,
            "new running must be greater than old running number"
        );

        /* Test 3 Filtering options
        using datetime filters to limit results using
        a) just from dateime
        b) just till datetime
        c) using from and till datetime

        checks: listings only have the limited results
        */
        let check_date_time_1 = booking_time_2 - Duration::seconds(1);
        let check_date_time_2 = booking_time_7 - Duration::seconds(1);
        let full_listing_user_1_3_result = booking_handle_1
            .list_journal_entries(Some(check_date_time_1), None)
            .await;
        let full_listing_user_1_4_result = booking_handle_1
            .list_journal_entries(None, Some(check_date_time_2))
            .await;
        let full_listing_user_1_5_result = booking_handle_1
            .list_journal_entries(Some(check_date_time_1), Some(check_date_time_2))
            .await;
        assert!(
            full_listing_user_1_3_result.is_ok(),
            "{}",
            full_listing_user_1_3_result.unwrap_err()
        );
        assert!(
            full_listing_user_1_4_result.is_ok(),
            "{}",
            full_listing_user_1_4_result.unwrap_err()
        );
        assert!(
            full_listing_user_1_5_result.is_ok(),
            "{}",
            full_listing_user_1_5_result.unwrap_err()
        );
        let full_listing_user_1_3 = full_listing_user_1_3_result.unwrap();
        let full_listing_user_1_4 = full_listing_user_1_4_result.unwrap();
        let full_listing_user_1_5 = full_listing_user_1_5_result.unwrap();
        assert_eq!(full_listing_user_1_3.len(), 2);
        assert_eq!(full_listing_user_1_4.len(), 2);
        assert_eq!(full_listing_user_1_5.len(), 1);
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_3,
            &finance_booking_request_1_2
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_3,
            &finance_booking_request_1_3
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_4,
            &finance_booking_request_1_1
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_4,
            &finance_booking_request_1_2
        ));
        assert!(check_journal_listing_contains_booking_request(
            &full_listing_user_1_5,
            &finance_booking_request_1_2
        ));

        /* Test 4 testing filtering for account listing
         a) no filtering
         b) just from
         c) just till
         d) using from and till
        */
        let finance_account_1_1_listing_1_result = booking_handle_1
            .list_account_booking_entries(vec![FinanceAccountBookingEntryListSearchOption::new(
                &finance_account_1_1.id,
                None,
                None,
            )])
            .await;
        let finance_account_1_1_listing_2_result = booking_handle_1
            .list_account_booking_entries(vec![FinanceAccountBookingEntryListSearchOption::new(
                &finance_account_1_1.id,
                Some(check_date_time_1),
                None,
            )])
            .await;
        let finance_account_1_1_listing_3_result = booking_handle_1
            .list_account_booking_entries(vec![FinanceAccountBookingEntryListSearchOption::new(
                &finance_account_1_1.id,
                None,
                Some(check_date_time_2),
            )])
            .await;
        let finance_account_1_1_listing_4_result = booking_handle_1
            .list_account_booking_entries(vec![FinanceAccountBookingEntryListSearchOption::new(
                &finance_account_1_1.id,
                Some(check_date_time_1),
                Some(check_date_time_2),
            )])
            .await;
        assert!(
            finance_account_1_1_listing_1_result.is_ok(),
            "{}",
            finance_account_1_1_listing_1_result.unwrap_err()
        );
        assert!(
            finance_account_1_1_listing_2_result.is_ok(),
            "{}",
            finance_account_1_1_listing_2_result.unwrap_err()
        );
        assert!(
            finance_account_1_1_listing_3_result.is_ok(),
            "{}",
            finance_account_1_1_listing_3_result.unwrap_err()
        );
        assert!(
            finance_account_1_1_listing_4_result.is_ok(),
            "{}",
            finance_account_1_1_listing_4_result.unwrap_err()
        );
        let finance_account_1_1_listing_1 = finance_account_1_1_listing_1_result.unwrap();
        let finance_account_1_1_listing_2 = finance_account_1_1_listing_2_result.unwrap();
        let finance_account_1_1_listing_3 = finance_account_1_1_listing_3_result.unwrap();
        let finance_account_1_1_listing_4 = finance_account_1_1_listing_4_result.unwrap();
        assert_eq!(finance_account_1_1_listing_1.len(), 3);
        assert_eq!(finance_account_1_1_listing_2.len(), 2);
        assert_eq!(finance_account_1_1_listing_3.len(), 2);
        assert_eq!(finance_account_1_1_listing_4.len(), 1);
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_1,
                &finance_booking_request_1_1
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_1,
                &finance_booking_request_1_2
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_1,
                &finance_booking_request_1_3
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_2,
                &finance_booking_request_1_2
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_2,
                &finance_booking_request_1_3
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_3,
                &finance_booking_request_1_1
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_3,
                &finance_booking_request_1_2
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &finance_account_1_1_listing_4,
                &finance_booking_request_1_2
            ),
            ""
        );
        //multi search
        let filter_option_1 = FinanceAccountBookingEntryListSearchOption::new(
            &finance_account_2_1.id,
            None,
            Some(booking_time_2),
        );
        let filter_option_2 = FinanceAccountBookingEntryListSearchOption::new(
            &finance_account_2_2.id,
            Some(booking_time_4),
            None,
        );
        let multi_account_list_result = booking_handle_2
            .list_account_booking_entries(vec![filter_option_1, filter_option_2])
            .await;
        assert!(
            multi_account_list_result.is_ok(),
            "{}",
            multi_account_list_result.unwrap_err()
        );
        let multi_account_list = multi_account_list_result.unwrap();
        assert_eq!(multi_account_list.len(), 2);
        assert_eq!(
            check_account_listing_contains_booking_request(
                &multi_account_list,
                &finance_booking_request_2_3
            ),
            ""
        );
        assert_eq!(
            check_account_listing_contains_booking_request(
                &multi_account_list,
                &finance_booking_request_2_2
            ),
            ""
        );

        /* Test 6 further invalid operations
        trying to perform invalid operations
        a) using datetime filtering where till datetime is before from datetime
            a1) for journal entries
            a2) for account entries
        b) insert a booking entry with a booking time already presents
            b1) for credit account
            b2) for debit account
        c) using a account from another user
            c1) for credit
            c2) for debit
        d) inserting a booking request before latest saldo entry
            d1) using debit account before debit saldo
            d2) using debit account before credit saldo
            d3) using credit account before debit saldo
            d4) using debit account before credit saldo
        e) listing account entries from a different user
        f) multi search with empty insert
        */
        let full_listing_user_1_6_result = booking_handle_1
            .list_journal_entries(Some(check_date_time_2), Some(check_date_time_1))
            .await;
        assert!(
            full_listing_user_1_6_result.is_err(),
            "filtering with date till before date from must fail"
        );
        let finance_account_1_1_listing_5_result = booking_handle_1
            .list_account_booking_entries(vec![FinanceAccountBookingEntryListSearchOption::new(
                &finance_account_1_1.id,
                Some(check_date_time_2),
                Some(check_date_time_1),
            )])
            .await;
        assert!(
            finance_account_1_1_listing_5_result.is_err(),
            "filtering with date till before date from must fail"
        );
        let finance_account_2_4 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_2_2.id,
            title: "account_2_4".into(),
            description: "description_2_4".into(),
        };
        let insert_finance_account_2_4_result =
            account_handle_2.finance_account_upsert(&finance_account_2_4);
        assert!(
            insert_finance_account_2_4_result.is_ok(),
            "{}",
            insert_finance_account_2_4_result.unwrap_err()
        );
        let finance_booking_request_2_4 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_4.id,
            credit_finance_account_id: finance_account_2_2.id,
            booking_time: booking_time_3,
            amount: 117,
            title: "f_b_r_2_4".into(),
            description: "description_f_b_r_2_4".into(),
        };
        let insert_finance_booking_request_2_4_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_4)
            .await;
        assert!(
            insert_finance_booking_request_2_4_result.is_err(),
            "inserting booking request for credit account with same booking time twice must fail"
        );

        let finance_booking_request_2_5 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_1.id,
            credit_finance_account_id: finance_account_2_4.id,
            booking_time: booking_time_3,
            amount: 119,
            title: "f_b_r_2_5".into(),
            description: "description_f_b_r_2_5".into(),
        };
        let insert_finance_booking_request_2_5_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_5)
            .await;
        assert!(
            insert_finance_booking_request_2_5_result.is_err(),
            "inserting booking request for debit account with same booking time twice must fail"
        );

        let booking_time_8: async_session::chrono::DateTime<Utc> =
            booking_time_7 + Duration::days(1);
        let finance_booking_request_2_6 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_4.id,
            credit_finance_account_id: finance_account_1_1.id,
            booking_time: booking_time_8,
            amount: 127,
            title: "f_b_r_2_6".into(),
            description: "description_f_b_r_2_6".into(),
        };
        let insert_finance_booking_request_2_6_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_6)
            .await;
        assert!(
            insert_finance_booking_request_2_6_result.is_err(),
            "inserting booking request for credit account from another user must fail"
        );

        let finance_booking_request_2_7 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_1_1.id,
            credit_finance_account_id: finance_account_2_4.id,
            booking_time: booking_time_8,
            amount: 127,
            title: "f_b_r_2_7".into(),
            description: "description_f_b_r_2_7".into(),
        };
        let insert_finance_booking_request_2_7_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_7)
            .await;
        assert!(
            insert_finance_booking_request_2_7_result.is_err(),
            "inserting booking request for debit account from another user must fail"
        );

        let booking_time_9: async_session::chrono::DateTime<Utc> =
            booking_time_8 + Duration::days(1);
        let finance_booking_request_2_8 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: true,
            debit_finance_account_id: finance_account_2_1.id,
            credit_finance_account_id: finance_account_2_2.id,
            booking_time: booking_time_9,
            amount: 127,
            title: "f_b_r_2_8".into(),
            description: "description_f_b_r_2_8".into(),
        };
        let insert_finance_booking_request_2_8_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_8)
            .await;
        assert!(
            insert_finance_booking_request_2_8_result.is_ok(),
            "could not prepare saldo check: {}",
            insert_finance_booking_request_2_8_result.unwrap_err()
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_2_8,
                &insert_finance_booking_request_2_8_result.unwrap()
            ),
            ""
        );

        let saldo_error_text = "Can not insert before saldo";
        let booking_time_10: async_session::chrono::DateTime<Utc> =
            booking_time_9 - Duration::hours(1);
        let finance_booking_request_2_9 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_1.id,
            credit_finance_account_id: finance_account_2_3.id,
            booking_time: booking_time_10,
            amount: 127,
            title: "f_b_r_2_9".into(),
            description: "description_f_b_r_2_9".into(),
        };
        let insert_finance_booking_request_2_9_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_9)
            .await;
        assert!(
            insert_finance_booking_request_2_9_result.is_err(),
            "operation should have failed"
        );
        assert!(insert_finance_booking_request_2_9_result
            .unwrap_err()
            .to_string()
            .contains(saldo_error_text));

        let finance_booking_request_2_10 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_3.id,
            credit_finance_account_id: finance_account_2_1.id,
            booking_time: booking_time_10,
            amount: 127,
            title: "f_b_r_2_10".into(),
            description: "description_f_b_r_2_10".into(),
        };
        let insert_finance_booking_request_2_10_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_10)
            .await;
        assert!(
            insert_finance_booking_request_2_10_result.is_err(),
            "operation should have failed"
        );
        assert!(insert_finance_booking_request_2_10_result
            .unwrap_err()
            .to_string()
            .contains(saldo_error_text));

        let finance_booking_request_2_11 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_2.id,
            credit_finance_account_id: finance_account_2_3.id,
            booking_time: booking_time_10,
            amount: 127,
            title: "f_b_r_2_11".into(),
            description: "description_f_b_r_2_11".into(),
        };
        let insert_finance_booking_request_2_11_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_11)
            .await;
        assert!(
            insert_finance_booking_request_2_11_result.is_err(),
            "operation should have failed"
        );
        assert!(insert_finance_booking_request_2_11_result
            .unwrap_err()
            .to_string()
            .contains(saldo_error_text));

        let finance_booking_request_2_12 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_2_3.id,
            credit_finance_account_id: finance_account_2_2.id,
            booking_time: booking_time_10,
            amount: 127,
            title: "f_b_r_2_12".into(),
            description: "description_f_b_r_2_12".into(),
        };
        let insert_finance_booking_request_2_12_result = booking_handle_2
            .finance_insert_booking_entry(&finance_booking_request_2_12)
            .await;
        assert!(
            insert_finance_booking_request_2_12_result.is_err(),
            "operation should have failed"
        );
        assert!(insert_finance_booking_request_2_12_result
            .unwrap_err()
            .to_string()
            .contains(saldo_error_text));

        let finance_account_1_1_listing_6_result = booking_handle_1
            .list_account_booking_entries(vec![FinanceAccountBookingEntryListSearchOption::new(
                &finance_account_2_1.id,
                None,
                None,
            )])
            .await;
        assert!(
            finance_account_1_1_listing_6_result.is_err(),
            "operation should have failed"
        );

        let empty_search_options = Vec::new();
        let finance_empty_listing_result = booking_handle_1
            .list_account_booking_entries(empty_search_options)
            .await;
        assert!(
            finance_empty_listing_result.is_err(),
            "query with empty search options needs to fail"
        );
        assert!(finance_empty_listing_result
            .unwrap_err()
            .contains("could not query because search options is empty"));
    }

    #[tokio::test]
    async fn test_accounting_booking_calculate_with_mock() {
        let dummy_connection_settings = DbConnectionSetting {
            instance: "".into(),
            password: "".into(),
            url: "".into(),
            user: "".into(),
        };
        let user_id_1 = Uuid::new();

        let entry_object1 =
            InMemoryDatabaseData::create_in_memory_database_entry_object(&user_id_1);

        let _insert_result =
            InMemoryDatabaseData::insert_in_memory_database(Vec::from([entry_object1]));

        let in_memory_db = InMemoryDatabaseHandler {};

        let mut account_handle_1 = FinanceAccountingConfigHandle::new(&user_id_1, &in_memory_db);

        let booking_handle_1 =
            FinanceBookingHandle::new(&dummy_connection_settings, &user_id_1, &in_memory_db);

        let mut finance_account_type_1_1 = FinanceAccountType {
            description: "SomeTypeDescription_1_1".to_string(),
            title: "SomeType_1_1".to_string(),
            id: Uuid::new(),
        };
        let mut finance_account_type_1_2 = FinanceAccountType {
            description: "SomeTypeDescription_1_1".to_string(),
            title: "SomeType_1_2".to_string(),
            id: Uuid::new(),
        };

        let insert_finance_account_type_1_1_result =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_1_1);
        let insert_finance_account_type_1_2_result =
            account_handle_1.finance_account_type_upsert(&mut finance_account_type_1_2);

        let finance_account_1_1 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_1_1.id,
            title: "account_1_1".into(),
            description: "description_1_1".into(),
        };
        let finance_account_1_2 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_1_2.id,
            title: "account_1_2".into(),
            description: "description_1_2".into(),
        };
        let finance_account_1_3 = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: finance_account_type_1_2.id,
            title: "account_1_3".into(),
            description: "description_1_3".into(),
        };

        let insert_finance_account_1_1_result =
            account_handle_1.finance_account_upsert(&finance_account_1_1);
        let insert_finance_account_1_2_result =
            account_handle_1.finance_account_upsert(&finance_account_1_2);
        let insert_finance_account_1_3_result =
            account_handle_1.finance_account_upsert(&finance_account_1_3);

        assert!(
            insert_finance_account_type_1_1_result.is_ok(),
            "{}",
            insert_finance_account_type_1_1_result.unwrap_err()
        );
        assert!(
            insert_finance_account_type_1_2_result.is_ok(),
            "{}",
            insert_finance_account_type_1_2_result.unwrap_err()
        );

        assert!(
            insert_finance_account_1_1_result.is_ok(),
            "{}",
            insert_finance_account_1_1_result.unwrap_err()
        );
        assert!(
            insert_finance_account_1_2_result.is_ok(),
            "{}",
            insert_finance_account_1_2_result.unwrap_err()
        );
        assert!(
            insert_finance_account_1_3_result.is_ok(),
            "{}",
            insert_finance_account_1_3_result.unwrap_err()
        );

        let amount_1 = 17;
        let amount_2 = 23;
        let amount_3 = 41;

        let booking_time_1 = Utc
            .with_ymd_and_hms(Utc::now().year(), 1, 1, 10, 15, 25)
            .unwrap();
        let booking_time_2 = booking_time_1 + Duration::days(1);
        let booking_time_3 = booking_time_2 + Duration::days(1);

        let finance_booking_request_1_1 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_1_1.id,
            credit_finance_account_id: finance_account_1_2.id,
            booking_time: booking_time_1,
            amount: amount_1,
            title: "f_b_r_1_1".into(),
            description: "description_f_b_r_1_1".into(),
        };
        let finance_booking_request_1_2 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_1_1.id,
            credit_finance_account_id: finance_account_1_3.id,
            booking_time: booking_time_2,
            amount: amount_2,
            title: "f_b_r_1_2".into(),
            description: "description_f_b_r_1_2".into(),
        };
        let finance_booking_request_1_3 = FinanceBookingRequest {
            is_simple_entry: true,
            is_saldo: false,
            debit_finance_account_id: finance_account_1_2.id,
            credit_finance_account_id: finance_account_1_3.id,
            booking_time: booking_time_3,
            amount: amount_3,
            title: "f_b_r_1_3".into(),
            description: "description_f_b_r_1_3".into(),
        };
        let account_1_running_saldo_amount = amount_1 + amount_2;
        let account_1_running_saldo_type = AccountBalanceType::Debit;
        let account_2_running_saldo_amount = amount_1.abs_diff(amount_3);
        let account_2_running_saldo_type = AccountBalanceType::Debit;
        let account_3_running_saldo_amount = amount_2 + amount_3;
        let account_3_running_saldo_type = AccountBalanceType::Credit;

        let insert_finance_booking_request_1_1_result = booking_handle_1
            .finance_insert_booking_entry(&finance_booking_request_1_1)
            .await;
        let insert_finance_booking_request_1_2_result = booking_handle_1
            .finance_insert_booking_entry(&finance_booking_request_1_2)
            .await;
        let insert_finance_booking_request_1_3_result = booking_handle_1
            .finance_insert_booking_entry(&finance_booking_request_1_3)
            .await;
        assert!(
            insert_finance_booking_request_1_1_result.is_ok(),
            "{}",
            insert_finance_booking_request_1_1_result.unwrap_err()
        );
        assert!(
            insert_finance_booking_request_1_2_result.is_ok(),
            "{}",
            insert_finance_booking_request_1_2_result.unwrap_err()
        );
        assert!(
            insert_finance_booking_request_1_3_result.is_ok(),
            "{}",
            insert_finance_booking_request_1_3_result.unwrap_err()
        );

        let balance_account_1_result: Result<Vec<AccountBalanceInfo>, String> = booking_handle_1
            .calculate_balance_info(&vec![finance_account_1_1.id])
            .await;
        let balance_account_2_result: Result<Vec<AccountBalanceInfo>, String> = booking_handle_1
            .calculate_balance_info(&vec![finance_account_1_2.id])
            .await;
        let balance_account_3_result = booking_handle_1
            .calculate_balance_info(&vec![finance_account_1_3.id])
            .await;

        assert!(
            balance_account_1_result.is_ok(),
            "{}",
            balance_account_1_result.unwrap_err()
        );
        assert!(
            balance_account_2_result.is_ok(),
            "{}",
            balance_account_2_result.unwrap_err()
        );
        assert!(
            balance_account_3_result.is_ok(),
            "{}",
            balance_account_3_result.unwrap_err()
        );

        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_1_1,
                &insert_finance_booking_request_1_1_result.unwrap()
            ),
            ""
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_1_2,
                &insert_finance_booking_request_1_2_result.unwrap()
            ),
            ""
        );
        assert_eq!(
            check_entry_response_match_entry_request(
                &finance_booking_request_1_3,
                &insert_finance_booking_request_1_3_result.unwrap()
            ),
            ""
        );

        let balance_account_1_info = balance_account_1_result.unwrap();
        let balance_account_2_info = balance_account_2_result.unwrap();
        let balance_account_3_info = balance_account_3_result.unwrap();
        assert_eq!(balance_account_1_info.len(), 1);
        assert_eq!(balance_account_2_info.len(), 1);
        assert_eq!(balance_account_3_info.len(), 1);
        assert_eq!(
            check_balance_account_info(
                &balance_account_1_info[0],
                &finance_account_1_1.id,
                &account_1_running_saldo_amount,
                &account_1_running_saldo_type
            ),
            ""
        );
        assert_eq!(
            check_balance_account_info(
                &balance_account_2_info[0],
                &finance_account_1_2.id,
                &account_2_running_saldo_amount,
                &account_2_running_saldo_type
            ),
            ""
        );
        assert_eq!(
            check_balance_account_info(
                &balance_account_3_info[0],
                &finance_account_1_3.id,
                &account_3_running_saldo_amount,
                &account_3_running_saldo_type
            ),
            ""
        );
    }

    #[tokio::test]
    async fn test_accounting_booking_with_mongodb() {
        testing_accounting_config::test_accounting_handle::init();
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
        let mongo_db = DbHandlerMongoDB::new(&db_connection);

        let account_handle_1 = FinanceAccountingConfigHandle::new(&user_id_1, &mongo_db);
        let booking_handle_1 = FinanceBookingHandle::new(&db_connection, &user_id_1, &mongo_db);

        let repair_result = mongo_db
            .repair_counter_record_for_user(&db_connection, &user_id_1)
            .await;
        assert!(repair_result.is_ok(), "{}", repair_result.unwrap_err());
        let _ = prepare_mongo_db_for_tests(&account_handle_1, &booking_handle_1, &db_connection);

        /* Test 0, further requirements
         * ensure that at least 4 finance accounts are available, at least 2 accounta with debit balance, at least 2 accounts with debit balance
         * get balance information for each account
         * get last booking entry for each account
         */
        let accounts_per_user_result = account_handle_1.finance_account_list_async(None).await;
        assert!(
            accounts_per_user_result.is_ok(),
            "{}",
            accounts_per_user_result.unwrap_err()
        );
        let accounts_per_user = accounts_per_user_result.unwrap();

        let account_ids: Vec<Uuid> = accounts_per_user.iter().map(|elem| elem.id).collect();
        let balance_info_all_accounts_result =
            booking_handle_1.calculate_balance_info(&account_ids).await;
        assert!(
            balance_info_all_accounts_result.is_ok(),
            "{}",
            balance_info_all_accounts_result.unwrap_err()
        );
        let balance_info = balance_info_all_accounts_result.unwrap();

        let debit_accounts_info: Vec<&AccountBalanceInfo> = balance_info
            .iter()
            .filter(|elem| elem.balance_type.eq(&AccountBalanceType::Debit))
            .collect();
        let credit_accounts_info: Vec<&AccountBalanceInfo> = balance_info
            .iter()
            .filter(|elem| elem.balance_type.eq(&AccountBalanceType::Credit))
            .collect();
        assert!(
            debit_accounts_info.len() > 1,
            "Not enought accounts with debit balance"
        );
        assert!(
            credit_accounts_info.len() > 1,
            "Not enought accounts with credit balance"
        );

        let test_account_a_balance_info = debit_accounts_info[0];
        let test_account_b_balance_info = debit_accounts_info[1];
        let test_account_c_balance_info = credit_accounts_info[0];
        let test_account_d_balance_info = credit_accounts_info[1];
        let list_test_accounts_ids = vec![
            test_account_a_balance_info.account_id,
            test_account_b_balance_info.account_id,
            test_account_c_balance_info.account_id,
            test_account_d_balance_info.account_id,
        ];
        let mut hashmap_last_booking_entries_per_account = HashMap::new();
        let mut max_booking_time = Utc
            .with_ymd_and_hms(Utc::now().year(), 1, 1, 10, 15, 25)
            .unwrap();
        for account_id in list_test_accounts_ids.iter() {
            let account_entries_result = booking_handle_1
                .list_account_booking_entries(vec![
                    FinanceAccountBookingEntryListSearchOption::new(&account_id, None, None),
                ])
                .await;
            assert!(
                account_entries_result.is_ok(),
                "{}",
                account_entries_result.unwrap_err()
            );
            let account_entries = account_entries_result.unwrap();
            let latest_pos_option = account_entries.iter().max_by_key(|elem| elem.booking_time);
            if latest_pos_option.is_some() {
                let latest_pos = latest_pos_option.unwrap();
                hashmap_last_booking_entries_per_account.insert(account_id, latest_pos.clone());
                if latest_pos.booking_time.gt(&max_booking_time) {
                    max_booking_time = latest_pos.booking_time;
                }
            }
        }

        /* Test 1, change balance, credit to debit
        select 4 accounts A, B, C and D, so that a and B have a debit balance, C and D a credit balance (see above)
        insert amount 1 with A to B
        check balance information for all 4 accounts
        insert amount 2 with C to D
        check balance information for all 4 accounts
        insert amount 3 with B to C, so that balance type switches
        check balance information  for all 4 accounts
        insert amount 4 with A to D, so that balance type switches
        check balance information for all 4 accounts
         */
        let test_run_id = Uuid::new().to_string();
        let booking_time_1 = max_booking_time + Duration::hours(1);
        let amount_a_b = std::cmp::min(
            test_account_a_balance_info.amount,
            test_account_b_balance_info.amount,
        ) / 2;
        let insert_request_a_b = FinanceBookingRequest {
            amount: amount_a_b,
            booking_time: booking_time_1,
            credit_finance_account_id: test_account_a_balance_info.account_id,
            debit_finance_account_id: test_account_b_balance_info.account_id,
            description: format!("A to B: {}, test run {}", amount_a_b, test_run_id),
            title: "A to B".into(),
            is_saldo: false,
            is_simple_entry: true,
        };
        let insert_request_a_b_result = booking_handle_1
            .finance_insert_booking_entry(&insert_request_a_b)
            .await;
        assert!(
            insert_request_a_b_result.is_ok(),
            "{}",
            insert_request_a_b_result.unwrap_err()
        );
        let balance_info_1_accounts_result = booking_handle_1
            .calculate_balance_info(&list_test_accounts_ids)
            .await;
        assert!(
            balance_info_1_accounts_result.is_ok(),
            "{}",
            balance_info_1_accounts_result.unwrap_err()
        );
        let test_account_a_balance_info_1 = AccountBalanceInfo {
            account_id: test_account_a_balance_info.account_id,
            amount: test_account_a_balance_info.amount.abs_diff(amount_a_b),
            balance_type: test_account_a_balance_info.balance_type.clone(),
        };
        let test_account_b_balance_info_1 = AccountBalanceInfo {
            account_id: test_account_b_balance_info.account_id,
            amount: test_account_b_balance_info.amount + amount_a_b,
            balance_type: test_account_b_balance_info.balance_type.clone(),
        };
        let balance_check_1_result = check_multiple_balance_info(
            &balance_info_1_accounts_result.unwrap(),
            &vec![
                &test_account_a_balance_info_1,
                &test_account_b_balance_info_1,
                test_account_c_balance_info,
                test_account_d_balance_info,
            ],
        );
        assert_eq!(balance_check_1_result, "");

        let booking_time_2 = booking_time_1 + Duration::hours(1);
        let amount_c_d = std::cmp::min(
            test_account_c_balance_info.amount,
            test_account_d_balance_info.amount,
        ) / 2;
        let insert_request_c_d = FinanceBookingRequest {
            amount: amount_c_d,
            booking_time: booking_time_2,
            credit_finance_account_id: test_account_c_balance_info.account_id,
            debit_finance_account_id: test_account_d_balance_info.account_id,
            description: format!("C to D: {}, test run {}", amount_c_d, test_run_id),
            title: "C to D".into(),
            is_saldo: false,
            is_simple_entry: true,
        };
        let insert_request_c_d_result = booking_handle_1
            .finance_insert_booking_entry(&insert_request_c_d)
            .await;
        assert!(
            insert_request_c_d_result.is_ok(),
            "{}",
            insert_request_c_d_result.unwrap_err()
        );
        let balance_info_2_accounts_result = booking_handle_1
            .calculate_balance_info(&list_test_accounts_ids)
            .await;
        assert!(
            balance_info_2_accounts_result.is_ok(),
            "{}",
            balance_info_2_accounts_result.unwrap_err()
        );
        let test_account_c_balance_info_2 = AccountBalanceInfo {
            account_id: test_account_c_balance_info.account_id,
            amount: test_account_c_balance_info.amount + amount_c_d,
            balance_type: test_account_c_balance_info.balance_type.clone(),
        };
        let test_account_d_balance_info_2 = AccountBalanceInfo {
            account_id: test_account_d_balance_info.account_id,
            amount: test_account_d_balance_info.amount.abs_diff(amount_c_d),
            balance_type: test_account_d_balance_info.balance_type.clone(),
        };
        let balance_check_2_result = check_multiple_balance_info(
            &balance_info_2_accounts_result.unwrap(),
            &vec![
                &test_account_a_balance_info_1,
                &test_account_b_balance_info_1,
                &test_account_c_balance_info_2,
                &test_account_d_balance_info_2,
            ],
        );
        assert_eq!(balance_check_2_result, "");

        let booking_time_3 = booking_time_2 + Duration::hours(1);
        let amount_b_c = std::cmp::max(
            test_account_b_balance_info_1.amount,
            test_account_c_balance_info_2.amount,
        ) + 17;
        let insert_request_b_c = FinanceBookingRequest {
            amount: amount_b_c,
            booking_time: booking_time_3,
            credit_finance_account_id: test_account_b_balance_info.account_id,
            debit_finance_account_id: test_account_c_balance_info.account_id,
            description: format!("B to C: {}, test run {}", amount_b_c, test_run_id),
            title: "B to C".into(),
            is_saldo: false,
            is_simple_entry: true,
        };

        let insert_request_b_c_result = booking_handle_1
            .finance_insert_booking_entry(&insert_request_b_c)
            .await;
        assert!(
            insert_request_b_c_result.is_ok(),
            "{}",
            insert_request_b_c_result.unwrap_err()
        );

        let balance_info_3_accounts_result = booking_handle_1
            .calculate_balance_info(&list_test_accounts_ids)
            .await;
        assert!(
            balance_info_3_accounts_result.is_ok(),
            "{}",
            balance_info_3_accounts_result.unwrap_err()
        );
        let test_account_b_balance_info_3 = AccountBalanceInfo {
            account_id: test_account_b_balance_info.account_id,
            amount: test_account_b_balance_info_1.amount.abs_diff(amount_b_c),
            balance_type: AccountBalanceType::Credit,
        };
        let test_account_c_balance_info_3 = AccountBalanceInfo {
            account_id: test_account_c_balance_info.account_id,
            amount: test_account_c_balance_info_2.amount.abs_diff(amount_b_c),
            balance_type: AccountBalanceType::Debit,
        };
        let balance_check_3_result = check_multiple_balance_info(
            &balance_info_3_accounts_result.unwrap(),
            &vec![
                &test_account_a_balance_info_1,
                &test_account_b_balance_info_3,
                &test_account_c_balance_info_3,
                &test_account_d_balance_info_2,
            ],
        );
        assert_eq!(balance_check_3_result, "");

        let booking_time_4 = booking_time_3 + Duration::hours(1);
        let amount_a_d = std::cmp::max(
            test_account_a_balance_info_1.amount,
            test_account_d_balance_info_2.amount,
        ) + 23;
        let insert_request_a_d = FinanceBookingRequest {
            amount: amount_a_d,
            booking_time: booking_time_4,
            credit_finance_account_id: test_account_a_balance_info.account_id,
            debit_finance_account_id: test_account_d_balance_info.account_id,
            description: format!("A to D: {}, test run {}", amount_a_d, test_run_id),
            title: "A to D".into(),
            is_saldo: false,
            is_simple_entry: true,
        };
        let insert_request_a_d_result = booking_handle_1
            .finance_insert_booking_entry(&insert_request_a_d)
            .await;
        assert!(
            insert_request_a_d_result.is_ok(),
            "{}",
            insert_request_a_d_result.unwrap_err()
        );
        let balance_info_4_accounts_result = booking_handle_1
            .calculate_balance_info(&list_test_accounts_ids)
            .await;
        assert!(
            balance_info_4_accounts_result.is_ok(),
            "{}",
            balance_info_4_accounts_result.unwrap_err()
        );
        let test_account_a_balance_info_4 = AccountBalanceInfo {
            account_id: test_account_a_balance_info.account_id,
            amount: test_account_a_balance_info_1.amount.abs_diff(amount_a_d),
            balance_type: AccountBalanceType::Credit,
        };
        let test_account_d_balance_info_4 = AccountBalanceInfo {
            account_id: test_account_d_balance_info.account_id,
            amount: test_account_d_balance_info_2.amount.abs_diff(amount_a_d),
            balance_type: AccountBalanceType::Debit,
        };

        let balance_check_4_result = check_multiple_balance_info(
            &balance_info_4_accounts_result.unwrap(),
            &vec![
                &test_account_a_balance_info_4,
                &test_account_b_balance_info_3,
                &test_account_c_balance_info_3,
                &test_account_d_balance_info_4,
            ],
        );
        assert_eq!(balance_check_4_result, "");

        /* Test 2 creating saldos
        a) inserting a saldo with correct booking type but wrong amount
        b) inserting a saldo with wrong booking type but correct amount
        c) inserting a salo with correct booking type an correct amount
        checks: a) and b) fail, c) succeeds
        not required for now: calculation saldo will be implemented in version 0.0.5
         */

        /* Test 3 inserting bookings aftef and before saldo
        a) after saldo => succeeds
        b) before saldo => fails
        addtional check: get balance info and compare
        not required for now: calculation saldo will be implemented in version 0.0.5
         */

        /* Test 4 further invalid operation
        a) credit and debit on same account
        b) using a no existing accont
        c) multi list with empty search option
        */

        let booking_time_5 = booking_time_4 + Duration::hours(1);
        let amount_a_a = std::cmp::max(
            test_account_a_balance_info_1.amount,
            test_account_d_balance_info_2.amount,
        ) + 23;
        let insert_request_a_a = FinanceBookingRequest {
            amount: amount_a_a,
            booking_time: booking_time_5,
            credit_finance_account_id: test_account_a_balance_info.account_id,
            debit_finance_account_id: test_account_a_balance_info.account_id,
            description: format!("A to A: {}, test run {}", amount_a_a, test_run_id),
            title: "A to A".into(),
            is_saldo: false,
            is_simple_entry: true,
        };
        let insert_request_a_a_response_result = booking_handle_1
            .finance_insert_booking_entry(&insert_request_a_a)
            .await;
        assert!(
            insert_request_a_a_response_result.is_err(),
            "using same account for credit and debit muss fail"
        );

        let invalid_account = FinanceAccount {
            id: Uuid::new(),
            finance_account_type_id: accounts_per_user[0].finance_account_type_id,
            description: format!("description for invalid account, test run {}", test_run_id),
            title: "invalid account".into(),
        };
        let booking_time_6 = booking_time_5 + Duration::hours(1);
        let amount_a_i = std::cmp::max(
            test_account_a_balance_info_1.amount,
            test_account_d_balance_info_2.amount,
        ) + 23;
        let insert_request_a_i = FinanceBookingRequest {
            amount: amount_a_i,
            booking_time: booking_time_6,
            credit_finance_account_id: test_account_a_balance_info.account_id,
            debit_finance_account_id: invalid_account.id,
            description: format!("A to I: {}, test run {}", amount_a_i, test_run_id),
            title: "A to I".into(),
            is_saldo: false,
            is_simple_entry: true,
        };
        let insert_request_a_i_response_result = booking_handle_1
            .finance_insert_booking_entry(&insert_request_a_i)
            .await;
        assert!(
            insert_request_a_i_response_result.is_err(),
            "using invalid account for debit muss fail"
        );

        let booking_time_7 = booking_time_6 + Duration::hours(1);
        let amount_i_a = std::cmp::max(
            test_account_a_balance_info_1.amount,
            test_account_d_balance_info_2.amount,
        ) + 23;
        let insert_request_i_a = FinanceBookingRequest {
            amount: amount_i_a,
            booking_time: booking_time_7,
            credit_finance_account_id: invalid_account.id,
            debit_finance_account_id: test_account_a_balance_info.account_id,
            description: format!("I to A: {}, test run {}", amount_i_a, test_run_id),
            title: "I to A".into(),
            is_saldo: false,
            is_simple_entry: true,
        };
        let insert_request_i_a_response_result = booking_handle_1
            .finance_insert_booking_entry(&insert_request_i_a)
            .await;
        assert!(
            insert_request_i_a_response_result.is_err(),
            "using invalid account for credit muss fail"
        );

        let empty_search_options = Vec::new();
        let finance_empty_listing_result = booking_handle_1
            .list_account_booking_entries(empty_search_options)
            .await;
        assert!(
            finance_empty_listing_result.is_err(),
            "query with empty search options needs to fail"
        );
        assert!(finance_empty_listing_result
            .unwrap_err()
            .contains("could not query because search options is empty"));
    }

    fn check_journal_listing_contains_booking_request(
        list_to_check: &Vec<FinanceJournalEntry>,
        element_to_check: &FinanceBookingRequest,
    ) -> bool {
        let position_option = list_to_check.iter().position(|elem: &FinanceJournalEntry| {
            elem.booking_time.eq(&element_to_check.booking_time)
                && elem
                    .credit_finance_account_id
                    .eq(&element_to_check.credit_finance_account_id)
                && elem
                    .debit_finance_account_id
                    .eq(&element_to_check.debit_finance_account_id)
        });
        if let Some(position) = position_option {
            return (list_to_check[position].amount.eq(&element_to_check.amount))
                && (list_to_check[position]
                    .booking_time
                    .eq(&element_to_check.booking_time))
                && (list_to_check[position]
                    .credit_finance_account_id
                    .eq(&element_to_check.credit_finance_account_id))
                && (list_to_check[position]
                    .debit_finance_account_id
                    .eq(&element_to_check.debit_finance_account_id))
                && (list_to_check[position]
                    .is_saldo
                    .eq(&element_to_check.is_saldo))
                && (list_to_check[position]
                    .is_simple_entry
                    .eq(&element_to_check.is_simple_entry))
                && (list_to_check[position]
                    .description
                    .eq(&element_to_check.description))
                && (list_to_check[position].title.eq(&element_to_check.title));
        }
        return false;
    }

    /* return empty string if all is fine, if not describe watch did not match (detailed messaged only added if required during debugging) */
    fn check_account_listing_contains_booking_request(
        list_to_check: &Vec<FinanceAccountBookingEntry>,
        element_to_check: &FinanceBookingRequest,
    ) -> String {
        let position_option = list_to_check
            .iter()
            .position(|elem: &FinanceAccountBookingEntry| {
                elem.booking_time.eq(&element_to_check.booking_time)
                    && (elem
                        .finance_account_id
                        .eq(&element_to_check.credit_finance_account_id)
                        || elem
                            .finance_account_id
                            .eq(&element_to_check.debit_finance_account_id))
            });
        if let Some(position) = position_option {
            let booking_type_required = if list_to_check[position]
                .finance_account_id
                .eq(&element_to_check.credit_finance_account_id)
            {
                if element_to_check.is_saldo {
                    BookingEntryType::SaldoCredit
                } else {
                    BookingEntryType::Credit
                }
            } else {
                if element_to_check.is_saldo {
                    BookingEntryType::SaldoDebit
                } else {
                    BookingEntryType::Debit
                }
            };

            if list_to_check[position].amount.ne(&element_to_check.amount) {
                return "amount does not match".into();
            }
            if list_to_check[position]
                .booking_time
                .ne(&element_to_check.booking_time)
            {
                return "booking_time does not match".into();
            }
            if list_to_check[position]
                .booking_type
                .ne(&booking_type_required)
            {
                return "booking_type does not match".into();
            }
            if list_to_check[position]
                .description
                .ne(&element_to_check.description)
            {
                return "description does not match".into();
            }
            if list_to_check[position].title.ne(&element_to_check.title) {
                return "title does not match".into();
            }

            return "".into();
        }
        return "no entry for booking time found".into();
    }

    /* return empty string if all is fine, if not describe watch did not match (detailed messaged only added if required during debugging) */
    fn check_entry_response_match_entry_request(
        entry_request: &FinanceBookingRequest,
        entry_response: &FinanceBookingResult,
    ) -> String {
        let credit_booking_type = if entry_request.is_saldo {
            BookingEntryType::SaldoCredit
        } else {
            BookingEntryType::Credit
        };
        let debit_booking_type = if entry_request.is_saldo {
            BookingEntryType::SaldoDebit
        } else {
            BookingEntryType::Debit
        };

        if entry_request
            .is_saldo
            .ne(&entry_response.journal_entry.is_saldo)
        {
            return "is_saldo does not match".into();
        }
        if entry_request
            .is_simple_entry
            .ne(&entry_response.journal_entry.is_simple_entry)
        {
            return "is_simple_entry does not match".into();
        }
        if entry_request
            .amount
            .ne(&entry_response.journal_entry.amount)
        {
            return "amount does not match".into();
        }

        if entry_request
            .booking_time
            .eq(&entry_response.journal_entry.booking_time)
            && entry_request
                .credit_finance_account_id
                .eq(&entry_response.journal_entry.credit_finance_account_id)
            && entry_request
                .debit_finance_account_id
                .eq(&entry_response.journal_entry.debit_finance_account_id)
            && entry_request
                .description
                .eq(&entry_response.journal_entry.description)
            && entry_request.title.eq(&entry_response.journal_entry.title)
        {
            if credit_booking_type.eq(&entry_response.credit_account_entry.booking_type)
                && entry_request
                    .amount
                    .eq(&entry_response.credit_account_entry.amount)
                && entry_request
                    .booking_time
                    .eq(&entry_response.credit_account_entry.booking_time)
                && entry_request
                    .credit_finance_account_id
                    .eq(&entry_response.credit_account_entry.finance_account_id)
                && entry_request
                    .description
                    .eq(&entry_response.credit_account_entry.description)
                && entry_request
                    .title
                    .eq(&entry_response.credit_account_entry.title)
                && entry_response
                    .journal_entry
                    .id
                    .eq(&entry_response.credit_account_entry.finance_journal_diary_id)
            {
                if debit_booking_type.eq(&entry_response.debit_account_entry.booking_type)
                    && entry_request
                        .amount
                        .eq(&entry_response.debit_account_entry.amount)
                    && entry_request
                        .booking_time
                        .eq(&entry_response.debit_account_entry.booking_time)
                    && entry_request
                        .debit_finance_account_id
                        .eq(&entry_response.debit_account_entry.finance_account_id)
                    && entry_request
                        .description
                        .eq(&entry_response.debit_account_entry.description)
                    && entry_request
                        .title
                        .eq(&entry_response.debit_account_entry.title)
                    && entry_response
                        .journal_entry
                        .id
                        .eq(&entry_response.debit_account_entry.finance_journal_diary_id)
                {
                    return "".into();
                }
            }
        }
        return "does not match".into();
    }

    //return "" if all OK, otherwise return what mismatched
    fn check_balance_account_info(
        info_to_check: &AccountBalanceInfo,
        account_id: &Uuid,
        amount: &u64,
        balance_type: &AccountBalanceType,
    ) -> String {
        if account_id.ne(&info_to_check.account_id) {
            return "Account ID missmatched".into();
        }
        if amount.ne(&info_to_check.amount) {
            return "Amount missmatched".into();
        }
        if balance_type.ne(&info_to_check.balance_type) {
            return "Balance Type missmatched".into();
        }

        return "".into();
    }

    fn check_multiple_balance_info(
        balance_info_list_to_check: &Vec<AccountBalanceInfo>,
        balance_info_list_expected: &Vec<&AccountBalanceInfo>,
    ) -> String {
        if balance_info_list_to_check
            .len()
            .ne(&balance_info_list_expected.len())
        {
            return format!(
                "list sizse does not match: {} instead of {}",
                balance_info_list_to_check.len(),
                balance_info_list_expected.len()
            );
        }
        for balance_to_check in balance_info_list_to_check {
            let position_option = balance_info_list_expected
                .iter()
                .position(|elem| elem.account_id.eq(&balance_to_check.account_id));
            if position_option.is_none() {
                return format!(
                    "Could not find expected info for account {}",
                    balance_to_check.account_id
                );
            }
            let balance_expected = balance_info_list_expected[position_option.unwrap()];
            if balance_to_check.amount.ne(&balance_expected.amount) {
                return format!(
                    "amount for account {} does not match: {} instead of {}",
                    balance_to_check.account_id, balance_to_check.amount, balance_expected.amount
                );
            }
            if balance_to_check
                .balance_type
                .ne(&balance_expected.balance_type)
            {
                return format!(
                    "balance type for account {} does not match: {} instead of {}",
                    balance_to_check.account_id,
                    balance_to_check.balance_type,
                    balance_expected.balance_type
                );
            }
        }

        return "".into();
    }

    fn get_max_running_number_from_journal_list(journal_list: &Vec<FinanceJournalEntry>) -> u64 {
        let max_option = journal_list.iter().max_by_key(|elem| elem.running_number);
        if max_option.is_none() {
            return 0;
        }
        return max_option.unwrap().running_number;
    }

    fn prepare_mongo_db_for_tests(
        account_handle_1: &FinanceAccountingConfigHandle<'_>,
        booking_handle_1: &FinanceBookingHandle,
        db_connection: &DbConnectionSetting,
    ) -> bool {
        super::GLOBAL_PREPARED_MONGODB.call_once(|| {
            if !DbHandlerMongoDB::validate_db_structure(&db_connection) {
                panic!("Could not validate backend structure")
            }
            let accounts_per_user_result = account_handle_1.finance_account_list(None);
            assert!(
                accounts_per_user_result.is_ok(),
                "{}",
                accounts_per_user_result.unwrap_err()
            );
            let accounts_per_user = accounts_per_user_result.unwrap();

            let account_ids: Vec<Uuid> = accounts_per_user.iter().map(|elem| elem.id).collect();
            let balance_info_all_accounts_result =
                local_calculate_balance_info_sync(&booking_handle_1, &account_ids);
            assert!(
                balance_info_all_accounts_result.is_ok(),
                "{}",
                balance_info_all_accounts_result.unwrap_err()
            );
            let balance_info = balance_info_all_accounts_result.unwrap();

            let debit_accounts_info: Vec<&AccountBalanceInfo> = balance_info
                .iter()
                .filter(|elem| elem.balance_type.eq(&AccountBalanceType::Debit))
                .collect();
            let credit_accounts_info: Vec<&AccountBalanceInfo> = balance_info
                .iter()
                .filter(|elem| elem.balance_type.eq(&AccountBalanceType::Credit))
                .collect();

            if credit_accounts_info.len() < 2 {
                if debit_accounts_info.len() > 3 {
                    let mut credit_counter = credit_accounts_info.len();
                    while credit_counter < 2 {
                        let index_1 = credit_counter * 2;
                        let index_2 = credit_counter * 2 + 1;
                        let amount_mod = debit_accounts_info[index_1].amount + 100;
                        //panic!("Fore some reason the following line will not return");
                        let saldo_information_list_result = futures::executor::block_on(
                            booking_handle_1.finance_get_last_saldo_account_entries(Some(vec![
                                debit_accounts_info[index_1].account_id,
                                debit_accounts_info[index_2].account_id,
                            ])),
                        );
                        if saldo_information_list_result.is_err() {
                            panic!(
                                "Could not prepare MONGODB: {}",
                                saldo_information_list_result.unwrap_err()
                            )
                        }
                        let saldo_information_list: Vec<FinanceAccountBookingEntry> =
                            saldo_information_list_result
                                .unwrap()
                                .into_values()
                                .collect();
                        let last_saldo_time_option = match saldo_information_list.len() {
                            0 => None,
                            1 => Some(saldo_information_list[0].booking_time),
                            _ => Some(std::cmp::max(
                                saldo_information_list[0].booking_time,
                                saldo_information_list[1].booking_time,
                            )),
                        };
                        let search_options = vec![
                            FinanceAccountBookingEntryListSearchOption::new(
                                &debit_accounts_info[index_1].account_id,
                                last_saldo_time_option,
                                None,
                            ),
                            FinanceAccountBookingEntryListSearchOption::new(
                                &debit_accounts_info[index_2].account_id,
                                last_saldo_time_option,
                                None,
                            ),
                        ];
                        let booking_entries_result = local_list_account_booking_entries_multi_sync(
                            &booking_handle_1,
                            search_options,
                        );
                        if booking_entries_result.is_err() {
                            panic!(
                                "Could not prepare MONGODB: {}",
                                booking_entries_result.unwrap_err()
                            )
                        }
                        let booking_entries = booking_entries_result.unwrap();
                        let max_booking_entry =
                            booking_entries.iter().max_by_key(|elem| elem.booking_time);
                        let update_time = match max_booking_entry {
                            Some(max_elem) => max_elem.booking_time + Duration::hours(1),
                            None => Utc
                                .with_ymd_and_hms(Utc::now().year(), 1, 1, 10, 15, 25)
                                .unwrap(),
                        };

                        let insert_request_mod = FinanceBookingRequest {
                            amount: amount_mod,
                            booking_time: update_time,
                            credit_finance_account_id: debit_accounts_info[index_1].account_id,
                            debit_finance_account_id: debit_accounts_info[index_2].account_id,
                            description: format!("preparing with amount {}", amount_mod),
                            title: "Prepare".into(),
                            is_saldo: false,
                            is_simple_entry: true,
                        };
                        let insert_request_mod_response_result = futures::executor::block_on(
                            booking_handle_1.finance_insert_booking_entry(&insert_request_mod),
                        );

                        if insert_request_mod_response_result.is_err() {
                            panic!(
                                "Could not prepare MONGODB: {}",
                                insert_request_mod_response_result.unwrap_err()
                            )
                        }

                        credit_counter += 1;
                    }
                } else {
                    panic!("Could not prepare for tests: not enough credit accounts")
                }
            }
        });
        return true;
    }

    /* some functions are not called in synced mode in productive code */
    fn local_calculate_balance_info_sync(
        booking_handle_1: &FinanceBookingHandle,
        accounts_to_calculate: &Vec<Uuid>,
    ) -> Result<Vec<AccountBalanceInfo>, String> {
        let return_var = futures::executor::block_on(
            booking_handle_1.calculate_balance_info(accounts_to_calculate),
        );
        return return_var;
    }

    fn local_list_account_booking_entries_multi_sync(
        booking_handle_1: &FinanceBookingHandle,
        search_options: Vec<FinanceAccountBookingEntryListSearchOption>,
    ) -> Result<Vec<FinanceAccountBookingEntry>, String> {
        let return_var = futures::executor::block_on(
            booking_handle_1.list_account_booking_entries(search_options),
        );
        return return_var;
    }
}
