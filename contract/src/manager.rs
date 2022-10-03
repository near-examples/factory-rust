use near_sdk::{ env, near_bindgen };

use crate::*;

#[near_bindgen]
impl Contract {

  pub fn update_stored_contract(&mut self) {
    // This method receives the code to be stored in the contract, but instead of
    // receiving it through a parameter, we read it directly from the contract's input.
    // This is necessary, since otherwise the deserialization consumes all the GAS!
    self.code = env::input().expect("Error: No input").to_vec();
  }

  pub fn get_code(&self) -> &Vec<u8> {
    // If a contract wants to update themselves, they can ask us for the code needed
    &self.code
  }
}