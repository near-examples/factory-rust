use near_workspaces::types::{AccountId, KeyType, NearToken, SecretKey};
use serde_json::json;

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    let alice = sandbox
        .create_tla(
            "alice.test.near".parse().unwrap(),
            SecretKey::from_random(KeyType::ED25519),
        )
        .await?
        .unwrap();

    let bob = sandbox.dev_create_account().await?;

    let res = contract
        .call("create_factory_subaccount_and_deploy")
        .args_json(json!({"name": "donation_for_alice", "beneficiary": alice.id()}))
        .max_gas()
        .deposit(NearToken::from_near(5))
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

    Ok(())
}
