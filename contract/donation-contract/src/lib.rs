/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env,
    json_types::U128,
    near_bindgen,
    serde::Serialize,
    AccountId, Balance, ONE_NEAR,
};

const STORAGE_COST_PER_BYTE: Balance = ONE_NEAR / 100_000;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    donation_target: String,
    donation_accounts: UnorderedMap<AccountId, Balance>,
    donation_messages: UnorderedMap<AccountId, Option<String>>,
    total_donations: Balance,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    account_id: AccountId,
    amount: U128,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DonationMessage {
    account_id: AccountId,
    message: String,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn init(donation_target: String) -> Self {
        Self {
            donation_target,
            donation_accounts: UnorderedMap::new(b"d"),
            donation_messages: UnorderedMap::new(b"m"),
            total_donations: 0,
        }
    }

    #[payable]
    pub fn donate(&mut self, message: Option<String>) {
        assert_one_yocto();

        let donated_amount = env::attached_deposit();
        let account_donated = env::signer_account_id();

        self.total_donations += donated_amount;

        let account_donated_amount =
            self.donation_accounts.get(&account_donated).unwrap_or(0) + donated_amount;

        self.donation_accounts
            .insert(&account_donated, &account_donated_amount);

        self.donation_messages.insert(&account_donated, &message);
    }

    pub fn show_message(&self, account_id: AccountId) -> String {
        self.donation_messages
            .get(&account_id)
            .unwrap_or_else(|| Option::Some(String::new()))
            .unwrap_or_default()
    }

    pub fn list_messages(&self, from_index: Option<U128>, take: Option<u64>) -> Vec<String> {
        self.donation_messages
            .iter()
            .skip(u128::from(from_index.unwrap_or(U128(0))) as usize)
            .take(take.unwrap_or(50) as usize)
            .filter_map(|(_, message)| {
                if message.clone().unwrap_or_default().is_empty() {
                    None
                } else {
                    message
                }
            })
            .collect()
    }

    pub fn show_donation(&self, account_id: AccountId) -> U128 {
        U128(self.donation_accounts.get(&account_id).unwrap_or_default())
    }

    pub fn list_donations(&self, from_index: Option<U128>, take: Option<u64>) -> Vec<U128> {
        self.donation_accounts
            .iter()
            .skip(u128::from(from_index.unwrap_or(U128(0))) as usize)
            .take(take.unwrap_or(50) as usize)
            .filter_map(|(_, amount_donated)| {
                if amount_donated == 0 {
                    None
                } else {
                    Some(U128(amount_donated))
                }
            })
            .collect()
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
        let contract = Contract::init(String::new());
        // this test did not call set_greeting so should return the default "Hello" greeting
        // assert_eq!(contract.get_greeting(), "Hello".to_string());
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::init(String::new());
        // this test did not call set_greeting so should return the default "Hello" greeting
        // contract.set_greeting("howdy".to_string());
        // assert_eq!(contract.get_greeting(), "howdy".to_string());
    }
}
