use std::{error::Error};

use alloy::{
    hex::FromHex,
    primitives::{Address, Bytes, FixedBytes, Uint},
    sol_types::SolValue,
    sol
};
use valence_domain_clients::{
    clients::{coprocessor::CoprocessorClient, ethereum::EthereumClient},
    coprocessor::base_client::CoprocessorBaseClient,
    evm::{base_client::EvmBaseClient, request_provider_client::RequestProviderClient},
};
use sp1_sdk::{HashableKey, SP1VerifyingKey};
use informal_program_demo::{SP1_VERIFIER};
use informal_program_demo::types::sol_types::{
    Authorization, BaseAccount,
    SP1VerificationGateway,
    processor_contract::LiteProcessor,
    MockERC20, ERC1967Proxy,
    Forwarder
};

sol! {
    enum IntervalType {
        TIME,
        BLOCKS
    }

    struct ForwardingConfig {
        address tokenAddress;
        uint256 maxAmount;
    }

    struct ForwarderConfig {
        address inputAccount;
        address outputAccount;
        ForwardingConfig[] forwardingConfigs;
        IntervalType intervalType;
        uint64 minInterval;
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();

    let mnemonic = "test test test test test test test test test test test junk";
    let rpc_url = "http://127.0.0.1:8545";

    let eth_client = EthereumClient::new(rpc_url, &mnemonic, None)?;
    let my_address = eth_client.signer().address();
    let rp = eth_client.get_request_provider().await?;

    let send_account_tx =
        BaseAccount::deploy_builder(&rp, my_address, vec![]).into_transaction_request();


    let send_account = eth_client
        .sign_and_send(send_account_tx)
        .await?
        .contract_address
        .unwrap();
    println!("Send account deployed at: {}", send_account);

    let deposit_account_tx =
        BaseAccount::deploy_builder(&rp, my_address, vec![]).into_transaction_request();

    let deposit_account = eth_client
        .sign_and_send(deposit_account_tx)
        .await?
        .contract_address
        .unwrap();
    println!("Deposit account deployed at: {}", deposit_account);
    
    let processor =
        LiteProcessor::deploy_builder(&rp, FixedBytes::<32>::default(), Address::ZERO, 0, vec![])
            .into_transaction_request();

    let processor_address = eth_client
        .sign_and_send(processor)
        .await?
        .contract_address
        .unwrap();
    println!("Processor deployed at: {processor_address}");

    let token_tx = MockERC20::deploy_builder(
        &rp,
        "Demo Token".to_string(),
        "DEMO".to_string(),
        18
    );
    let token = eth_client
        .sign_and_send(token_tx.into_transaction_request())
        .await?
        .contract_address
        .unwrap();

    let forwarding_config = ForwardingConfig {
        tokenAddress: token,
        maxAmount: Uint::from(100),
    };

    let forwarder_config = ForwarderConfig {
        inputAccount: send_account,
        outputAccount: deposit_account,
        forwardingConfigs: vec![forwarding_config],
        intervalType: IntervalType::BLOCKS,
        minInterval: 1
    };

    let forwarder = Forwarder::deploy_builder(
        &rp,
        my_address,
        processor_address,
        forwarder_config.abi_encode().into()
    );

    let forwarder = eth_client
        .sign_and_send(forwarder.into_transaction_request())
        .await?
        .contract_address
        .unwrap();
    println!("Forwarder library deployed at {}", forwarder);
        

    let send_account = BaseAccount::new(send_account, &rp);
    let approve_library_tx = send_account
        .approveLibrary(forwarder)
        .into_transaction_request();
    eth_client.sign_and_send(approve_library_tx).await?;
    println!("Forwarder library approved from send account");

    let verification_gateway =
        SP1VerificationGateway::deploy_builder(&rp).into_transaction_request();
    let verification_gateway_implementation = eth_client
        .sign_and_send(verification_gateway)
        .await?
        .contract_address
        .unwrap();

    let proxy_tx =
        ERC1967Proxy::deploy_builder(&rp, verification_gateway_implementation, Bytes::new())
            .into_transaction_request();
    let verification_gateway_address = eth_client
        .sign_and_send(proxy_tx)
        .await?
        .contract_address
        .unwrap();
    println!("Verification Gateway deployed at: {verification_gateway_address}");

    // Initialize the verification gateway
    // We need to get the domain vk of the coprocessor
    let coprocessor_client = CoprocessorClient::default();
    let domain_vk = coprocessor_client.get_domain_vk().await?;
    let sp1_domain_vk: SP1VerifyingKey = bincode::deserialize(&domain_vk)?;
    let domain_vk = FixedBytes::<32>::from_hex(sp1_domain_vk.bytes32()).unwrap();

    let verification_gateway = SP1VerificationGateway::new(verification_gateway_address, &rp);
    let initialize_verification_gateway_tx = verification_gateway
        .initialize(
            "0x0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap(),
            SP1_VERIFIER.parse().unwrap(),
            domain_vk
        )
        .into_transaction_request();
    eth_client
        .sign_and_send(initialize_verification_gateway_tx)
        .await?;
    println!("Verification Gateway initialized");

    // Transfer the ownership of the verification gateway
    let transfer_ownership_tx = verification_gateway
        .transferOwnership(my_address)
        .into_transaction_request();
    eth_client.sign_and_send(transfer_ownership_tx).await?;
    println!(
        "Verification Gateway ownership transferred to: {}",
        my_address
    );

    let authorization = Authorization::deploy_builder(
        &rp,
        my_address, // We will be initial owners to eventually add the authorizations, then we need to transfer ownership
        processor_address,
        verification_gateway_address,
        true, // Store callbacks
    );

    let authorization_address = eth_client
        .sign_and_send(authorization.into_transaction_request())
        .await?
        .contract_address
        .unwrap();
    println!("Authorization deployed at: {authorization_address}");

    // Add authorization contract as an authorized address to the proccessor
    let processor = LiteProcessor::new(processor_address, &rp);

    let add_authorization_tx = processor
        .addAuthorizedAddress(authorization_address)
        .into_transaction_request();

    eth_client.sign_and_send(add_authorization_tx).await?;
    println!("Authorization added to processor");

    // Transfer ownership of the send account to the owner
    let transfer_ownership_tx = send_account
        .transferOwnership(my_address)
        .into_transaction_request();
    eth_client.sign_and_send(transfer_ownership_tx).await?;

    // Query to verify the ownership was transferred
    let new_owner = send_account.owner().call().await?._0;
    println!("Deposit account ownership transferred to: {new_owner}");
    assert_eq!(new_owner, my_address);

    Ok(())

}
