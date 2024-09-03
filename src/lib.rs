// Find all our documentation at https://docs.near.org
use near_sdk::store::LazyOption;
use near_sdk::{near, Gas, NearToken};

mod deploy;
mod manager;

const NEAR_PER_STORAGE: NearToken = NearToken::from_yoctonear(10u128.pow(19)); // 10e19yⓃ
const DEFAULT_CONTRACT: &[u8] = include_bytes!("./donation-contract/donation.wasm");
const TGAS: Gas = Gas::from_tgas(1);
const NO_DEPOSIT: NearToken = NearToken::from_near(0); // 0yⓃ

// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    // Since a contract is something big to store, we use LazyOptions
    // this way it is not deserialized on each method call
    code: LazyOption<Vec<u8>>,
    // Please note that it is much more efficient to **not** store this
    // code in the state, and directly use `DEFAULT_CONTRACT`
    // However, this does not enable to update the stored code.
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            code: LazyOption::new("code".as_bytes(), Some(DEFAULT_CONTRACT.to_vec())),
        }
    }
}
