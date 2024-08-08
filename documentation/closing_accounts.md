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