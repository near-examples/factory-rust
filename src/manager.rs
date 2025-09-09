use near_sdk::{near, NearToken};

use crate::{GlobalContractId, GlobalFactoryContract, GlobalFactoryContractExt};

#[near]
impl GlobalFactoryContract {
    #[private]
    pub fn update_global_contract_id(&mut self, contract_id: GlobalContractId, min_deposit: NearToken) {
        self.global_contract_id = contract_id;
        self.min_deposit_amount = min_deposit;
    }

    pub fn get_global_contract_id(&self) -> GlobalContractId {
        self.global_contract_id.clone()
    }

    pub fn get_min_deposit(&self) -> NearToken {
        self.min_deposit_amount
    }
}
