# Fungible Token Factory

This factory creates a subaccount for each token, attaches the shared global FT
contract, and initializes it with the supplied owner, total supply, and metadata.
It does not embed or build a local FT contract.

## Build and deploy

Install [cargo-near](https://github.com/near/cargo-near#installation), then run:

```bash
cargo near build
cargo near deploy
```

Initialize the factory during deployment by calling `new` with:

```json
{"global_contract_id": "ft.globals.primitives.testnet"}
```

Use `ft.globals.primitives.near` on mainnet. Initialization is required and can
only run once. Accounts referencing this global contract follow updates published
by its owner.

## Create a token

Both `get_required` and `create_token` accept the same `args` object:

```json
{
  "args": {
    "owner_id": "alice.testnet",
    "total_supply": "100000000",
    "metadata": {
      "spec": "ft-1.0.0",
      "name": "Example Token",
      "symbol": "EXAMPLE",
      "decimals": 6,
      "icon": null,
      "reference": null,
      "reference_hash": null
    }
  }
}
```

First call `get_required` as a view function. Its result is the required deposit
in yoctoNEAR, encoded as a JSON string. Attach that amount when calling
`create_token` with 300 Tgas. The symbol determines the subaccount:
`EXAMPLE` creates `example.<factory-account>`. Symbols must contain only ASCII
letters and digits, and the resulting account ID must be valid.

The owner receives the full initial supply, expressed in the token's smallest
units. Created token accounts have no access keys. `create_token` returns `true`
on success; failed account creation or FT initialization returns `false` and
refunds the attached deposit. Invalid arguments or insufficient deposits fail
the initial call.

`get_required` calculates initial storage for the published
[near-examples/FT implementation](https://github.com/near-examples/FT): serialized
token arguments + 261 bytes of fixed storage overhead + the global contract ID's
byte length. It returns zero for footprints of at most 770 bytes; larger
footprints require funding the entire storage size at the current storage byte
cost. Gas is paid separately. The calculation must be revisited if the global
FT's storage layout or protocol storage rules change.

Users register separately through the FT's `storage_deposit` method before
receiving transfers; their future storage is not included in the initial deposit.

## Test

```bash
cargo test
```

Tests use near-api and near-sandbox. They fetch the published FT code from
`ft.globals.primitives.near` once per test run and deploy it as a global contract
inside the local sandbox. Internet access to the mainnet RPC is required to fetch
the code; all transactions run locally.

Coverage includes explicit initialization, token metadata and supply, ownership
of the initial balance, storage registration and transfers, deposit enforcement,
duplicate creation refunds, invalid arguments, exact storage quotes for large
metadata, the 769/770/771-byte boundary, Unicode metadata, and variable owner and
global contract IDs.
