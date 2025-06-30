extern crate alloc;

use std::error::Error;
use alloy::primitives::{Address, Bytes, FixedBytes};
use alloy_sol_types::{sol, SolCall, SolValue};
use valence_domain_clients::{
    clients::ethereum::EthereumClient,
    evm::{base_client::EvmBaseClient, request_provider_client::RequestProviderClient},
};
use simple_program_demo::types::sol_types::{Authorization};
use simple_program_demo::{FORWARDER, AUTHORIZATION};


sol! {
    /// Duration type for Valence messages
    enum DurationType {
        Height,
        Time
    }

    /// Duration structure
    struct Duration {
        DurationType durationType;
        uint64 value;
    }

    /// Retry times type
    enum RetryTimesType {
        NoRetry,
        Indefinitely,
        Amount
    }

    /// Retry times structure
    struct RetryTimes {
        RetryTimesType retryType;
        uint64 amount;
    }

    /// Retry logic structure
    struct RetryLogic {
        RetryTimes times;
        Duration interval;
    }

    /// Atomic function structure
    struct AtomicFunction {
        address contractAddress;
    }

    /// Atomic subroutine structure
    struct AtomicSubroutine {
        AtomicFunction[] functions;
        RetryLogic retryLogic;
    }

    /// Subroutine type
    enum SubroutineType {
        Atomic,
        NonAtomic
    }

    /// Subroutine structure
    struct Subroutine {
        SubroutineType subroutineType;
        bytes subroutine;
    }

    /// Priority enum
    enum Priority {
        Medium,
        High
    }

    /// SendMsgs structure
    struct SendMsgs {
        uint64 executionId;
        Priority priority;
        Subroutine subroutine;
        uint64 expirationTime;
        bytes[] messages;
    }

    /// ProcessorMessage type enum
    enum ProcessorMessageType {
        Pause,
        Resume,
        EvictMsgs,
        SendMsgs,
        InsertMsgs
    }

    /// ProcessorMessage structure
    struct ProcessorMessage {
        ProcessorMessageType messageType;
        bytes message;
    }

    /// ZkMessage structure for Valence Authorization
    struct ZkMessage {
        uint64 registry;
        uint64 blockNumber;
        address authorizationContract;
        ProcessorMessage processorMessage;
    }

    function forward() external;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mnemonic = "test test test test test test test test test test test junk";
    let rpc_url = "http://127.0.0.1:8545";

    let eth_client = EthereumClient::new(rpc_url, &mnemonic, None)?;
    let rp = eth_client.get_request_provider().await?;

    let authorization = Authorization::new(AUTHORIZATION, &rp);

    let forward_call = forwardCall {};

    // ABI encode the transfer call
    let encoded_transfer_call = forward_call.abi_encode();

    let atomic_function = AtomicFunction {
        contractAddress: FORWARDER.to_string().parse().unwrap(),
    };

    // Create retry logic with NoRetry for atomic execution
    let retry_logic = RetryLogic {
        times: RetryTimes {
            retryType: RetryTimesType::NoRetry,
            amount: 0,
        },
        interval: Duration {
            durationType: DurationType::Time,
            value: 0,
        },
    };

    // Create AtomicSubroutine
    let atomic_subroutine = AtomicSubroutine {
        functions: alloc::vec![atomic_function],
        retryLogic: retry_logic,
    };

    // Encode the atomic subroutine
    let encoded_subroutine = atomic_subroutine.abi_encode();

    // Create Subroutine wrapper
    let subroutine = Subroutine {
        subroutineType: SubroutineType::Atomic,
        subroutine: alloy_sol_types::private::Bytes::from(encoded_subroutine),
    };

    // Create SendMsgs message with the properly encoded transfer call
    let send_msgs = SendMsgs {
        executionId: 1, // Generated execution ID
        priority: Priority::Medium,
        subroutine,
        expirationTime: 0, // No expiration
        messages: alloc::vec![alloy_sol_types::private::Bytes::from(encoded_transfer_call)],
    };

    // Encode SendMsgs
    let encoded_send_msgs = send_msgs.abi_encode();

    // Create ProcessorMessage
    let processor_message = ProcessorMessage {
        messageType: ProcessorMessageType::SendMsgs,
        message: alloy_sol_types::private::Bytes::from(encoded_send_msgs),
    };


    let tx = authorization
        .addStandardAuthorizations(
            vec!["forward".to_string()],
            vec![vec![Address::ZERO]],
            vec![vec![Authorization::AuthorizationData {
                contractAddress: FORWARDER.to_string().parse().unwrap(),
                useFunctionSelector: true,
                functionSelector: FixedBytes::<4>::new(forwardCall::SELECTOR),
                callHash: FixedBytes::<32>::default(),
            }]],
        )
        .into_transaction_request();
    eth_client.sign_and_send(tx).await?;

    
    let tx = authorization
        .sendProcessorMessage(
            "forward".to_string(),
            Bytes::from(processor_message.abi_encode()),
        )
        .into_transaction_request();
    eth_client.sign_and_send(tx).await?;

    Ok(())
}
