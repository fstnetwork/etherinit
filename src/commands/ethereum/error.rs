use crate::ethereum_controller::Error as EthereumControllerError;
use crate::network_keeper::Error as NetworkKeeperError;
use crate::utils::env_var::Error as EnvVarError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Io error: {}", _0)]
    StdIo(std::io::Error),

    #[fail(display = "Parse integrate error: {}", _0)]
    NumParseInt(std::num::ParseIntError),

    #[fail(display = "Tokio timer error: {}", _0)]
    Timer(tokio::timer::Error),

    #[fail(display = "Environment variable error: {}", _0)]
    EnvVar(#[fail(cause)] EnvVarError),

    #[fail(display = "Network Keeper error: {}", _0)]
    NetworkKeeper(NetworkKeeperError),

    #[fail(display = "Ethereum Controller error: {}", _0)]
    EthereumController(EthereumControllerError),

    #[fail(display = "Unknown node role: {}", _0)]
    UnknownNodeRole(String),

    #[fail(display = "Invalid mnemonic phrase {}", _0)]
    InvalidMnemonicPhrase(String),

    #[fail(display = "Failed to extract miner index from HOSTNAME={}", _0)]
    FailedToExtractMinerIndexFromHostname(String),

    // #[fail(
    //     display = "Too larget miner index: {}, miner count: {}",
    //     miner_index, miner_count
    // )]
    // TooLargeMinerIndex {
    //     miner_index: usize,
    //     miner_count: usize,
    // },

    // #[fail(display = "Invalid sealer master seed: {}", _0)]
    // InvalidSealerMasterSeed(String),

    // #[fail(display = "Invalid miner count: {}", _0)]
    // InvalidMinerCount(String),

    // #[fail(display = "Invalid miner index: {}", _0)]
    // InvalidMinerIndex(String),

    // #[fail(display = "Invalid consensus engine type: {}", _0)]
    // InvalidConsensusEngineType(String),

    // #[fail(display = "Invalid gas limit value: {}", _0)]
    // InvalidGasLimitValue(String),

    // #[fail(display = "Invalid private key: {}", _0)]
    // InvalidPrivateKey(String),

    // #[fail(display = "Invalid HD path: {}", _0)]
    // InvalidHDPath(String),

    // #[fail(
    //     display = "Failed to generate private key from seed {:?} and path {}",
    //     seed, path
    // )]
    // FailedToGeneratePrivateKey { seed: Vec<u8>, path: String },
    #[fail(display = "Failed to fetch system info")]
    FailedToFetchSystemInfo,

    #[fail(display = "Failed to fetch peers")]
    FailedToFetchPeers,

    #[fail(display = "Failed to fetch chain spec")]
    FailedToFetchChainSpec,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::StdIo(error)
    }
}

impl From<EnvVarError> for Error {
    fn from(error: EnvVarError) -> Error {
        Error::EnvVar(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Error {
        Error::NumParseInt(error)
    }
}

impl From<tokio::timer::Error> for Error {
    fn from(error: tokio::timer::Error) -> Error {
        Error::Timer(error)
    }
}

impl From<NetworkKeeperError> for Error {
    fn from(error: NetworkKeeperError) -> Error {
        Error::NetworkKeeper(error)
    }
}

impl From<EthereumControllerError> for Error {
    fn from(error: EthereumControllerError) -> Error {
        Error::EthereumController(error)
    }
}
