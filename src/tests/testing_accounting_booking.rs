#[cfg(test)]

mod test_accounting_handle {
    use async_session::chrono::{Datelike, Duration, TimeZone, Utc};
    use mongodb::bson::Uuid;

    use crate::{
        accounting_config_logic::FinanceAccountingConfigHandle,
        accounting_logic::FinanceBookingHandle,
        database_handler_mongodb::DbConnectionSetting,
        datatypes::{
            AccountBalanceInfo, AccountBalanceType, BookingEntryType, FinanceAccount,
            FinanceAccountType, FinanceBookingRequest, FinanceBookingResult, FinanceJournalEntry,
        },
        tests::mocking_database::{InMemoryDatabaseData, InMemoryDatabaseHandler},
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

        let mut account_handle_1 = FinanceAccountingConfigHandle::new(
            &dummy_connection_settings,
            &user_id_1,
            &in_memory_db,
        );
        let mut account_handle_2 = FinanceAccountingConfigHandle::new(
            &dummy_connection_settings,
            &user_id_2,
            &in_memory_db,
        );
        let mut account_handle_3 = FinanceAccountingConfigHandle::new(
            &dummy_connection_settings,
            &user_id_3,
            &in_memory_db,
        );

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
        let insert_finance_booking_request_1_1_result =
            booking_handle_1.finance_insert_booking_entry(&finance_booking_request_1_1);
        let insert_finance_booking_request_1_2_result =
            booking_handle_1.finance_insert_booking_entry(&finance_booking_request_1_2);
        let insert_finance_booking_request_2_1_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_1);
        let insert_finance_booking_request_2_2_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_2);
        let insert_finance_booking_request_2_3_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_3);
        let insert_finance_booking_request_3_1_result =
            booking_handle_3.finance_insert_booking_entry(&finance_booking_request_3_1);

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

        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_1_1,
            &insert_finance_booking_request_1_1_result.unwrap()
        ));
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_1_2,
            &insert_finance_booking_request_1_2_result.unwrap()
        ));
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_2_1,
            &insert_finance_booking_request_2_1_result.unwrap()
        ));
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_2_2,
            &insert_finance_booking_request_2_2_result.unwrap()
        ));
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_2_3,
            &insert_finance_booking_request_2_3_result.unwrap()
        ));

        let full_listing_user_1_result = booking_handle_1.list_journal_entries(None, None);
        let full_listing_user_2_result = booking_handle_2.list_journal_entries(None, None);
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
        let insert_finance_booking_request_1_3_result =
            booking_handle_1.finance_insert_booking_entry(&finance_booking_request_1_3);
        assert!(
            insert_finance_booking_request_1_3_result.is_ok(),
            "{}",
            insert_finance_booking_request_1_3_result.unwrap_err()
        );
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_1_3,
            &insert_finance_booking_request_1_3_result.unwrap()
        ));
        let full_listing_user_1_2_result = booking_handle_1.list_journal_entries(None, None);
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
            running_number_2 < running_number_1,
            "new running is not greater than old running number"
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
        let full_listing_user_1_3_result =
            booking_handle_1.list_journal_entries(Some(check_date_time_1), None);
        let full_listing_user_1_4_result =
            booking_handle_1.list_journal_entries(None, Some(check_date_time_2));
        let full_listing_user_1_5_result =
            booking_handle_1.list_journal_entries(Some(check_date_time_1), Some(check_date_time_2));
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
            &finance_booking_request_1_1
        ));

        /* Test 4 further invalid operations
        trying to perform invalid operations
        a) using datetime filtering where till datetime is before from datetime
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
        */
        let full_listing_user_1_6_result =
            booking_handle_1.list_journal_entries(Some(check_date_time_2), Some(check_date_time_1));
        assert!(
            full_listing_user_1_6_result.is_err(),
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
        let insert_finance_booking_request_2_4_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_4);
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
        let insert_finance_booking_request_2_5_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_5);
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
        let insert_finance_booking_request_2_6_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_6);
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
        let insert_finance_booking_request_2_7_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_7);
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
        let insert_finance_booking_request_2_8_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_8);
        assert!(
            insert_finance_booking_request_2_8_result.is_ok(),
            "could not prepare saldo check: {}",
            insert_finance_booking_request_2_8_result.unwrap_err()
        );
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_2_8,
            &insert_finance_booking_request_2_8_result.unwrap()
        ));

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
        let insert_finance_booking_request_2_9_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_9);
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
        let insert_finance_booking_request_2_10_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_10);
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
        let insert_finance_booking_request_2_11_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_11);
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
        let insert_finance_booking_request_2_12_result =
            booking_handle_2.finance_insert_booking_entry(&finance_booking_request_2_12);
        assert!(
            insert_finance_booking_request_2_12_result.is_err(),
            "operation should have failed"
        );
        assert!(insert_finance_booking_request_2_12_result
            .unwrap_err()
            .to_string()
            .contains(saldo_error_text));
    }

    #[tokio::test]
    async fn test_accounting_booking_calculate_with_mock() {
        /* this test is "just" for testing the creation of entries, for validation of calculation please see other tests */
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

        let mut account_handle_1 = FinanceAccountingConfigHandle::new(
            &dummy_connection_settings,
            &user_id_1,
            &in_memory_db,
        );

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
        let booking_time_4 = booking_time_3 + Duration::days(1);
        let booking_time_5 = booking_time_4 + Duration::days(1);
        let booking_time_6 = booking_time_5 + Duration::days(1);
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
        let account_1_running_saldo_amount = amount_1 - amount_2;
        let account_1_running_saldo_type = AccountBalanceType::Credit;
        let account_2_running_saldo_amount = amount_1.abs_diff(amount_3);
        let account_2_running_saldo_type = AccountBalanceType::Debit;
        let account_3_running_saldo_amount = amount_2 + amount_3;
        let account_3_running_saldo_type = AccountBalanceType::Debit;

        let insert_finance_booking_request_1_1_result =
            booking_handle_1.finance_insert_booking_entry(&finance_booking_request_1_1);
        let insert_finance_booking_request_1_2_result =
            booking_handle_1.finance_insert_booking_entry(&finance_booking_request_1_2);
        let insert_finance_booking_request_1_3_result =
            booking_handle_1.finance_insert_booking_entry(&finance_booking_request_1_3);
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

        let balance_account_1_result =
            booking_handle_1.calculate_balance_info(&vec![finance_account_1_1.id]);
        let balance_account_2_result =
            booking_handle_1.calculate_balance_info(&vec![finance_account_1_2.id]);
        let balance_account_3_result =
            booking_handle_1.calculate_balance_info(&vec![finance_account_1_3.id]);

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

        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_1_1,
            &insert_finance_booking_request_1_1_result.unwrap()
        ));
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_1_2,
            &insert_finance_booking_request_1_2_result.unwrap()
        ));
        assert!(check_entry_response_match_entry_request(
            &finance_booking_request_1_3,
            &insert_finance_booking_request_1_3_result.unwrap()
        ));

        let balance_account_1_info = balance_account_1_result.unwrap();
        let balance_account_2_info = balance_account_2_result.unwrap();
        let balance_account_3_info = balance_account_3_result.unwrap();
        assert_eq!(balance_account_1_info.len(), 1);
        assert_eq!(balance_account_2_info.len(), 1);
        assert_eq!(balance_account_3_info.len(), 1);
        assert!(check_balance_account_info(
            &balance_account_1_info[0],
            &finance_account_1_1.id,
            &account_1_running_saldo_amount,
            &account_1_running_saldo_type
        ));
        assert!(check_balance_account_info(
            &balance_account_2_info[0],
            &finance_account_1_2.id,
            &account_2_running_saldo_amount,
            &account_2_running_saldo_type
        ));
        assert!(check_balance_account_info(
            &balance_account_3_info[0],
            &finance_account_1_3.id,
            &account_3_running_saldo_amount,
            &account_3_running_saldo_type
        ));
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

    fn check_entry_response_match_entry_request(
        entry_request: &FinanceBookingRequest,
        entry_response: &FinanceBookingResult,
    ) -> bool {
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
            .eq(&entry_response.journal_entry.is_saldo)
            && entry_request
                .is_simple_entry
                .eq(&entry_response.journal_entry.is_saldo)
            && entry_request
                .amount
                .eq(&entry_response.journal_entry.amount)
            && entry_request
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
                        .credit_finance_account_id
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
                    return true;
                }
            }
        }
        return false;
    }

    fn check_balance_account_info(
        info_to_check: &AccountBalanceInfo,
        account_id: &Uuid,
        amount: &u128,
        balance_type: &AccountBalanceType,
    ) -> bool {
        if account_id.eq(&info_to_check.account_id)
            && amount.eq(&info_to_check.amount)
            && balance_type.eq(&info_to_check.balance_type)
        {
            return true;
        } else {
            return false;
        }
    }
    fn get_max_running_number_from_journal_list(journal_list: &Vec<FinanceJournalEntry>) -> u64 {
        let max_option = journal_list.iter().max_by_key(|elem| elem.running_number);
        if max_option.is_none() {
            return 0;
        }
        return max_option.unwrap().running_number;
    }
}
