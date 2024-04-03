use near_sdk::{env, near};

use crate::{Contract, ContractExt};

#[near]
impl Contract {
    #[private]
    pub fn update_stored_contract(&mut self) {
        // This method receives the code to be stored in the contract directly
        // from the contract's input. In this way, it avoids the overhead of
        // deserializing parameters, which would consume a huge amount of GAS
        self.code.set(env::input());
    }

    pub fn get_code(&self) -> &Vec<u8> {
        // If a contract wants to update themselves, they can ask for the code needed
        self.code.get().as_ref().unwrap()
    }
}
