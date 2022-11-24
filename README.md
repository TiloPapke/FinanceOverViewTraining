# FinanceOverViewTraining
Main goal of this project is to train development using "rust". Side goal is to create a web application that allows to log income and expances

reached mile stones:

goal points for version 0.0.2:
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

goal points for version 0.0.3:
- [x] session controls
- [in progress] minimal user settings page
- [ ] provide registration via email, choosing own user name
- [ ] provide password reset via email
hint for later development: a user should only see his own data.

goal points for version 0.0.4
- [ ] unit tests
- [ ] hashed session id to ensure a users identity for futres calls so that a user can only change own data
- [ ] simple finance account management
- [ ] journaling of income and expanses

goal points for version 0.0.5
- [ ] calculting saldo over all finance accounts
- [ ] export to csv (simple)

