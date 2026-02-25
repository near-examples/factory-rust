use factory_contract_global::GlobalContractId;
use near_api::AccountId;
use near_sdk::{serde_json::json, NearToken};

const DEFAULT_GLOBAL_CONTRACT_ACCOUNT_ID: &str = "ft.globals.primitives.testnet";

const TEST_GLOBAL_CONTRACT_ACCOUNT_ID: &str = "ft.globals.primitives.testnet";
const TEST_GLOBAL_CONTRACT_HASH: &str = "3vaopJ7aRoivvzZLngPQRBEd8VJr2zPLTxQfnRCoFgNX";
const TEST_DEPOSIT_AMOUNT: u128 = 100; // 0.1 NEAR

/// TODO: add tests for deploy method as soon as near-workspaces-rs supports deploying global contracts.
/// Currently it does not, therefore it's impossible to deploy global contract to use it in tests.

/// Test management of global contract ID
#[tokio::test]
async fn test_manager() -> anyhow::Result<()> {
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let factory_wasm_path = cargo_near_build::build_with_cli(Default::default()).unwrap();
    let factory_wasm = std::fs::read(factory_wasm_path)?;

    let factory_contract = create_subaccount(&sandbox, "factory.sandbox")
        .await
        .unwrap()
        .as_contract();

    // Initialize signer for the contract deployment
    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    // Deploy the base contract
    near_api::Contract::deploy(factory_contract.account_id().clone())
        .use_code(factory_wasm)
        .without_init_call()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let default_contract_id: GlobalContractId = factory_contract
        .call_function("get_global_contract_id", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(
        default_contract_id,
        GlobalContractId::AccountId(DEFAULT_GLOBAL_CONTRACT_ACCOUNT_ID.parse().unwrap())
    );

    factory_contract
        .call_function(
            "update_global_contract_id",
            json!({
              "contract_id": GlobalContractId::CodeHash(TEST_GLOBAL_CONTRACT_HASH.to_string()),
              "min_deposit": NearToken::from_millinear(TEST_DEPOSIT_AMOUNT)
            }),
        )
        .transaction()
        .max_gas()
        .with_signer(factory_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let global_contract_id: GlobalContractId = factory_contract
        .call_function("get_global_contract_id", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(
        global_contract_id,
        GlobalContractId::CodeHash(TEST_GLOBAL_CONTRACT_HASH.to_string())
    );

    factory_contract
        .call_function("update_global_contract_id", json!({
              "contract_id": GlobalContractId::AccountId(TEST_GLOBAL_CONTRACT_ACCOUNT_ID.parse().unwrap()),
              "min_deposit": NearToken::from_millinear(TEST_DEPOSIT_AMOUNT)
        }))
        .transaction()
        .max_gas()
        .with_signer(factory_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let global_contract_id: GlobalContractId = factory_contract
        .call_function("get_global_contract_id", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(
        global_contract_id,
        GlobalContractId::AccountId(TEST_GLOBAL_CONTRACT_ACCOUNT_ID.parse().unwrap())
    );

    let min_deposit: NearToken = factory_contract
        .call_function("get_min_deposit", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert!(min_deposit.eq(&NearToken::from_millinear(TEST_DEPOSIT_AMOUNT)));
    Ok(())
}

/// Test error cases and edge conditions
#[tokio::test]
async fn test_global_contract_edge_cases() -> anyhow::Result<()> {
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let factory_wasm_path = cargo_near_build::build_with_cli(Default::default()).unwrap();
    let factory_wasm = std::fs::read(factory_wasm_path)?;

    let factory_contract = create_subaccount(&sandbox, "factory.sandbox")
        .await
        .unwrap()
        .as_contract();

    // Initialize signer for the contract deployment
    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    // Deploy the base contract
    near_api::Contract::deploy(factory_contract.account_id().clone())
        .use_code(factory_wasm)
        .without_init_call()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    factory_contract
        .call_function("update_global_contract_id", json!({
            "contract_id": GlobalContractId::CodeHash("11111111111111111111111111111111".to_string()),
            "min_deposit": NearToken::from_millinear(TEST_DEPOSIT_AMOUNT)
        }))
        .transaction()
        .max_gas()
        .with_signer(factory_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Test using non-existent global contract
    factory_contract
        .call_function("deploy", json!({ "name": "new_ft" }))
        .transaction()
        .max_gas()
        .with_signer(factory_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_failure();

    Ok(())
}

async fn create_subaccount(
    sandbox: &near_sandbox::Sandbox,
    name: &str,
) -> testresult::TestResult<near_api::Account> {
    let account_id: AccountId = name.parse().unwrap();
    sandbox
        .create_account(account_id.clone())
        .initial_balance(NearToken::from_near(10))
        .send()
        .await?;
    Ok(near_api::Account(account_id))
}
