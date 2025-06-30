# Overview
This is a demo of a simple valence program that forwards 100 tokens from one account to another.

# Usage

## Setup Environment
Install the following tools:
 - Foundry: [https://getfoundry.sh](https://getfoundry.sh)
 - cargo-valence v0.3.1 (replace the tag in the install instructions if needed): [https://github.com/timewave-computer/valence-coprocessor/tree/main](https://github.com/timewave-computer/valence-coprocessor/tree/main)
 - cargo

## Startup Anvil from mainnet
```bash
anvil -f https://eth-mainnet.public.blastapi.io
```

## Deploy Contracts
```bash
cargo run --bin deploy
```
Record the Authorization and Forwarder contract addresses in the relevant constants in [./src/lib.rs](./src/lib.rs).
Set the variable `FORWARDER_LIBRARY_CONTRACT` in [./coprocessor-app/crates/circuit/lib.rs](./coprocessor-app/crates/circuit/lib.rs) with the Forwarder contract address printed in the logs.

Record the DEMO Token address and the Send and Deposit account addresses printed at the top of the log.
Also record the DEO

## Query send account balance
```bash
cast call <DEMO Token address> 'balanceOf(address)(uint256)' <Send Account Address> --rpc-url http://localhost:8545
```
There should be a balance of 150.

## Deploy the Coprocessor App
```bash
cd coprocessor-app
cargo-valence --socket prover.timewave.computer:37281 \
  deploy circuit \
  --controller ./crates/controller \
  --circuit valence-coprocessor-app-circuit
cd ..
```
Record the ID inside the `controller` attribute of the JSON output
in the `COPROCESSOR_APP_ID` constant in [./src/lib.rs](./src/lib.rs).

## Intitialize and Execute Contracts
```bash
cargo run --bin activate
```
Then query the Send and Deposit account balances
```bash
cast call <DEMO Token address> 'balanceOf(address)(uint256)' <Send Account Address> --rpc-url http://localhost:8545
cast call <DEMO Token address> 'balanceOf(address)(uint256)' <Deposit Account Address> --rpc-url http://localhost:8545
```
TThe Send account show now have a balance of 50 and the Deposit account should have a balance of 100.

## Without ZK
It is also possible to use this demo without ZK proofs or the coprocessor.
Deploy the contracts as explained above and update [./src/lib.rs](./src/lib.rs).
Then to activate run the following:
```bash
cargo run --bin nonzk-activate
```

