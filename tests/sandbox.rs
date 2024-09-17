use near_workspaces::types::{AccountId, NearToken};
use serde_json::json;

const TEN_NEAR: NearToken = NearToken::from_near(10);

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    // Create accounts
    let alice = create_subaccount(&root, "alice").await?;
    let bob = create_subaccount(&root, "bob").await?;

    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    // Launch new donation contract through factory
    let res = alice
        .call(contract.id(), "create_factory_subaccount_and_deploy")
        .args_json(json!({"name": "donation_for_alice", "beneficiary": alice.id()}))
        .max_gas()
        .deposit(NearToken::from_millinear(1700))
        .transact()
        .await?;

    assert!(res.is_success());

    let sub_accountid: AccountId = format!("donation_for_alice.{}", contract.id())
        .parse()
        .unwrap();

    let res = bob
        .view(&sub_accountid, "get_beneficiary")
        .args_json({})
        .await?;

    assert_eq!(res.json::<AccountId>()?, alice.id().clone());

    let res = bob
        .call(&sub_accountid, "donate")
        .args_json({})
        .max_gas()
        .deposit(NearToken::from_near(5))
        .transact()
        .await?;

    assert!(res.is_success());

    // Try to create new donation contract with insufficient deposit
    let res = alice
        .call(contract.id(), "create_factory_subaccount_and_deploy")
        .args_json(json!({"name": "donation_for_alice_2", "beneficiary": alice.id()}))
        .max_gas()
        .deposit(NearToken::from_millinear(1500))
        .transact()
        .await?;

    assert!(res.is_failure());

    Ok(())
}

async fn create_subaccount(
    root: &near_workspaces::Account,
    name: &str,
) -> Result<near_workspaces::Account, Box<dyn std::error::Error>> {
    let subaccount = root
        .create_subaccount(name)
        .initial_balance(TEN_NEAR)
        .transact()
        .await?
        .unwrap();

    Ok(subaccount)
}