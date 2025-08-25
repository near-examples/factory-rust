use near_sdk::{env, log, near, AccountId, NearToken, Promise, PromiseError, PublicKey};

use crate::{Contract, ContractExt, NEAR_PER_STORAGE};

#[near]
impl Contract {
    #[payable]
    pub fn deploy_as_global_account_id(
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

        // Require a little more since storage cost is not exact
        // let minimum_needed = contract_deploying_cost.saturating_add(NearToken::from_millinear(100));
        assert!(
            attached >= contract_deploying_cost,
            "Attach at least {contract_deploying_cost} yⓃ"
        );

        let pk = public_key.unwrap_or(env::signer_account_pk());

        let promise = Promise::new(subaccount.clone())
            .create_account()
            .transfer(env::attached_deposit())
            .add_full_access_key(pk)
            .deploy_global_contract_by_account_id(code);

        // Add callback
        promise.then(
            Self::ext(env::current_account_id()).deploy_as_global_account_id_callback(
                subaccount,
                env::predecessor_account_id(),
                attached,
            ),
        )
    }

    #[private]
    pub fn deploy_as_global_account_id_callback(
        &mut self,
        account: AccountId,
        user: AccountId,
        attached: NearToken,
        #[callback_result] create_deploy_result: Result<(), PromiseError>,
    ) -> bool {
        if let Ok(_result) = create_deploy_result {
            log!("Correctly created and deployed to {account}");
            return true;
        };

        log!("Error creating {account}, returning {attached}yⓃ to {user}");
        Promise::new(user).transfer(attached);
        false
    }
}
