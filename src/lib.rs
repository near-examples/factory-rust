// Find all our documentation at https://docs.near.org
use near_sdk::store::LazyOption;
use near_sdk::{near, Gas, NearToken};

mod deploy;

const NEAR_PER_STORAGE: NearToken = NearToken::from_yoctonear(10u128.pow(18)); // 10e18yⓃ
const FT_CONTRACT: &[u8] = include_bytes!("./ft-contract/ft.wasm");
const TGAS: Gas = Gas::from_tgas(1); // 10e12yⓃ
const NO_DEPOSIT: NearToken = NearToken::from_near(0); // 0yⓃ

// Define the contract structure
#[near(contract_state)]
pub struct Contract { }

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self { }
    }
}
