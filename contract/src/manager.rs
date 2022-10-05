use near_sdk::{env, near_bindgen};

use crate::{Contract, ContractExt};

#[near_bindgen]
impl Contract {
    #[private]
    pub fn update_stored_contract(&mut self) {
        // This method receives the code to be stored in the contract directly
        // from the contract's input. In this way, it avoids the overhead of
        // deserializing parameters, which would consume a huge amount of GAS
        self.code = env::input().expect("Error: No input").to_vec();
    }

    pub fn get_code(&self) -> &Vec<u8> {
        // If a contract wants to update themselves, they can ask us for the code needed
        &self.code
    }
}

