# FinanceOverViewTraining
Main goal of this project is to train development using "rust". Side goal is to create a web application that allows to log income and expances

reached mile stones:
version 0.0.1:
- [x] given out simple text via http request
- [x] secure simple server configuration in an ini file

open mile stones:
hint: new miles stones might be added

goal points for version 0.0.2:
- [ ] integrate simple log mechanismn
- [ ] using database to count amount of incoming request (might be removed in later versions)
- [ ] providing secure https connection with self signed certificates
Versions hints:
* needs couchbase server (https://www.couchbase.com/downloads)
* configure couchbase server at http://localhost:8091/ui/index.html after installation
* add to config file: clustername, database user and database password, bucket name
* needs cmake
* might have to allow some scripts from cmake to be run, like "get_repo_version.ps1" (see https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.security/set-executionpolicy?view=powershell-7.2) 
* couchbase version 1.0.0-alpha.4 uses an old version of cmake (0.1.45) and therefore does not support current visual studio. Either install an old Visual Studio or create a fork of couchbase with updated dependency


goal points for version 0.0.3:
- [ ] provide registration via email, choosing own user name
- [ ] provide password reset via email
hint for later development: a user should only see his own data.
