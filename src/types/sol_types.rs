use alloy::sol;

sol!(
    #[sol(rpc)]
    BaseAccount,
    "src/contracts/BaseAccount.sol/BaseAccount.json",
);

// Need to use a module to avoid name conflicts with Authorization
pub mod processor_contract {
    alloy::sol!(
        #[sol(rpc)]
        LiteProcessor,
        "src/contracts/LiteProcessor.sol/LiteProcessor.json",
    );
}

sol!(
    #[sol(rpc)]
    Authorization,
    "src/contracts/Authorization.sol/Authorization.json",
);

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    Forwarder,
    "src/contracts/Forwarder.sol/Forwarder.json",
);

sol!(
    #[sol(rpc)]
    ERC1967Proxy,
    "src/contracts/ERC1967Proxy.sol/ERC1967Proxy.json",
);

sol!(
    #[sol(rpc)]
    MockERC20,
    "src/contracts/MockERC20.sol/MockERC20.json",
);

sol!(
    #[sol(rpc)]
    SP1VerificationGateway,
    "src/contracts/SP1VerificationGateway.sol/SP1VerificationGateway.json",
);
