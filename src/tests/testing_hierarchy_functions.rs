#[cfg(test)]

pub(crate) mod test_accounting_handle {

    async fn test_extract_hierarchy() {
        panic!("no checks for test_extract_hierarchy defined");

        /* Test 1 extract hierarchy for user with no account defined
        checks:
        * extract is fine
         */

        /* Test 2 extract hierarchy for user with one account defined
        checks:
        * extract is fine
         */

        /* Test 3 extract hierarchy for user with two account defined
        checks:
        * extract is fine
         */

        /* Test 4 extract hierarchy for user with multiple Account
        A>B, A>C
        checks:
        * extract is fine
         */

        /* Test 5 extract hierarchy for user with multiple Account
        A>B, B>C
        checks:
        * extract is fine
         */

        /* Test 6 extract hierarchy for user with multiple Account
        A>B, B>C, C>A
        checks:
        * extract is fine
         */

        /* Test 7 extract hierarchy for user with multiple Account
        A>B, B>C, C>D
        checks:
        * extract is fine
         */

        /* Test 8 extract hierarchy for user with multiple Account
        A>B, B>C, D
        checks:
        * extract is fine
         */
    }

    async fn test_check_hierarchy() {
        /* Test 1 check hierarchy for user with no account defined
        checks:
        * check is fine
         */

        /* Test 2 check hierarchy for user with one account defined
        checks:
        * check is fine
         */

        /* Test 3 check hierarchy for user with two account defined
        checks:
        * check is fine
         */

        /* Test 4 check hierarchy for user with multiple Account
        A>B, A>C
        checks:
        * check is fine
         */

        /* Test 5 check hierarchy for user with multiple Account
        A>B, B>C
        checks:
        * check is fine
         */

        /* Test 6 check hierarchy for user with multiple Account
        A>B, B>C, C>A
        checks:
        * check gives error either circulary reference or no single root
         */

        /* Test 7 check hierarchy for user with multiple Account
        A>B, B>C, C>D
        checks:
        * check gives error no single root
         */

        /* Test 8 check hierarchy for user with multiple Account
        A>B, C>D, E
        checks:
        * check gives error either no single root or floating node
         */
        panic!("no checks for test_check_hierarchy defined");
    }

    async fn test_hierarchy_closing_validation() {
        /* Test group A Config:
        A>B, C>D, B>E D>E */

        /*Test 1, Check calculated values
        Current saldos:
        A: +1.02
        B: +2.05
        C: -0.95
        D: -2.12
        E: 0
        Calculated closing values:
        A: -1.02
        B: -1.03 (after closing A)
        C: +0.95
        D: +0.15 (afer closing C)
        E:  0.00
        Check Result: E has saldo of 0
        */

        /* Test 2, like Test 1 but Account A is manupluated: +1.01
        Check: saldo of E is -0.01 => Rasing error
         */
        /* Test 3, like Test 1 but Account B is manupluated: +2.06
        Check: saldo of E is +0.01 => Rasing error
         */
        /* Test 4, like Test 1 but Account C is manupluated: -0.96, Account D is aminuplated: -2.15
        Check: saldo of E is -0.04 => Rasing error
        */

        /* Test 5
           using values of Test 1,
           writes values into data structure

           check: after reloading all accounts have a saldo of 0
        */

        panic!("no checks for test_check_hierarchy defined");
    }
}
