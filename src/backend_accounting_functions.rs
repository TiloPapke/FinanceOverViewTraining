use crate::{
    accounting_config_database::DBFinanceConfigFunctions,
    datatypes::{FinanceAccount, FinanceAccountHierarchyNode},
    html_render::AccountTableBookingRow,
};

pub struct BackendAccountingFunctions<'a> {
    db_finance_config_functions: &'a dyn DBFinanceConfigFunctions,
}

impl<'a> BackendAccountingFunctions<'a> {
    pub fn new(db_finance_config_functions: &'a dyn DBFinanceConfigFunctions) -> Self {
        Self {
            db_finance_config_functions,
        }
    }

    pub fn extract_hierachy(&self) -> Result<Vec<FinanceAccountHierarchyNode>, String> {
        panic!("extract_hierachy currently not implemented");
    }

    pub fn check_hierarchy(
        _hierarchy_nodes: &Vec<FinanceAccountHierarchyNode>,
    ) -> Result<bool, String> {
        panic!("check_hierarchy currently not implemented");
    }

    fn calculate_closing_row_single_account(
        _account_to_close: &FinanceAccount,
    ) -> Result<AccountTableBookingRow, String> {
        panic!("calculate_closing_row_single_account currently not implemented");
    }

    pub fn calculate_closing_row_all_accounts() -> Result<Vec<AccountTableBookingRow>, String> {
        panic!("calculate_closing_row_all_accounts currently not implemented");
    }
}
