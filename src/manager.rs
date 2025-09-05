use near_sdk::{near, NearToken};

use crate::{GlobalContractId, GlobalFactoryContract, GlobalFactoryContractExt};

#[near]
impl GlobalFactoryContract {
    #[private]
    pub fn update_global_contract_id(&mut self, contract_id: String) {
        self.global_contract_id = GlobalContractId::from(contract_id);
    }

    pub fn get_global_contract_id(&self) -> String {
        self.global_contract_id.to_string()
    }

    #[private]
    pub fn update_min_deposit(&mut self, amount: NearToken) {
        self.min_deposit_amount = amount;
    }

    pub fn get_min_deposit(&self) -> NearToken {
        self.min_deposit_amount
    }
}
