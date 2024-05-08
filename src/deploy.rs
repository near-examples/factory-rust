use near_sdk::serde::Serialize;
use near_sdk::{env, log, near, AccountId, NearToken, Promise, PromiseError, PublicKey};

use crate::{Contract, ContractExt, NEAR_PER_STORAGE, NO_DEPOSIT, TGAS};

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
struct DonationInitArgs {
    beneficiary: AccountId,
}

#[near]
impl Contract {
    #[payable]
    pub fn create_factory_subaccount_and_deploy(
        &mut self,
        name: String,
        beneficiary: AccountId,
        public_key: Option<PublicKey>,
    ) -> Promise {
        // Assert the sub-account is valid
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{name}.{current_account}").parse().unwrap();
        assert!(
            env::is_valid_account_id(subaccount.as_bytes()),
            "Invalid subaccount"
        );

        // Assert enough tokens are attached to create the account and deploy the contract
        let attached = env::attached_deposit();

        let code = self.code.clone().unwrap();
        let contract_bytes = code.len() as u128;
        let minimum_needed = NEAR_PER_STORAGE.saturating_mul(contract_bytes);
        assert!(
            attached >= minimum_needed,
            "Attach at least {minimum_needed}, {attached} was provided "
        );

        let init_args = near_sdk::serde_json::to_vec(&DonationInitArgs { beneficiary }).unwrap();

        let mut promise = Promise::new(subaccount.clone())
            .create_account()
            .transfer(attached)
            .deploy_contract(code)
            .function_call(
                "init".to_owned(),
                init_args,
                NO_DEPOSIT,
                TGAS.saturating_mul(5),
            );

        // Add full access key is the user passes one
        if let Some(pk) = public_key {
            promise = promise.add_full_access_key(pk);
        }

        // Add callback
        promise.then(
            Self::ext(env::current_account_id()).create_factory_subaccount_and_deploy_callback(
                subaccount,
                env::predecessor_account_id(),
                attached,
            ),
        )
    }

    #[private]
    pub fn create_factory_subaccount_and_deploy_callback(
        &mut self,
        account: AccountId,
        user: AccountId,
        attached: NearToken,
        #[callback_result] create_deploy_result: Result<(), PromiseError>,
    ) -> bool {
        match create_deploy_result {
            Ok(_result) => {
                log!(format!("Correctly created and deployed to {account}"));
                return true;
            }
            Err(returned_err) => {
                log!(format!("Bad error happened {:?}", returned_err));
            }
        }

        log!(format!(
            "Error creating {account}, returning {attached} to {user}"
        ));
        Promise::new(user).transfer(attached);
        false
    }
}
