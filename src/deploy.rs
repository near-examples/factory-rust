use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{borsh, env, json_types::U128, near, require, AccountId, NearToken, Promise};

use crate::{Contract, ContractExt, FT_CONTRACT, NO_DEPOSIT, TGAS};

type TokenId = String;

const EXTRA_BYTES: usize = 10000;

#[near(serializers = [json, borsh])]
pub struct TokenArgs {
    owner_id: AccountId,
    total_supply: U128,
    metadata: FungibleTokenMetadata,
}

pub fn is_valid_token_id(token_id: &TokenId) -> bool {
    for c in token_id.as_bytes() {
        match c {
            b'0'..=b'9' | b'a'..=b'z' => (),
            _ => return false,
        }
    }
    true
}

#[near]
impl Contract {
    pub fn get_required(&self, args: &TokenArgs) -> NearToken {
        env::storage_byte_cost().saturating_mul(
            (FT_CONTRACT.len() + EXTRA_BYTES + borsh::to_vec(args).unwrap().len() * 2)
                .try_into()
                .unwrap(),
        )
    }

    #[payable]
    pub fn create_token(&mut self, args: TokenArgs) -> Promise {
        args.metadata.assert_valid();
        let token_id = args.metadata.symbol.to_ascii_lowercase();

        require!(is_valid_token_id(&token_id), "Invalid Symbol");

        // Assert the sub-account is valid
        let token_account_id = format!("{}.{}", token_id, env::current_account_id());
        require!(
            env::is_valid_account_id(token_account_id.as_bytes()),
            "Token Account ID is invalid"
        );

        // Assert enough tokens are attached to create the account and deploy the contract
        let attached = env::attached_deposit();
        let required = self.get_required(&args);

        require!(
            attached >= required,
            format!("Attach at least {required} yⓃ")
        );

        let token_account_id: AccountId = format!("{}.{}", token_id, env::current_account_id())
            .parse()
            .unwrap();
        let init_args = near_sdk::serde_json::to_vec(&args).unwrap();

        Promise::new(token_account_id)
            .create_account()
            .transfer(attached)
            .deploy_contract(FT_CONTRACT.to_vec())
            .function_call(
                "new".to_owned(),
                init_args,
                NO_DEPOSIT,
                TGAS.saturating_mul(50),
            )
    }
}
