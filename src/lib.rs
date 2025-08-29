use std::sync::Arc;

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::{
    borsh, env, ext_contract, near, store, AccountId, CryptoHash, Promise, PromiseError,
};

mod manager;

const DEFAULT_GLOBAL_CONTRACT_ID: &str = "ft.globals.primitives.testnet";

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum GlobalContractId {
    AccountId(AccountId),
    CodeHash(String),
}

#[near(contract_state)]
pub struct GlobalFactoryContract {
    pub global_contract_id: GlobalContractId,
    /// Store the hash of deployed global contracts for reference
    pub deployed_global_contracts: store::IterableMap<AccountId, GlobalContractId>,
}

impl Default for GlobalFactoryContract {
    fn default() -> Self {
        Self {
            global_contract_id: GlobalContractId::AccountId(
                DEFAULT_GLOBAL_CONTRACT_ID.parse().unwrap(),
            ),
            deployed_global_contracts: store::IterableMap::new(b"d".to_vec()),
        }
    }
}

#[near]
impl GlobalFactoryContract {
    /// Deploy a global contract with the given bytecode, identifiable by its code hash
    #[payable]
    pub fn deploy(&mut self, name: String) -> Promise {
        // Assert the sub-account is valid
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{name}.{current_account}").parse().unwrap();
        assert!(
            env::is_valid_account_id(subaccount.as_bytes()),
            "Invalid subaccount"
        );

        self.deployed_global_contracts
            .insert(subaccount.clone(), self.global_contract_id.clone());

        match self.global_contract_id {
            GlobalContractId::AccountId(ref account_id) => {
                env::log_str(&format!(
                    "Using global contract deployed by account: {}",
                    account_id
                ));

                Promise::new(subaccount)
                    .create_account()
                    .transfer(env::attached_deposit())
                    .add_full_access_key(env::signer_account_pk())
                    .use_global_contract_by_account_id(account_id.clone())
            }
            GlobalContractId::CodeHash(ref code_hash) => {
                env::log_str(&format!(
                    "Using global contract with code hash: {:?}",
                    code_hash
                ));
                Promise::new(subaccount)
                    .create_account()
                    .transfer(env::attached_deposit())
                    .add_full_access_key(env::signer_account_pk())
                    .use_global_contract(code_hash.as_bytes().to_vec())
            }
        }
    }

    /// List all deployed global contracts
    pub fn list_global_contracts(&self) -> Vec<(AccountId, GlobalContractId)> {
        self.deployed_global_contracts
            .iter()
            .map(|(account_id, global_contract_id)| {
                (account_id.clone(), (global_contract_id.clone()))
            })
            .collect()
    }

    pub fn get_deployed_contract_global_id(
        &self,
        account_id: AccountId,
    ) -> Option<GlobalContractId> {
        self.deployed_global_contracts.get(&account_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    fn get_context(predecessor_account_id: AccountId) -> near_sdk::VMContext {
        VMContextBuilder::new()
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id)
            .build()
    }

    #[test]
    fn test_deploy() {
        let context = get_context(accounts(1));
        testing_env!(context);

        let mut contract = GlobalFactoryContract::default();

        contract.deploy("test_contract".to_string());

        // Check that the contract was recorded
        let stored_id = contract.get_deployed_contract_global_id(
            format!("test_contract.{}", accounts(0)).parse().unwrap(),
        );
        assert!(stored_id.is_some());

        assert_eq!(
            stored_id.unwrap(),
            GlobalContractId::AccountId(DEFAULT_GLOBAL_CONTRACT_ID.parse().unwrap()),
        );
    }

    #[test]
    fn test_list_global_contracts() {
        let context = get_context(accounts(1));
        testing_env!(context);

        let mut contract = GlobalFactoryContract::default();

        contract.deploy("test_contract".to_string());

        let contracts = contract.list_global_contracts();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].0, format!("test_contract.{}", accounts(0)));
        assert_eq!(contracts[0].1, contract.get_global_contract_id());
    }
}
