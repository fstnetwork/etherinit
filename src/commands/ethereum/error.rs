error_chain! {
foreign_links {
    StdIo(std::io::Error);
    EnvVar(super::utils::env_var::Error);
    NumParseInt(std::num::ParseIntError);
    AddrParse(std::net::AddrParseError);
    JsonParse(serde_json::Error);
    Ethereum(super::ethereum_controller::Error);
    NetworkKeeper(super::network_keeper::Error);
    Timer(tokio_timer::Error);
}

errors {
    UnknownNodeRole(t: String) {
        description("Unknown node role")
        display("Unknown node role: {}", t)
    }
    TooLargeMinerIndex(miner_index: usize, miner_count: usize) {
        description("Too large miner index")
        display("Too larget miner index: {}, miner count: {}", miner_index, miner_count)
    }
    InvalidSealerMasterSeed(s: String) {
        description("Invalid sealer master seed")
        display("Invalid sealer master seed: {}", s)
    }
    InvalidMinerCount(s: String) {
        description("Invalid miner count")
        display("Invalid miner count: {}", s)
    }
    InvalidMinerIndex(s: String) {
        description("Invalid miner index")
        display("Invalid miner index: {}", s)
    }
    InvalidConsensusEngineType(s: String) {
        description("Invalid consensus engine type")
        display("Invalid consensus engine type: {}", s)
    }
    InvalidGasLimitValue(s: String) {
        description("Invalid gas limit value")
        display("Invalid gas limit value: {}", s)
    }
    InvalidPrivateKey(s: String) {
        description("Invalid private key")
        display("Invalid private key: {}", s)
    }
    InvalidHDPath(s: String) {
        description("Invalid HD path")
        display("Invalid HD path: {}", s)
    }
    InvalidMnemonicPhrase(phrase: String) {
        description("Invalid mnemonic phrase")
        display("Invalid mnemonic phrase {}", phrase)
    }
    FailedToGeneratePrivateKey(seed: Vec<u8>, path: String) {
        description("Failed to generate private key")
        display("Failed to generate private key from seed {:?} and path {}", seed, path)
    }
    FailedToFetchSystemInfo {
        description("Failed to fetch system info")
        display("Failed to fetch system info")
    }
    FailedToFetchPeers {
        description("Failed to fetch peers")
        display("Failed to fetch peers")
    }
    FailedToFetchChainSpec {
        description("Failed to fetch chain spec")
        display("Failed to fetch chain spec")
    }
}
}
