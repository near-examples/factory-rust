#!/bin/sh

echo ">> Building contracts"

rustup target add wasm32-unknown-unknown
cargo build -p donation-conrtact --target wasm32-unknown-unknown --release
cargo build -p donation-factory --target wasm32-unknown-unknown --release
