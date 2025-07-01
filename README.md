# Overview
This is a demo of a simple valence program that forwards 100 tokens from one account (called the "Send" account) to another (called the "Deposit" account in this demo).

Once the program has been deployed, the send account is loaded with 1000 DEMO tokens. This README describes two methods for activating the forwarder to send tokens from the Send account to the Deposit account. The first method relies on the zk-coprocessor, while the second uses a simple on-chain authorization check.

# Table of Contents
- [Key Components](#key-components)
  - [Smart Contracts](#smart-contracts)
  - [Rust scripts](#rust-scripts)
  - [Coprocessor App (ZK Proof Generation)](#coprocessor-app-zk-proof-generation)
- [Usage](#usage)
  - [Setup Environment](#setup-environment)
  - [Startup Anvil from mainnet](#startup-anvil-from-mainnet)
  - [Deploy Contracts](#deploy-contracts)
  - [Query send account balance](#query-send-account-balance)
  - [Deploy the Coprocessor App](#deploy-the-coprocessor-app)
  - [Initialize and Execute Contracts](#intitialize-and-execute-contracts)
  - [Without ZK](#without-zk)

# Key Components

## Smart Contracts
Valence contracts
- **`BaseAccount`**: Account contracts that hold tokens and can approve libraries
- **`Forwarder`**: The main contract that handles token forwarding logic
- **`Authorization`**: Manages ZK proof verification and authorization
- **`LiteProcessor`**: Processes messages and executes forwarding operations
- **`SP1VerificationGateway`**: Verifies SP1 (Succinct Labs) ZK proofs

Other EVM contracts
- **`MockERC20`**: A test ERC20 token called "DEMO" for the demonstration
- **`ERC1967Proxy`**: Upgradeable proxy pattern for the verification gateway

## Rust scripts
Three key binaries:

- **`deploy`**: Sets up the entire system by:
  - Deploying all smart contracts
  - Creating send and deposit accounts
  - Minting 1000 DEMO tokens to the send account
  - Configuring the forwarder with transfer parameters
  - Setting up authorization and verification systems

- **`activate`**: Executes the ZK-proof-based forwarding by:
  - Generating a ZK proof using the coprocessor
  - Submitting the proof to the authorization contract
  - Triggering the token transfer (100 tokens from send to deposit account)

- **`nonzk-activate`**: Alternative execution without ZK proofs

## Coprocessor App (ZK Proof Generation)
Located in `coprocessor-app/`, this generates ZK proofs that validate the token transfer operation:
- **`circuit`**: Defines the ZK circuit logic for token transfer validation
- **`controller`**: Manages the proof generation process
- **`domain`**: Handles state proof management for the coprocessor. Not used in this program.

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
Set the variable `FORWARDER_LIBRARY_CONTRACT` in [./coprocessor-app/crates/circuit/src/lib.rs](./coprocessor-app/crates/circuit/src/lib.rs) with the Forwarder contract address printed in the logs.

Record the DEMO Token address and the Send and Deposit account addresses printed at the top of the log.

## Query send account balance
```bash
cast call <DEMO Token address> 'balanceOf(address)(uint256)' <Send Account Address> --rpc-url http://localhost:8545
```
There should be a balance of 1000 DEMO tokens in the send account.

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
> If you see `Error: error decoding response body`, we recommend running the above step again. This is a known issue while making calls to the co-processor.

Then query the Send and Deposit account balances
```bash
cast call <DEMO Token address> 'balanceOf(address)(uint256)' <Send Account Address> --rpc-url http://localhost:8545
cast call <DEMO Token address> 'balanceOf(address)(uint256)' <Deposit Account Address> --rpc-url http://localhost:8545
```
The Send account should decrease by 100 and the Deposit account should increase by 100.

## Without ZK
It is also possible to use this demo without ZK proofs or the coprocessor.
Deploy the contracts as explained above and update [./src/lib.rs](./src/lib.rs).
Then to activate run the following:
```bash
cargo run --bin nonzk-activate
```

