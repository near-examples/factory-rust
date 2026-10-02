use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh, env, json_types::U128, log, near, require, serde_json::json, AccountId, NearToken,
    Promise, PromiseError,
};

use crate::{Contract, ContractExt, NO_DEPOSIT, TGAS};

type TokenId = String;

// Storage layout of near-examples/FT published at ft.globals.primitives.*:
// account: 100; STATE record: 79; metadata record: 41 + metadata bytes;
// owner balance record: 61 + owner ID bytes. TokenArgs already includes the
// metadata, owner ID, its 4-byte length, and a 16-byte total supply, so add
// 100 + 79 + 41 + 61 - 4 - 16 = 261 bytes, plus the global reference.
// Revisit this layout if the configured global FT implementation changes.
const FT_STORAGE_OVERHEAD: usize = 261;
const ZERO_BALANCE_STORAGE_LIMIT: usize = 770;

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
        let storage_bytes = self.global_contract_id.as_bytes().len()
            + FT_STORAGE_OVERHEAD
            + borsh::to_vec(args).unwrap().len();
        // NEAR exempts accounts using at most 770 bytes. Above that limit,
        // the whole storage footprint must be funded, not just the excess.
        if storage_bytes <= ZERO_BALANCE_STORAGE_LIMIT {
            NO_DEPOSIT
        } else {
            env::storage_byte_cost().saturating_mul(storage_bytes.try_into().unwrap())
        }
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

        let init_args = near_sdk::serde_json::to_vec(&args).unwrap();

        let user = env::predecessor_account_id();
        let callback_args = json!({ "user": user, "deposit": attached })
            .to_string()
            .into_bytes()
            .to_vec();

        Promise::new(token_account_id.parse().unwrap())
            .create_account()
            .transfer(attached)
            .use_global_contract_by_account_id(self.global_contract_id.clone())
            .function_call(
                "new".to_owned(),
                init_args,
                NO_DEPOSIT,
                TGAS.saturating_mul(50),
            )
            .then(Promise::new(env::current_account_id()).function_call(
                "create_callback".to_string(),
                callback_args,
                NearToken::from_near(0),
                TGAS.saturating_mul(30),
            ))
    }

    #[private]
    pub fn create_callback(
        &self,
        user: AccountId,
        deposit: NearToken,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> bool {
        match call_result {
            Ok(_) => true,
            Err(e) => {
                log!("Error creating token: {:?}", e);
                Promise::new(user).transfer(deposit).detach();
                false
            }
        }
    }
}
