const DEFAULT_GLOBAL_CONTRACT_ACCOUNT_ID: &str = "ft.globals.primitives.testnet";
const DEFAULT_GLOBAL_CONTRACT_HASH: &str = "3vaopJ7aRoivvzZLngPQRBEd8VJr2zPLTxQfnRCoFgNX";

/// TODO: add tests for deploy method as soon as near-workspaces-rs supports deploying global contracts.
/// Currently it does not, therefore it's impossible to deploy global contract to use it in tests.

/// Test management of global contract ID
#[tokio::test]
async fn test_manager() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox_with_version("2.7.0").await?;
    let factory_wasm = near_workspaces::compile_project(".").await?;
    let factory_contract = worker.dev_deploy(&factory_wasm).await?;

    let change_contract_id_res_1 = factory_contract
        .call("update_global_contract_id")
        .args_json((DEFAULT_GLOBAL_CONTRACT_HASH.to_string(),))
        .max_gas()
        .transact()
        .await?;
    println!("change_contract_id_res: {change_contract_id_res_1:?}");
    assert!(change_contract_id_res_1.is_success());

    let global_contract_id = factory_contract
        .call("get_global_contract_id")
        .args_json(())
        .view()
        .await?
        .json::<Option<String>>()?
        .expect("Should have stored global contract ID");
    assert_eq!(global_contract_id, DEFAULT_GLOBAL_CONTRACT_HASH);

    let change_contract_id_res_2 = factory_contract
        .call("update_global_contract_id")
        .args_json((DEFAULT_GLOBAL_CONTRACT_ACCOUNT_ID.to_string(),))
        .max_gas()
        .transact()
        .await?;
    println!("change_contract_id_res: {change_contract_id_res_2:?}");
    assert!(change_contract_id_res_2.is_success());

    let global_contract_id = factory_contract
        .call("get_global_contract_id")
        .args_json(())
        .view()
        .await?
        .json::<Option<String>>()?
        .expect("Should have stored global contract ID");
    assert_eq!(global_contract_id, DEFAULT_GLOBAL_CONTRACT_ACCOUNT_ID);
    Ok(())
}

/// Test error cases and edge conditions
#[tokio::test]
async fn test_global_contract_edge_cases() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox_with_version("2.7.0").await?;
    let factory_wasm = near_workspaces::compile_project(".").await?;
    let factory_contract = worker.dev_deploy(&factory_wasm).await?;

    let change_contract_id_res = factory_contract
        .call("update_global_contract_id")
        .args_json(("11111111111111111111111111111111".to_string(),))
        .max_gas()
        .transact()
        .await?;
    assert!(change_contract_id_res.is_success());

    // Test using non-existent global contract
    let res = factory_contract
        .call("deploy")
        .args_json(("new_ft",))
        .max_gas()
        .transact()
        .await?;
    assert!(
        res.is_failure(),
        "Not failed to use global contract by hash: {res:?}"
    );

    Ok(())
}
