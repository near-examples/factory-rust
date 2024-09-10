use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{json_types::U128, near};
use near_workspaces::{types::NearToken, Account, AccountId, Contract};
use serde_json::json;

#[near(serializers = [json, borsh])]
struct TokenArgs {
    owner_id: AccountId,
    total_supply: U128,
    metadata: FungibleTokenMetadata,
}

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = sandbox.dev_deploy(&contract_wasm).await?;
    let token_owner_account = sandbox.root_account().unwrap();
    let alice_account = token_owner_account
        .create_subaccount("alice")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await?
        .into_result()?;
    let bob_account = token_owner_account
        .create_subaccount("bob")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await?
        .into_result()?;

    create_token(
        &contract,
        &token_owner_account,
        &alice_account,
        &bob_account,
    )
    .await?;

    Ok(())
}

async fn create_token(
    contract: &Contract,
    token_owner_account: &Account,
    alice_account: &Account,
    bob_account: &Account,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initial setup
    let symbol = "TEST";
    let total_supply = U128(100);
    let token_id = symbol.to_ascii_lowercase();
    let metadata = FungibleTokenMetadata {
        spec: "ft-1.0.0".to_string(),
        name: "Test Token".to_string(),
        symbol: symbol.to_string(),
        decimals: 1,
        icon: None,
        reference: None,
        reference_hash: None,
    };
    let token_args = TokenArgs {
        owner_id: token_owner_account.id().clone(),
        total_supply,
        metadata,
    };

    // Getting required deposit based on provided arguments
    let required_deposit: serde_json::Value = contract
        .view("get_required")
        .args_json(json!({"args": token_args}))
        .await?
        .json()?;

    // Creating token with less than required deposit (should fail)
    let res_0 = contract
        .call("create_token")
        .args_json(json!({"args": token_args}))
        .max_gas()
        .deposit(NearToken::from_yoctonear(
          required_deposit.as_str().unwrap().parse::<u128>()? - 1,
        ))
        .transact()
        .await?;
    assert!(res_0.is_failure());

    // Creating token with the required deposit
    let res_1 = contract
        .call("create_token")
        .args_json(json!({"args": token_args}))
        .max_gas()
        .deposit(NearToken::from_yoctonear(
            required_deposit.as_str().unwrap().parse::<u128>()?,
        ))
        .transact()
        .await?;

    assert!(res_1.is_success());

    // Checking created token account and metadata
    let token_account_id: AccountId = format!("{}.{}", token_id, contract.id()).parse().unwrap();
    let token_metadata: FungibleTokenMetadata = token_owner_account
        .view(&token_account_id, "ft_metadata")
        .args_json(json!({}))
        .await?
        .json()?;

    assert_eq!(token_metadata.symbol, symbol);

    // Checking token supply
    let token_total_supply: serde_json::Value = token_owner_account
        .view(&token_account_id, "ft_total_supply")
        .args_json(json!({}))
        .await?
        .json()?;
    assert_eq!(
        token_total_supply.as_str().unwrap().parse::<u128>()?,
        u128::from(total_supply)
    );

    // Checking total supply belongs to the owner account
    let token_owner_balance: serde_json::Value = token_owner_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": token_owner_account.id()}))
        .await?
        .json()?;

    assert_eq!(
        token_owner_balance.as_str().unwrap().parse::<u128>()?,
        u128::from(total_supply)
    );

    // Checking transfering tokens from owner to other account
    let _ = alice_account
        .call(&token_account_id, "storage_deposit")
        .args_json(json!({"account_id": alice_account.id()}))
        .max_gas()
        .deposit(NearToken::from_millinear(250))
        .transact()
        .await?;
    let alice_balance_before: serde_json::Value = alice_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": alice_account.id()}))
        .await?
        .json()?;
    assert_eq!(alice_balance_before, "0");

    let _ = token_owner_account
        .call(&token_account_id, "ft_transfer")
        .args_json(json!({
            "receiver_id": alice_account.id(),
            "amount": "1",
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    let alice_balance_after: serde_json::Value = alice_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": alice_account.id()}))
        .await?
        .json()?;
    assert_eq!(alice_balance_after, "1");

    // Checking transfering token from alice to bob
    let _ = bob_account
        .call(&token_account_id, "storage_deposit")
        .args_json(json!({"account_id": bob_account.id()}))
        .max_gas()
        .deposit(NearToken::from_millinear(250))
        .transact()
        .await?;
    let bob_balance_before: serde_json::Value = bob_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": bob_account.id()}))
        .await?
        .json()?;
    assert_eq!(bob_balance_before, "0");
    let _ = alice_account
        .call(&token_account_id, "ft_transfer")
        .args_json(json!({
            "receiver_id": bob_account.id(),
            "amount": "1",
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    let bob_balance_after: serde_json::Value = bob_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": bob_account.id()}))
        .await?
        .json()?;
    assert_eq!(bob_balance_after, "1");
    Ok(())
}
