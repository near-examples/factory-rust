// Find all our documentation at https://docs.near.org
use near_sdk::{near, Gas, NearToken};

mod deploy;

const FT_CONTRACT: &[u8] = include_bytes!("./ft-contract/ft.wasm");
const TGAS: Gas = Gas::from_tgas(1); // 10e12yⓃ
const NO_DEPOSIT: NearToken = NearToken::from_near(0); // 0yⓃ

// Define the contract structure
#[near(contract_state)]
pub struct Contract {}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {}
    }
}
