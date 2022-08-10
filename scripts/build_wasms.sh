#!/bin/bash
# This script builds the genesis head and wasm files for registering a parachain
# Run this script from the parent directory!

# Obtain Wasm runtime validation function
./target/release/parachain-collator export-genesis-wasm --chain ../specs/rococo-local-parachain-2000-raw.json > para-2000-wasm

# Generate a parachain genesis state
./target/release/parachain-collator export-genesis-state --chain ../specs/rococo-local-parachain-2000-raw.json > para-2000-genesis


