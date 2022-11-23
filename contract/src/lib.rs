use near_sdk::collections::LazyOption;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, Balance, Gas};

mod deploy;
mod manager;

const NEAR_PER_STORAGE: Balance = 10_000_000_000_000_000_000; // 10e18yⓃ
const DEFAULT_CONTRACT: &[u8] = include_bytes!("./donation-contract/donation.wasm");
const TGAS: Gas = Gas(10u64.pow(12)); // 10e12yⓃ
const NO_DEPOSIT: Balance = 0; // 0yⓃ

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    // Since a contract is something big to store, we use LazyOptions
    // this way it is not deserialized on each method call
    code: LazyOption<Vec<u8>>,
    // Please note that it is much more efficient to **not** store this
    // code in the state, and directly use `DEFAULT_CONTRACT`
    // However, this does not enable to update the stored code.
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            code: LazyOption::new("code".as_bytes(), Some(&DEFAULT_CONTRACT.to_vec())),
        }
    }
}

