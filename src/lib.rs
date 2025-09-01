use near_sdk::{env, near, AccountId, Promise};

mod manager;

const DEFAULT_GLOBAL_CONTRACT_ID: &str = "ft.globals.primitives.testnet";

#[derive(Clone, Debug, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
pub enum GlobalContractId {
    AccountId(AccountId),
    CodeHash(String),
}

impl ToString for GlobalContractId {
    fn to_string(&self) -> String {
        match self {
            GlobalContractId::AccountId(account_id) => account_id.to_string(),
            GlobalContractId::CodeHash(code_hash) => code_hash.clone(),
        }
    }
}

impl From<String> for GlobalContractId {
    fn from(s: String) -> Self {
        if s.parse::<AccountId>().is_ok() {
            GlobalContractId::AccountId(s.parse().unwrap())
        } else {
            GlobalContractId::CodeHash(s)
        }
    }
}

#[near(contract_state)]
pub struct GlobalFactoryContract {
    pub global_contract_id: GlobalContractId,
}

impl Default for GlobalFactoryContract {
    fn default() -> Self {
        Self {
            global_contract_id: GlobalContractId::AccountId(
                DEFAULT_GLOBAL_CONTRACT_ID.parse().unwrap(),
            ),
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

        let promise = Promise::new(subaccount)
            .create_account()
            .transfer(env::attached_deposit())
            .add_full_access_key(env::signer_account_pk());
        match self.global_contract_id {
            GlobalContractId::AccountId(ref account_id) => {
                env::log_str(&format!(
                    "Using global contract deployed by account: {}",
                    account_id
                ));

                promise.use_global_contract_by_account_id(account_id.clone())
            }
            GlobalContractId::CodeHash(ref code_hash) => {
                env::log_str(&format!(
                    "Using global contract with code hash: {:?}",
                    code_hash
                ));
                promise.use_global_contract(code_hash.as_bytes().to_vec())
            }
        }
    }
}
