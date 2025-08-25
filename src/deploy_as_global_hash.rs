use near_sdk::{
    env, json_types::Base58CryptoHash, log, near, AccountId, CryptoHash, NearToken, Promise,
    PromiseError, PublicKey,
};

use crate::{Contract, ContractExt, NEAR_PER_STORAGE};

#[near]
impl Contract {
    #[payable]
    pub fn deploy_as_global_hash(
        &mut self,
        name: String,
        public_key: Option<PublicKey>,
    ) -> Promise {
        // Assert the sub-account is valid
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{name}.{current_account}").parse().unwrap();
        assert!(
            env::is_valid_account_id(subaccount.as_bytes()),
            "Invalid subaccount"
        );

        let code = self.code.clone().unwrap();

        // Assert enough tokens are attached to create the account and deploy the contract
        let attached = env::attached_deposit();
        let contract_bytes = code.len() as u128;
        let contract_deploying_cost = NEAR_PER_STORAGE
            .saturating_mul(contract_bytes)
            .saturating_mul(10); // The cost per byte of global contract code is set as 10x the storage staking cost per byte

        assert!(
            attached >= contract_deploying_cost,
            "Attach at least {contract_deploying_cost} yⓃ"
        );

        let hash: CryptoHash = env::sha256(&code).try_into().unwrap();
        let hash_str = String::from(&Base58CryptoHash::from(hash));

        let pk = public_key.unwrap_or(env::signer_account_pk());

        let promise = Promise::new(subaccount.clone())
            .create_account()
            .transfer(env::attached_deposit())
            .add_full_access_key(pk)
            .deploy_global_contract(code);

        // Add callback
        promise.then(
            Self::ext(env::current_account_id()).deploy_as_global_hash_callback(
                subaccount,
                env::predecessor_account_id(),
                attached,
                hash_str,
            ),
        )
    }

    #[private]
    pub fn deploy_as_global_hash_callback(
        &mut self,
        account: AccountId,
        user: AccountId,
        attached: NearToken,
        hash: String,
        #[callback_result] create_deploy_result: Result<(), PromiseError>,
    ) -> bool {
        if let Ok(_result) = create_deploy_result {
            log!("Correctly created and deployed. Contract hash: {:?}", hash);
            return true;
        };

        log!("Error creating {account}, returning {attached}yⓃ to {user}");
        Promise::new(user).transfer(attached);
        false
    }
}
