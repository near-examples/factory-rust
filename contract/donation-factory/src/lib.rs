/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedSet,
    env, log, near_bindgen,
    serde::Serialize,
    AccountId, Balance, Gas, Promise, ONE_NEAR,
};

// Define the default message
const MIN_STORAGE_COST: Balance = ONE_NEAR / 10;
const DONATION_CONTRACT: &[u8] = include_bytes!("../donation.wasm");

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    donation_contracts: UnorderedSet<String>,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            donation_contracts: UnorderedSet::new(b"d"),
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
struct DonationContractInitArgs {
    donation_target: String,
}

fn create_new_account_id(account_id: AccountId) -> AccountId {
    format!("donation.{account_id}").parse().unwrap()
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    pub fn create_donation_contract(&self, donation_target: Option<String>) -> Promise {
        let signer_account_id = env::signer_account_id();
        let donation_target = donation_target.unwrap_or_else(|| signer_account_id.to_string());

        assert!(
            env::is_valid_account_id(donation_target.as_bytes()),
            "Invalid donation target provided!"
        );

        let new_account_id = create_new_account_id(signer_account_id);
        let init_args =
            near_sdk::serde_json::to_vec(&DonationContractInitArgs { donation_target }).unwrap();

        Promise::new(new_account_id.clone())
            .create_account()
            .transfer(MIN_STORAGE_COST)
            .deploy_contract(DONATION_CONTRACT.to_vec())
            .add_full_access_key(env::signer_account_pk())
            .function_call("init".to_owned(), init_args, 0, Gas(5 * 10u64.pow(13)))
            .then(
                Self::ext(env::current_account_id())
                    .on_donation_contract_deployed(new_account_id.to_string()),
            )
    }

    pub fn remove_donation_contract(&self) -> Promise {
        let signer_account_id = env::signer_account_id();
        let new_account_id = create_new_account_id(signer_account_id);

        Promise::new(new_account_id.clone())
            .delete_account(signer_account_id)
            .then(
                Self::ext(env::current_account_id())
                    .on_donation_contract_deleted(new_account_id.to_string()),
            )
    }

    #[private]
    pub fn on_donation_contract_deployed(&mut self, donation_account: String) {
        self.donation_contracts.insert(&donation_account);

        log!("Successfully deployed donation contract to {donation_account}");
    }

    #[private]
    pub fn on_donation_contract_deleted(&mut self, donation_account: String) {
        self.donation_contracts.remove(&donation_account);

        log!("Successfully deleted donation contract from {donation_account}");
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_greeting() {
        let contract = Contract::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        // assert_eq!(contract.get_greeting(), "Hello".to_string());
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        // contract.set_greeting("howdy".to_string());
        // assert_eq!(contract.get_greeting(), "howdy".to_string());
    }
}
