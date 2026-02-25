use near_sdk::{env, near, AccountId, NearToken, Promise};

mod manager;

const DEFAULT_GLOBAL_CONTRACT_ID: &str = "ft.globals.primitives.testnet";
const DEFAULT_DEPOSIT_AMOUNT: u128 = 200; // 0.2 NEAR

#[derive(Clone, Debug, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
pub enum GlobalContractId {
    AccountId(AccountId),
    CodeHash(String),
}

#[near(contract_state)]
pub struct GlobalFactoryContract {
    pub global_contract_id: GlobalContractId,
    pub min_deposit_amount: NearToken,
}

impl Default for GlobalFactoryContract {
    fn default() -> Self {
        Self {
            global_contract_id: GlobalContractId::AccountId(
                DEFAULT_GLOBAL_CONTRACT_ID.parse().unwrap(),
            ),
            min_deposit_amount: NearToken::from_millinear(DEFAULT_DEPOSIT_AMOUNT), // 0.2 NEAR
        }
    }
}

#[near]
impl GlobalFactoryContract {
    /// Deploy a global contract with the given bytecode, identifiable by its code hash
    #[payable]
    pub fn deploy(&mut self, name: String) -> Promise {
        // Assert enough tokens are attached to cover minimal initial deposit on created account
        let attached = env::attached_deposit();
        let minimum_needed = self.min_deposit_amount.exact_amount_display();
        assert!(
            attached.ge(&self.min_deposit_amount),
            "Attach at least {minimum_needed}"
        );

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
                let code_hash_vec = bs58::decode(code_hash).into_vec().unwrap();
                let code_hash_vec_array: [u8; 32] = code_hash_vec.try_into().unwrap();
                promise.use_global_contract(code_hash_vec_array)
            }
        }
    }
}
