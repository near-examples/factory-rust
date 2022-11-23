use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, Balance, Gas};

mod deploy;
mod manager;

const NEAR_PER_STORAGE: Balance = 10_000_000_000_000_000_000; // 10e19yⓃ
const DEFAULT_CONTRACT: &[u8] = include_bytes!("./donation-contract/donation.wasm");
const TGAS: Gas = Gas(10u64.pow(12)); // 10e12yⓃ
const NO_DEPOSIT: Balance = 0; // 0yⓃ

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    code: Vec<u8>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            code: DEFAULT_CONTRACT.to_vec(),
        }
    }
}

