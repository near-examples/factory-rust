// Find all our documentation at https://docs.near.org
use near_sdk::{near, AccountId, Gas, NearToken, PanicOnDefault};

mod deploy;

const TGAS: Gas = Gas::from_tgas(1); // 10e12yⓃ
const NO_DEPOSIT: NearToken = NearToken::from_near(0); // 0yⓃ

// Define the contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub global_contract_id: AccountId,
}

#[near]
impl Contract {
    /// Initialize with `ft.globals.primitives.near` on mainnet or
    /// `ft.globals.primitives.testnet` on testnet.
    #[init]
    pub fn new(global_contract_id: AccountId) -> Self {
        Self { global_contract_id }
    }
}
