use near_sdk::{near, AccountId};

use crate::{GlobalContractId, GlobalFactoryContract, GlobalFactoryContractExt};

#[near]
impl GlobalFactoryContract {
    #[private]
    pub fn update_global_contract_id(&mut self, contract_id: String, as_hash: bool) {
        if as_hash {
            self.global_contract_id = GlobalContractId::CodeHash(contract_id);
        } else {
            let account_id: AccountId = contract_id.parse().expect("Invalid account ID");
            self.global_contract_id = GlobalContractId::AccountId(account_id);
        }
    }

    pub fn get_global_contract_id(&self) -> GlobalContractId {
        self.global_contract_id.clone()
    }
}
