# Factory Contract Example

A factory is a smart contract that stores a compiled contract on itself, and
automatizes deploying it into sub-accounts.

This particular example presents a factory of donation contracts, and enables
to:

1. Create a sub-account of the factory and deploy the stored contract on it
   (create_factory_subaccount_and_deploy).
2. Change the stored contract using the update_stored_contract method.

```rust
#[payable]
    pub fn create_factory_subaccount_and_deploy(
        &mut self,
        name: String,
        beneficiary: AccountId,
        public_key: Option<PublicKey>,
    ) -> Promise {
        // Assert the sub-account is valid
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{name}.{current_account}").parse().unwrap();
        assert!(
            env::is_valid_account_id(subaccount.as_bytes()),
            "Invalid subaccount"
        );

        // Assert enough tokens are attached to create the account and deploy the contract
        let attached = env::attached_deposit();

        let code = self.code.clone().unwrap();
        let contract_bytes = code.len() as u128;
        let minimum_needed = NEAR_PER_STORAGE.saturating_mul(contract_bytes);
        assert!(
            attached >= minimum_needed,
            "Attach at least {minimum_needed} yⓃ"
        );

        let init_args = near_sdk::serde_json::to_vec(&DonationInitArgs { beneficiary }).unwrap();

        let mut promise = Promise::new(subaccount.clone())
            .create_account()
            .transfer(attached)
            .deploy_contract(code)
            .function_call(
                "init".to_owned(),
                init_args,
                NO_DEPOSIT,
                TGAS.saturating_mul(5),
            );

        // Add full access key is the user passes one
        if let Some(pk) = public_key {
            promise = promise.add_full_access_key(pk);
        }

        // Add callback
        promise.then(
            Self::ext(env::current_account_id()).create_factory_subaccount_and_deploy_callback(
                subaccount,
                env::predecessor_account_id(),
                attached,
            ),
        )
    }
```

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline. To deploy manually,
install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near deploy <account-id>
```

## How to Interact?

_In this example we will be using [NEAR CLI](https://github.com/near/near-cli)
to intract with the NEAR blockchain and the smart contract_

_If you want full control over of your interactions we recommend using the
[near-cli-rs](https://near.cli.rs)._

### Deploy the Stored Contract Into a Sub-Account

`create_factory_subaccount_and_deploy` will create a sub-account of the factory
and deploy the stored contract on it.

```bash
near call <factory-account> create_factory_subaccount_and_deploy '{ "name": "sub", "beneficiary": "<account-to-be-beneficiary>"}' --deposit 1.24 --accountId <account-id> --gas 300000000000000
```

This will create the `sub.<factory-account>`, which will have a `donation`
contract deployed on it:

```bash
near view sub.<factory-account> get_beneficiary
# expected response is: <account-to-be-beneficiary>
```

### Update the Stored Contract

`update_stored_contract` enables to change the compiled contract that the
factory stores.

The method is interesting because it has no declared parameters, and yet it
takes an input: the new contract to store as a stream of bytes.

To use it, we need to transform the contract we want to store into its `base64`
representation, and pass the result as input to the method:

```bash
# Use near-cli to update stored contract
export BYTES=`cat ./src/to/new-contract/contract.wasm | base64`
near call <factory-account> update_stored_contract "$BYTES" --base64 --accountId <factory-account> --gas 30000000000000
```

> This works because the arguments of a call can be either a `JSON` object or a
> `String Buffer`

## Factories - Explanations & Limitations

Factories are an interesting concept, here we further explain some of their
implementation aspects, as well as their limitations.

<br>

### Automatically Creating Accounts

NEAR accounts can only create sub-accounts of themselves, therefore, the
`factory` can only create and deploy contracts on its own sub-accounts.

This means that the factory:

1. **Can** create `sub.factory.testnet` and deploy a contract on it.
2. **Cannot** create sub-accounts of the `predecessor`.
3. **Can** create new accounts (e.g. `account.testnet`), but **cannot** deploy
   contracts on them.

It is important to remember that, while `factory.testnet` can create
`sub.factory.testnet`, it has no control over it after its creation.

### The Update Method

The `update_stored_contracts` has a very short implementation:

```rust
#[private]
    pub fn update_stored_contract(&mut self) {
        self.code.set(env::input());
    }
```

On first sight it looks like the method takes no input parameters, but we can
see that its only line of code reads from `env::input()`. What is happening here
is that `update_stored_contract` **bypasses** the step of **deserializing the
input**.

You could implement `update_stored_contract(&mut self, new_code: Vec<u8>)`,
which takes the compiled code to store as a `Vec<u8>`, but that would trigger
the contract to:

1. Deserialize the `new_code` variable from the input.
2. Sanitize it, making sure it is correctly built.

When dealing with big streams of input data (as is the compiled `wasm` file to
be stored), this process of deserializing/checking the input ends up **consuming
the whole GAS** for the transaction.

## Useful Links

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract
  development toolkit for Rust
- [near CLI-rs](https://near.cli.rs) - Iteract with NEAR blockchain from command
  line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub),
  [Twitter](https://twitter.com/neardevhub)
