# FinanceOverViewTraining
Main goal of this project is to train development using "rust". Side goal is to create a web application that allows to log income and expances

reached mile stones:

goal points for version 0.0.5
- [x] unit test case: check if UUID of different user account is used when updating or creating elements => reject
- [x] correct spellings like display_paswword_reset_with_token_page (now: display_password_reset_with_token_page)
- [x] code refactoring: centralizse session handle (eg getting session data and setting new expire timestamp)
- [x] code refactoring: connection setting handle
- [x] define timeout for session in server settings
- [ ] calculting saldo over all finance accounts
- [ ] export to csv (simple)

version 0.0.4:
- [x] unit tests
- [-] hashed session id to ensure a users identity for futres calls so that a user can only change own data, rejected instead user_id stored in server side session data
- [x] defintion of data structure for storage
- [x] hash content of field "reset_secret"
- [x] simple finance account management
- [x] journaling of income and expanses

!Attention!, please see this:
https://www.mongodb.com/docs/manual/core/retryable-writes/
https://stackoverflow.com/questions/58589631/mongoerror-this-mongodb-deployment-does-not-support-retryable-writes-please-ad
https://dba.stackexchange.com/questions/265236/how-can-we-use-transaction-in-mongodb-standalone-connection
might need a super user for this: https://stackoverflow.com/questions/23943651/mongodb-admin-user-not-authorized
in case normal Connection using MongoDB Compass fail: use Advanced Settings > Direct Connection, then excute rs.initiate()

version 0.0.3:
- [x] session controls
- [x] minimal user settings page
- [x] provide registration via email, choosing own user name
- [x] provide password reset via email
hint for later development: a user should only see his own data.

version 0.0.2:
- [x] integrate simple log mechanismn
- [x] using database to count amount of incoming request (might be removed in later versions)
- [x] providing secure https connection with self signed certificates
Versions hints:
* needs mongoDB Server (see https://www.mongodb.com/docs/manual/tutorial/install-mongodb-on-windows/)
* might have to allow some scripts to be run (see https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.security/set-executionpolicy?view=powershell-7.2) 
* certificates loaded from https://github.com/programatik29/axum-server/tree/master/examples/self-signed-certs
* you might have to change the certifactes

version 0.0.1:
- [x] given out simple text via http request
- [x] secure simple server configuration in an ini file

open mile stones:

hint: new miles stones might be added

goal points for version 0.0.6
- [ ] documentation for interface functions
- [ ] check if some mut markings in implementations of trait DBFinanceConfigFunctions can be removed
- [ ] redesign UI
- [ ] using a css framework for better styles
- [ ] updating account table info when inserting booking entry

goal points for version 0.0.7
- [ ] refactoring unit tests that uses mocking database, use an init method that prepares a datastruture for all mocking test
- [ ] when inserting accounting request the response should contain a timestamp and the running number (last one only when successful request) so that consecutive request have a different response text.
- [ ] extending unit tests: check return of html render and ajax method, especially for not being logged in and session timeout
- [ ] decision needed: should changing account type of existing accounts be allowed => yes/no? => unit tests required, kind of tests depends on decision
- [ ] better way for validating database structure, when each new trait could have new tables

goal points for version 0.0.7
- [ ] ipv6 configuration