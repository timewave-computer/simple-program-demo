use std::error::Error;
use log::info;
use serde_json::json;
use alloy::{
    hex::FromHex,
    primitives::{Bytes, FixedBytes},
    providers::Provider,
};
use sp1_sdk::{HashableKey, SP1VerifyingKey};
use valence_domain_clients::{
    clients::{coprocessor::CoprocessorClient, ethereum::EthereumClient},
    coprocessor::base_client::CoprocessorBaseClient,
    evm::{base_client::EvmBaseClient, request_provider_client::RequestProviderClient},
};
use informal_program_demo::types::sol_types::Authorization;
use informal_program_demo::{
    AUTHORIZATION, FORWARDER, COPROCESSOR_APP_ID
};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();

    let mnemonic = "test test test test test test test test test test test junk";
    let rpc_url = "http://127.0.0.1:8545";

    let eth_client = EthereumClient::new(rpc_url, &mnemonic, None)?;
    let rp = eth_client.get_request_provider().await?;

    let authorization = Authorization::new(AUTHORIZATION, &rp);

    // Get the VK for the coprocessor app
    let coprocessor_client = CoprocessorClient::default();
    let program_vk = coprocessor_client
        .get_vk(COPROCESSOR_APP_ID)
        .await?;

    let sp1_program_vk: SP1VerifyingKey = bincode::deserialize(&program_vk)?;
    let program_vk = FixedBytes::<32>::from_hex(sp1_program_vk.bytes32()).unwrap();
    let registries = vec![0];
    let authorized_addresses = vec![FORWARDER];
    let vks = vec![program_vk];

    // Remember we send arrays because we allow  multiple registries added at once
    let tx = authorization
        .addRegistries(registries, vec![authorized_addresses], vks, vec![false])
        .into_transaction_request();

    // Send the transaction
    eth_client.sign_and_send(tx).await?;
    println!("Authorization created successfully");

    let coprocessor_input = json!({});
    let zkp = coprocessor_client
            .prove(COPROCESSOR_APP_ID, &coprocessor_input)
            .await?;

    info!("co_processor zkp post response: {:?}", zkp);

    // extract the program and domain parameters by decoding the zkp
    let (proof_program, inputs_program) = zkp.program.decode()?;
    let (proof_domain, inputs_domain) = zkp.domain.decode()?;

    // build the eureka transfer zk message from decoded params
    let auth_eureka_transfer_zk_msg = authorization.executeZKMessage(
        Bytes::from(inputs_program),
        Bytes::from(proof_program),
        Bytes::from(inputs_domain),
        Bytes::from(proof_domain),
    );

    // sign and execute the tx & await its tx receipt before proceeding
    info!("posting zkp ethereum authorizations");
    let zk_auth_exec_response = eth_client
        .sign_and_send(auth_eureka_transfer_zk_msg.into_transaction_request())
        .await?;

    rp.get_transaction_receipt(zk_auth_exec_response.transaction_hash).await?;

    Ok(())
}
