use mongodb::bson::Uuid;

use crate::{accounting_config_database::DBFinanceConfigFunctions, datatypes::FinanceAccountType};

pub struct FinanceAccounttingHandle<'a> {
    user_id: &'a Uuid,
    db_connector: &'a dyn DBFinanceConfigFunctions,
}

impl<'a> FinanceAccounttingHandle<'a> {
    pub fn new(user_id: &'a Uuid, db_connector: &'a dyn DBFinanceConfigFunctions) -> Self {
        Self {
            user_id,
            db_connector,
        }
    }

    pub async fn finance_account_type_list(
        _finance_account_type: &FinanceAccountType,
    ) -> Result<Vec<FinanceAccountType>, String> {
        unimplemented!("Listing finance account type not impplemented")
    }

    pub async fn finance_account_type_upsert(
        _finance_account_type: &FinanceAccountType,
    ) -> Result<bool, String> {
        unimplemented!("upsert new finance account type not impplemented")
    }
}
