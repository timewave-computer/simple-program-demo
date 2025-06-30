# Steps

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
Set the `AUTHORIZATION` and `FORWARDER` constants in [./src/lib.rs](./src/lib.rs) to the respective contract addresses based on the logs of the deploy script.
Also set the variable `FORWARDER_LIBRARY_CONTRACT` in [./coprocessor-app/crates/circuit/lib.rs](./coprocessor-app/crates/circuit/lib.rs) with the Forwarder contract address.

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
