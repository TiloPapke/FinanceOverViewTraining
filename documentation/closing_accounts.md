this file is for brainstorming datastructures and mechanismns around closing account.

ideas
* define a hierachy of accounts, multiple accounts to many
* an account can have none, one or many child accounts
* there is excatly one account that is on top of the hierachy with no parent account
* all other accounts need to have one parent account
* parent account can be defined on account type level and on account level
* parent definition required either on account type level or account level
** when defined on both level, parent definition on account level overrides definition on account type level
* definition: account with no parrent account is at top of hierarchy
* definition: accounts with no subaccounts are at the bottom of hierarchy




datastructure
* new field parent_account_id in account type table (can be undefined)
* new field parent_account_id in account table (can be undefined)

required checks before calculation:
* each account has as parent_account_id, except one top account
* no circular references allowed
* any given parent_account_id belongs to an accounting table of the current user

functions of trait "accounting_saldo_handling" OR extending "DBFinanceConfigFunctions"(1) and "DBFinanceAccountingFunctions" (2)
* loading hierachy (1)
* checking hierarchy (1)
* adding/changing parent_account_id of accounts and account types (1)
* calculating saldo for single accounts (2)
* applying hierachy to all acounts (2)
=> extending existing traits, DBFinanceConfigFunctions describse what is there and DBFinanceAccountingFunctions describse what to do with it.