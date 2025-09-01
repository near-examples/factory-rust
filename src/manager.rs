use near_sdk::near;

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
}
