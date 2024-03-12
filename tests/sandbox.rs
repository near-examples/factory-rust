use near_workspaces::types::{KeyType, NearToken, SecretKey};
use serde_json::json;

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    let alice_account = sandbox
        .create_tla(
            "alice".parse().unwrap(),
            SecretKey::from_random(KeyType::ED25519),
        )
        .await?;

    let res = contract
        .call("create_factory_subaccount_and_deploy")
        .args_json(json!({"name": "donation", "beneficiary": alice_account.id()}))
        .max_gas()
        .deposit(NearToken::from_near(5))
        .transact()
        .await?;

    print!("{:?}", alice_account.account.id());

    assert!(res.is_success());

    // let res = contract
    //     .call("complex_call")
    //     .args_json((status_id, message))
    //     .max_gas()
    //     .transact()
    //     .await?;
    // assert!(res.is_success());
    // let value = res.json::<String>()?;
    // assert_eq!(message, value.trim_matches(|c| c == '"'));

    Ok(())
}
// use serde_json::json;

// #[tokio::test]
// async fn test_contract_is_operational() ->  {
//     let sandbox = near_workspaces::sandbox().await?;
//     let contract_wasm = near_workspaces::compile_project("./").await?;

//     let contract = sandbox.dev_deploy(&contract_wasm).await?;

//     let user_account = sandbox.dev_create_account().await?;

//     let outcome = user_account
//         .call(contract.id(), "set_greeting")
//         .args_json(json!({"greeting": "Hello World!"}))
//         .transact()
//         .await?;
//     assert!(outcome.is_success());

//     let user_message_outcome = contract
//         .view("get_greeting")
//         .args_json(json!({}))
//         .await?;
//     assert_eq!(user_message_outcome.json::<String>()?, "Hello World!");

//     Ok(())
// }
