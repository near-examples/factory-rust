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

#[tokio::test]
async fn test_too_low_deposit() -> Result<(), Box<dyn std::error::Error>> {
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

    let res = contract
        .call("create_factory_subaccount_and_deploy")
        .args_json(json!({"name": "donation_for_alice", "beneficiary": alice.id()}))
        .max_gas()
        .deposit(NearToken::from_near(1)) // NOTE: this is less than 1.55 NEAR
        .transact()
        .await?;

    // 1.55 NEAR corresponds to size of donation contract, deployed by factory
    // Storage used by the account      154.9 KB
    // assert!(format!("{:?}", res.into_result().unwrap_err()).contains("Attach at least 1.55 NEAR"));
    // TODO: replace all of below with above line

    let res = res.into_result();
    match res {
        Err(err) => {
            // noop
            println!("we've hit the expected assert branch");
            assert!(format!("{:?}", err).contains("Attach at least 1.55 NEAR") );
            
        }, 
        Ok(_value) => {
            // NOTE: this branch is hit, if 
            // https://github.com/near-examples/factory-rust/blob/main/src/lib.rs#L8 
            // is reverted to NearToken::from_yoctonear(10u128.pow(18))
            let bob = sandbox.dev_create_account().await?;
            let sub_accountid: AccountId = format!("donation_for_alice.{}", contract.id())
                .parse()
                .unwrap();
            let view_res = bob
                .view(&sub_accountid, "get_beneficiary")
                .args_json({})
                .await;
            assert!(view_res.is_err()); // NOTE: this line becomes `is_ok()` if deposit is increased `1` -> `2 NEAR` 
            let dbg = format!("{:#?}", view_res.unwrap_err());

            // NOTE: donation contract wasn't deployed
            assert!(dbg.contains("donation_for_alice"));
            assert!(dbg.contains("UnknownAccount"));
            
        }
        
    }
    Ok(())

}
