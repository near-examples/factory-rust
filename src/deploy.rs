use near_sdk::{env, json_types::U128, log, near, require, AccountId, NearToken, Promise, PromiseError, PublicKey};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

use crate::{Contract, ContractExt, FT_CONTRACT, NEAR_PER_STORAGE, NO_DEPOSIT, TGAS};


#[near(serializers = [json])]
pub struct TokenArgs {
    owner_id: AccountId,
    total_supply: U128,
    metadata: FungibleTokenMetadata,
}

#[near]
impl Contract {

    fn get_required(&self, args: &TokenArgs) -> u128 {
        ((FT_WASM_CODE.len() + EXTRA_BYTES + args.try_to_vec().unwrap().len() * 2) as NearToken)
            * STORAGE_PRICE_PER_BYTE)
            .into()
    }

    #[payable]
    pub fn create_token(
        &mut self,
        args: TokenArgs,
    ) -> Promise {
        args.metadata.assert_valid();
        let token_id = args.metadata.symbol.to_ascii_lowercase();

        require!(is_valid_token_id(&token_id), "Invalid Symbol");

        // Assert the sub-account is valid
        let token_account_id = format!("{}.{}", token_id, env::current_account_id());
        assert!(
            env::is_valid_account_id(token_account_id.as_bytes()),
            "Token Account ID is invalid"
        );

        // Assert enough tokens are attached to create the account and deploy the contract
        let attached = env::attached_deposit();
        let required = self.get_required(&args);

        assert!(
            attached >= required,
            "Attach at least {minimum_needed} yⓃ"
        );

        let init_args = near_sdk::serde_json::to_vec(args).unwrap();

        let mut promise = Promise::new(subaccount.clone())
            .create_account()
            .transfer(attached)
            .deploy_contract(FT_CONTRACT)
            .function_call(
                "new".to_owned(),
                init_args,
                NO_DEPOSIT,
                TGAS.saturating_mul(50),
            );
    }

}
