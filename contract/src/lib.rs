use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, Balance, Gas};

mod deploy;
mod manager;

const MIN_STORAGE_COST: Balance = 1_240_000_000_000_000_000_000_000; //1.24Ⓝ
const DONATION_CONTRACT_BYTES: &[u8] = include_bytes!("./donation-contract/donation.wasm");
const TGAS: Gas = Gas(10u64.pow(12));
const NO_DEPOSIT: Balance = 0;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
  code: Vec<u8>
}

impl Default for Contract {
  fn default() -> Self {
      Self{ code: DONATION_CONTRACT_BYTES.to_vec() }
  }
}