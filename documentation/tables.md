# UserList - List of all users
| column | type | description |
| :--- | :---: | ---: |
| user_id | UUID | unique identifier, primary key |
| user_name | text | clear identifier |
| password_hash | text | with argon2 hashed password, salt is used |
| first_name | text | first name of user |
| last_name | text | last name of user |
| reset_secret_hash | text | with argon2 hashed reset secret, salt is used, only with correct secret a password reset can be requested |
| password_reset_token_timestamp | UTC timestamp text | time when a password reset was requested |
| password_reset_token_value | text | random token, used during password reset to secure reset urls |

# FinanceAccountTypes - List of finance account type a user has defined
| column | type | description |
| :--- | :---: | ---: |
| finance_account_type_id | UUID | unique identifier, primary key |
| user_id | UUID | unique identifier, secondary key |
| title | text | name of the finance account type |
| description | text | additional information |

# FinanceAccountList - List of finance accounts of an user
| column | type | description |
| :--- | :---: | ---: |
| finance_account_id | UUID | unique identifier, primary key |
| user_id | UUID | unique identifier, secondary key |
| finance_account_type_id | UUID | unique identifier, secondary key |
| title | text | name of the finance account type |
| description | text | additional information |

# FinanceJournalDiary - list of all financial bookings of an user
| column | type | description |
| :--- | :---: | ---: |
| finance_journal_diary_id | UUID | unique identifier, primary key |
| user_id | UUID | unique identifier, secondary key |
| is_simple_entry | boolean | if true exavtly two finance accounts are involved, false more than two finance accounts are used, currently always true |
| is_saldo | boolean | if true this Entry closes accounts |
| debit_finance_account_id | UUID | unique identifier, secondary key, only used when is_simple_entry set to true  |
| credit_finance_account_id | UUID | unique identifier, secondary key, only used when is_simple_entry set to true |
| running_number | usigned int64 | running number (per user) of entry |
| booking_time | datetime | date and time when entry was created |
| amount | unsigned int128 | value of entry in cents |
| title | text | short desciption of entry |
| description | text | additional informaion |

# BookingEntries - list of financial bookings of an user for open accounts
| column | type | description |
| :--- | :---: | ---: |
| booking_entry_id | UUID | unique identifier, primary key |
| user_id | UUID | unique identifier, secondary key |
| finance_account_id | UUID | unique identifier, secondary key |
| finance_journal_diary_id | UUID | unique identifier, secondary key |
| booking_type | int8 | kind of booking entry, 4 possible values: credit (0), debit (1), saldo_credit (2) or saldo_debit (3) |
| booking_time | datetime | date and time when entry was created |
| amount | unsigned int128 | value of entry in cents|
| title | text | short desciption of entry |
| description | text | additional informaion |

# CounterCollection - needed to implement autoincrement function
| column | type | description |
| :--- | :---: | ---: |
| counter_entry_id | UUID | unique identifier, primary key |
| user_id | UUID | unique identifier, secondary key |
| booking_journal_max_number | unsigned int64 | current max counter for FinanceJournalDiary