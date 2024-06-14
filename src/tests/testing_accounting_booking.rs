#[cfg(test)]

mod test_accounting_handle {
    use mongodb::bson::Uuid;

    use crate::{
        accounting_logic::FinanceBookingHandle,
        database_handler_mongodb::DbConnectionSetting,
        tests::mocking_database::{InMemoryDatabaseData, InMemoryDatabaseHandler},
    };

    #[tokio::test]
    async fn test_accounting_booking_with_mock() {
        /* this test is "just" for testing the creation of entries, for validation please see other tests */
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

        let booking_handle_1 =
            FinanceBookingHandle::new(&dummy_connection_settings, &user_id_1, &in_memory_db);
        let booking_handle_2 =
            FinanceBookingHandle::new(&dummy_connection_settings, &user_id_2, &in_memory_db);
        let booking_handle_3 =
            FinanceBookingHandle::new(&dummy_connection_settings, &user_id_3, &in_memory_db);

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

        /* Test 2 increasing running number
        insert another entry for user 1

        checks:
        in full listing before new entry the biggest running booking number is lower than biggest running number in full listing after insert new booking entry
        */

        /* Test 3 Filtering options
        using datetime filters to limit results using
        a) just from dateime
        b) just till datetime
        c) using from and till datetime

        checks: listings only have the limited results
        */

        /* Test 4 further invalid operations
        trying to perform invalid operations
        a) using datetime filtering where till datetime is before from datetime
        b) insert a bookking entry with a booking time already presents
            b1) for credit account
            b2) for debit account
        */
    }
}
