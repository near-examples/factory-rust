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
    let root = sandbox.root_account().unwrap();

    let token_owner_account = root
        .create_subaccount("the-token-owner-account-1234567890123456789")
        .initial_balance(NearToken::from_near(5))
        .transact()
        .await?
        .into_result()?;

    let alice_account = root
        .create_subaccount("alice")
        .initial_balance(NearToken::from_near(5))
        .transact()
        .await?
        .into_result()?;

    let bob_account = root
        .create_subaccount("bob")
        .initial_balance(NearToken::from_near(5))
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
    factory: &Contract,
    token_owner_account: &Account,
    alice_account: &Account,
    bob_account: &Account,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initial setup
    let symbol = "SOMETHING";
    let total_supply = U128(100);
    let token_id = symbol.to_ascii_lowercase();
    let metadata = FungibleTokenMetadata {
        spec: "ft-1.0.0".to_string(),
        name: "The Something Token".to_string(),
        symbol: symbol.to_string(),
        decimals: 6,
        icon: Some("data:image/svg+xml,%3Csvg width='111' height='90' viewBox='0 0 111 90' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M24.4825 0.862305H88.0496C89.5663 0.862305 90.9675 1.64827 91.7239 2.92338L110.244 34.1419C111.204 35.7609 110.919 37.8043 109.549 39.1171L58.5729 87.9703C56.9216 89.5528 54.2652 89.5528 52.6139 87.9703L1.70699 39.1831C0.305262 37.8398 0.0427812 35.7367 1.07354 34.1077L20.8696 2.82322C21.6406 1.60483 23.0087 0.862305 24.4825 0.862305ZM79.8419 14.8003V23.5597H61.7343V29.6329C74.4518 30.2819 83.9934 32.9475 84.0642 36.1425L84.0638 42.803C83.993 45.998 74.4518 48.6635 61.7343 49.3125V64.2168H49.7105V49.3125C36.9929 48.6635 27.4513 45.998 27.3805 42.803L27.381 36.1425C27.4517 32.9475 36.9929 30.2819 49.7105 29.6329V23.5597H31.6028V14.8003H79.8419ZM55.7224 44.7367C69.2943 44.7367 80.6382 42.4827 83.4143 39.4727C81.0601 36.9202 72.5448 34.9114 61.7343 34.3597V40.7183C59.7966 40.8172 57.7852 40.8693 55.7224 40.8693C53.6595 40.8693 51.6481 40.8172 49.7105 40.7183V34.3597C38.8999 34.9114 30.3846 36.9202 28.0304 39.4727C30.8066 42.4827 42.1504 44.7367 55.7224 44.7367Z' fill='%23009393'/%3E%3C/svg%3E".to_string()),
        reference: None,
        reference_hash: None,
    };

    let token_args = TokenArgs {
        owner_id: token_owner_account.id().clone(),
        total_supply,
        metadata,
    };

    // Getting required deposit based on provided arguments
    let required_deposit: U128 = factory
        .view("get_required")
        .args_json(json!({"args": token_args}))
        .await?
        .json()?;

    // Creating token with less than required deposit (should fail)
    let not_enough = alice_account
        .call(factory.id(), "create_token")
        .args_json(json!({"args": token_args}))
        .max_gas()
        .deposit(NearToken::from_yoctonear(required_deposit.0 - 1))
        .transact()
        .await?;
    assert!(not_enough.is_failure());

    // Creating token with the required deposit
    let alice_succeeds = alice_account
        .call(factory.id(), "create_token")
        .args_json(json!({"args": token_args}))
        .max_gas()
        .deposit(NearToken::from_yoctonear(required_deposit.0))
        .transact()
        .await?;
    assert!(alice_succeeds.json::<bool>()? == true);

    // Creating same token fails
    let bob_balance = bob_account.view_account().await?.balance;

    let bob_fails = bob_account
        .call(factory.id(), "create_token")
        .args_json(json!({"args": token_args}))
        .max_gas()
        .deposit(NearToken::from_yoctonear(required_deposit.0))
        .transact()
        .await?;

    let bob_balance_after = bob_account.view_account().await?.balance;
    let rest = bob_balance.saturating_sub(bob_balance_after).as_millinear();
    println!("{:?}", rest);

    // bob fails
    assert!(bob_fails.json::<bool>()? == false);

    // but it gets back the money (i.e. looses less than 0.005 N)
    assert!(rest < 5);

    // Checking created token account and metadata
    let token_account_id: AccountId = format!("{}.{}", token_id, factory.id()).parse().unwrap();
    let token_metadata: FungibleTokenMetadata = token_owner_account
        .view(&token_account_id, "ft_metadata")
        .args_json(json!({}))
        .await?
        .json()?;

    assert_eq!(token_metadata.symbol, symbol);

    // Checking token supply
    let token_total_supply: U128 = token_owner_account
        .view(&token_account_id, "ft_total_supply")
        .args_json(json!({}))
        .await?
        .json()?;
    assert_eq!(token_total_supply.0, total_supply.0);

    // Checking total supply belongs to the owner account
    let token_owner_balance: U128 = token_owner_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": token_owner_account.id()}))
        .await?
        .json()?;

    assert_eq!(token_owner_balance.0, total_supply.0);

    // Checking transferring tokens from owner to other account
    let _ = alice_account
        .call(&token_account_id, "storage_deposit")
        .args_json(json!({"account_id": alice_account.id()}))
        .max_gas()
        .deposit(NearToken::from_millinear(250))
        .transact()
        .await?;

    let alice_balance_before: U128 = alice_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": alice_account.id()}))
        .await?
        .json()?;
    assert_eq!(alice_balance_before.0, 0);

    let _ = token_owner_account
        .call(&token_account_id, "ft_transfer")
        .args_json(json!({
            "receiver_id": alice_account.id(),
            "amount": "2",
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    let alice_balance_after: U128 = alice_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": alice_account.id()}))
        .await?
        .json()?;
    assert_eq!(alice_balance_after.0, 2);

    // Checking transferring token from alice to bob
    let _ = bob_account
        .call(&token_account_id, "storage_deposit")
        .args_json(json!({"account_id": bob_account.id()}))
        .max_gas()
        .deposit(NearToken::from_millinear(250))
        .transact()
        .await?;
    let bob_balance_before: U128 = bob_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": bob_account.id()}))
        .await?
        .json()?;
    assert_eq!(bob_balance_before.0, 0);

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
    let bob_balance_after: U128 = bob_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": bob_account.id()}))
        .await?
        .json()?;
    assert_eq!(bob_balance_after.0, 1);

    let alice_balance_after: U128 = alice_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": alice_account.id()}))
        .await?
        .json()?;
    assert_eq!(alice_balance_after.0, 1);

    // Checking total supply belongs to the owner account
    let token_owner_balance: U128 = token_owner_account
        .view(&token_account_id, "ft_balance_of")
        .args_json(json!({"account_id": token_owner_account.id()}))
        .await?
        .json()?;

    assert_eq!(token_owner_balance.0, total_supply.0 - 2);

    Ok(())
}
