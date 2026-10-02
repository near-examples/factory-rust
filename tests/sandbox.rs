use near_api::{
    types::account::ContractState, Account, AccountId, Contract, NearGas, NearToken, NetworkConfig,
    Signer,
};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    base64::{engine::general_purpose::STANDARD, Engine},
    json_types::U128,
    near,
};
use serde_json::json;
use std::sync::{Arc, OnceLock};
use testresult::{TestError, TestResult};

const PUBLISHED_FT: &str = "ft.globals.primitives.near";
const SANDBOX_FT: &str = "global.sandbox";

#[near(serializers = [json, borsh])]
struct TokenArgs {
    owner_id: AccountId,
    total_supply: U128,
    metadata: FungibleTokenMetadata,
}

struct TestContext {
    _sandbox: near_sandbox::Sandbox,
    network: NetworkConfig,
    signer: Arc<Signer>,
    factory: Contract,
    global_contract_id: AccountId,
    owner: Account,
    alice: Account,
    bob: Account,
}

fn factory_wasm() -> &'static Vec<u8> {
    static WASM: OnceLock<Vec<u8>> = OnceLock::new();
    WASM.get_or_init(|| {
        let path = cargo_near_build::build_with_cli(Default::default()).expect("Build factory");
        std::fs::read(path).expect("Read factory WASM")
    })
}

async fn published_ft_wasm() -> TestResult<&'static Vec<u8>> {
    static WASM: tokio::sync::OnceCell<Vec<u8>> = tokio::sync::OnceCell::const_new();
    WASM.get_or_try_init(|| async {
        // Fetch the actual published FT implementation; no local FT project or WASM is needed.
        let code = Contract::global_wasm()
            .by_account_id(PUBLISHED_FT.parse()?)
            .fetch_from_mainnet()
            .await?
            .data;
        Ok::<_, TestError>(STANDARD.decode(code.code_base64)?)
    })
    .await
}

async fn setup(initialize: bool) -> TestResult<TestContext> {
    setup_with_global(initialize, SANDBOX_FT).await
}

async fn setup_with_global(initialize: bool, global_id: &str) -> TestResult<TestContext> {
    let wasm = factory_wasm().clone();
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let network = NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let signer = Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    )?;
    for name in [
        "factory.sandbox",
        "owner.sandbox",
        "alice.sandbox",
        "bob.sandbox",
        global_id,
    ] {
        sandbox
            .create_account(name.parse()?)
            .initial_balance(NearToken::from_near(100))
            .send()
            .await?;
    }
    let factory = Contract("factory.sandbox".parse()?);
    let deployment = Contract::deploy(factory.account_id().clone()).use_code(wasm);
    let deployment = if initialize {
        deployment
            .with_init_call("new", json!({"global_contract_id": global_id}))?
            .with_signer(signer.clone())
    } else {
        deployment.without_init_call().with_signer(signer.clone())
    };
    deployment.send_to(&network).await?.assert_success();
    Ok(TestContext {
        _sandbox: sandbox,
        network,
        signer,
        factory,
        global_contract_id: global_id.parse()?,
        owner: Account("owner.sandbox".parse()?),
        alice: Account("alice.sandbox".parse()?),
        bob: Account("bob.sandbox".parse()?),
    })
}

async fn publish_ft(ctx: &TestContext) -> TestResult<()> {
    Contract::deploy_global_contract_code(published_ft_wasm().await?.clone())
        .as_account_id(ctx.global_contract_id.clone())
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.network)
        .await?
        .assert_success();
    Ok(())
}

fn token_args(ctx: &TestContext, symbol: &str) -> TokenArgs {
    TokenArgs {
        owner_id: ctx.owner.account_id().clone(),
        total_supply: U128(100),
        metadata: FungibleTokenMetadata {
            spec: "ft-1.0.0".to_string(),
            name: "The Something Token".to_string(),
            symbol: symbol.to_string(),
            decimals: 6,
            icon: None,
            reference: None,
            reference_hash: None,
        },
    }
}

async fn required(ctx: &TestContext, args: &TokenArgs) -> TestResult<NearToken> {
    Ok(ctx
        .factory
        .call_function("get_required", json!({"args": args}))
        .read_only()
        .fetch_from(&ctx.network)
        .await?
        .data)
}

async fn create(
    ctx: &TestContext,
    user: &Account,
    args: &TokenArgs,
    deposit: NearToken,
) -> TestResult<bool> {
    Ok(ctx
        .factory
        .call_function("create_token", json!({"args": args}))
        .transaction()
        .deposit(deposit)
        .gas(NearGas::from_tgas(300))
        .with_signer(user.account_id().clone(), ctx.signer.clone())
        .send_to(&ctx.network)
        .await?
        .assert_success()
        .json()?)
}

async fn ft_balance(ctx: &TestContext, token: &Contract, user: &Account) -> TestResult<U128> {
    Ok(token
        .call_function("ft_balance_of", json!({"account_id": user.account_id()}))
        .read_only()
        .fetch_from(&ctx.network)
        .await?
        .data)
}

#[tokio::test]
async fn test_create_token_and_transfers() -> TestResult<()> {
    let ctx = setup(true).await?;
    publish_ft(&ctx).await?;
    let args = token_args(&ctx, "SOMETHING");
    let deposit = required(&ctx, &args).await?;

    assert_eq!(deposit, NearToken::from_yoctonear(0));
    assert!(create(&ctx, &ctx.alice, &args, deposit).await?);

    let token_account = Account(format!("something.{}", ctx.factory.account_id()).parse()?);
    let token = token_account.as_contract();
    let account = token_account.view().fetch_from(&ctx.network).await?.data;
    assert_eq!(
        account.contract_state,
        ContractState::GlobalAccountId(SANDBOX_FT.parse()?)
    );
    assert_eq!(deposit, minimum_storage_deposit(account.storage_usage));
    assert!(token_account
        .list_keys()
        .fetch_from(&ctx.network)
        .await?
        .data
        .is_empty());

    let metadata: FungibleTokenMetadata = token
        .call_function("ft_metadata", ())
        .read_only()
        .fetch_from(&ctx.network)
        .await?
        .data;
    assert_eq!(
        serde_json::to_value(&metadata)?,
        serde_json::to_value(&args.metadata)?
    );
    let supply: U128 = token
        .call_function("ft_total_supply", ())
        .read_only()
        .fetch_from(&ctx.network)
        .await?
        .data;
    assert_eq!(supply.0, args.total_supply.0);
    assert_eq!(ft_balance(&ctx, &token, &ctx.owner).await?.0, supply.0);

    let before = ctx.bob.view().fetch_from(&ctx.network).await?.data.amount;
    assert!(!create(&ctx, &ctx.bob, &args, NearToken::from_millinear(100)).await?);
    let after = ctx.bob.view().fetch_from(&ctx.network).await?.data.amount;
    assert!(
        before.saturating_sub(after) < NearToken::from_millinear(10),
        "Duplicate creation must refund the deposit"
    );

    for user in [&ctx.alice, &ctx.bob] {
        token
            .call_function("storage_deposit", json!({"account_id": user.account_id()}))
            .transaction()
            .deposit(NearToken::from_millinear(250))
            .with_signer(user.account_id().clone(), ctx.signer.clone())
            .send_to(&ctx.network)
            .await?
            .assert_success();
        assert_eq!(ft_balance(&ctx, &token, user).await?.0, 0);
    }
    for (sender, receiver, amount) in [(&ctx.owner, &ctx.alice, "2"), (&ctx.alice, &ctx.bob, "1")] {
        token
            .call_function(
                "ft_transfer",
                json!({"receiver_id": receiver.account_id(), "amount": amount}),
            )
            .transaction()
            .deposit(NearToken::from_yoctonear(1))
            .with_signer(sender.account_id().clone(), ctx.signer.clone())
            .send_to(&ctx.network)
            .await?
            .assert_success();
    }
    assert_eq!(ft_balance(&ctx, &token, &ctx.owner).await?.0, 98);
    assert_eq!(ft_balance(&ctx, &token, &ctx.alice).await?.0, 1);
    assert_eq!(ft_balance(&ctx, &token, &ctx.bob).await?.0, 1);
    Ok(())
}

#[tokio::test]
async fn test_required_covers_large_metadata() -> TestResult<()> {
    let ctx = setup(true).await?;
    publish_ft(&ctx).await?;
    let mut args = token_args(&ctx, "LARGE");
    let base = required(&ctx, &args).await?;
    args.metadata.icon = Some(format!("data:image/svg+xml,{}", "x".repeat(20_000)));
    let deposit = required(&ctx, &args).await?;
    assert!(deposit > base);
    let insufficient = ctx
        .factory
        .call_function("create_token", json!({"args": args}))
        .transaction()
        .deposit(NearToken::from_yoctonear(deposit.as_yoctonear() - 1))
        .gas(NearGas::from_tgas(300))
        .with_signer(ctx.alice.account_id().clone(), ctx.signer.clone())
        .send_to(&ctx.network)
        .await?;
    assert!(insufficient.is_failure());
    assert!(format!("{insufficient:?}").contains("Attach at least"));
    assert!(create(&ctx, &ctx.alice, &args, deposit).await?);
    let token = Account(format!("large.{}", ctx.factory.account_id()).parse()?);
    let account = token.view().fetch_from(&ctx.network).await?.data;
    assert_eq!(deposit, minimum_storage_deposit(account.storage_usage));
    let metadata: FungibleTokenMetadata = token
        .as_contract()
        .call_function("ft_metadata", ())
        .read_only()
        .fetch_from(&ctx.network)
        .await?
        .data;
    assert_eq!(metadata.icon, args.metadata.icon);
    Ok(())
}

#[tokio::test]
async fn test_initialization_and_validation() -> TestResult<()> {
    let ctx = setup(false).await?;
    let args = token_args(&ctx, "TOKEN");
    let uninitialized = ctx
        .factory
        .call_function("get_required", json!({"args": args}))
        .read_only::<NearToken>()
        .fetch_from(&ctx.network)
        .await;
    assert!(uninitialized.is_err());
    assert!(format!("{uninitialized:?}").contains("not initialized"));
    ctx.factory
        .call_function("new", json!({"global_contract_id": SANDBOX_FT}))
        .transaction()
        .with_signer(ctx.factory.account_id().clone(), ctx.signer.clone())
        .send_to(&ctx.network)
        .await?
        .assert_success();
    let repeat = ctx
        .factory
        .call_function("new", json!({"global_contract_id": SANDBOX_FT}))
        .transaction()
        .with_signer(ctx.factory.account_id().clone(), ctx.signer.clone())
        .send_to(&ctx.network)
        .await?;
    assert!(repeat.is_failure());
    assert!(format!("{repeat:?}").contains("already been initialized"));

    let deposit = required(&ctx, &args).await?;
    let mut invalid_symbol = token_args(&ctx, "NOT-VALID");
    let mut invalid_metadata = token_args(&ctx, "INVALID");
    invalid_metadata.metadata.spec = "wrong".to_string();
    let mut too_long = token_args(&ctx, &"A".repeat(64));
    for (args, expected) in [
        (&mut invalid_symbol, "Invalid Symbol"),
        (&mut invalid_metadata, "require! assertion failed"),
        (&mut too_long, "Token Account ID is invalid"),
    ] {
        let outcome = ctx
            .factory
            .call_function("create_token", json!({"args": args}))
            .transaction()
            .deposit(deposit)
            .gas(NearGas::from_tgas(300))
            .with_signer(ctx.alice.account_id().clone(), ctx.signer.clone())
            .send_to(&ctx.network)
            .await?;
        assert!(outcome.is_failure());
        assert!(format!("{outcome:?}").contains(expected), "{outcome:?}");
    }
    Ok(())
}

fn minimum_storage_deposit(storage_bytes: u64) -> NearToken {
    if storage_bytes <= 770 {
        NearToken::from_yoctonear(0)
    } else {
        NearToken::from_yoctonear(storage_bytes as u128 * 10_u128.pow(19))
    }
}

#[tokio::test]
async fn test_required_at_zero_balance_boundary() -> TestResult<()> {
    let ctx = setup(true).await?;
    publish_ft(&ctx).await?;
    for bytes in [769, 770, 771] {
        let mut args = token_args(&ctx, &format!("EDGE{bytes}"));
        args.metadata.icon = Some(String::new());
        let base_bytes =
            261 + ctx.global_contract_id.as_bytes().len() + near_sdk::borsh::to_vec(&args)?.len();
        args.metadata.icon = Some("x".repeat(bytes - base_bytes));
        let deposit = required(&ctx, &args).await?;
        assert!(create(&ctx, &ctx.alice, &args, deposit).await?);
        let token = Account(format!("edge{bytes}.{}", ctx.factory.account_id()).parse()?);
        let account = token.view().fetch_from(&ctx.network).await?.data;
        assert_eq!(account.storage_usage, bytes as u64);
        assert_eq!(deposit, minimum_storage_deposit(account.storage_usage));
    }
    Ok(())
}

#[tokio::test]
async fn test_required_with_variable_fields() -> TestResult<()> {
    let global = format!("{}.sandbox", "g".repeat(56));
    let ctx = setup_with_global(true, &global).await?;
    publish_ft(&ctx).await?;
    for (index, owner) in ["a.near".to_string(), format!("{}.near", "o".repeat(59))]
        .into_iter()
        .enumerate()
    {
        let mut args = token_args(&ctx, &format!("FIELDS{index}"));
        args.owner_id = owner.parse()?;
        args.total_supply = U128(u128::MAX);
        args.metadata.name = "Token 🪙 日本語".repeat(40);
        args.metadata.icon = Some(String::new());
        args.metadata.reference = Some("https://example.com/token.json".to_string());
        args.metadata.reference_hash = Some(near_sdk::json_types::Base64VecU8(vec![0; 32]));
        let deposit = required(&ctx, &args).await?;
        assert!(create(&ctx, &ctx.alice, &args, deposit).await?);
        let token = Account(format!("fields{index}.{}", ctx.factory.account_id()).parse()?);
        let account = token.view().fetch_from(&ctx.network).await?.data;
        assert_eq!(deposit, minimum_storage_deposit(account.storage_usage));
        let balance: U128 = token
            .as_contract()
            .call_function("ft_balance_of", json!({"account_id": args.owner_id}))
            .read_only()
            .fetch_from(&ctx.network)
            .await?
            .data;
        assert_eq!(balance.0, args.total_supply.0);
    }
    Ok(())
}
