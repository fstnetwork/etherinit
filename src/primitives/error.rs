use crate::utils::env_var::Error as EnvVarError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Io error: {}", _0)]
    StdIo(std::io::Error),

    #[fail(display = "Parse integration error: {}", _0)]
    NumParseInt(std::num::ParseIntError),

    #[fail(display = "JSON error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[fail(display = "EthSign error: {}", _0)]
    EthSign(ethsign::Error),

    #[fail(display = "Invalid environment error = {:?}", _0)]
    EnvVar(EnvVarError),

    #[fail(display = "Invalid Ethereum program name: {}", _0)]
    InvalidEthereumProgramName(String),

    #[fail(display = "Invalid mnemonic phrase {}", _0)]
    InvalidMnemonicPhrase(String),

    #[fail(display = "Invalid consensus engine type: {}", _0)]
    InvalidConsensusEngineType(String),

    #[fail(display = "Invalid gas limit value: {}", _0)]
    InvalidGasLimitValue(String),

    #[fail(display = "Invalid minimum gas limit value: {}", _0)]
    InvalidMinimumGasLimitValue(String),

    #[fail(display = "Invalid account state data: {}", _0)]
    InvalidAccountStateData(String),

    #[fail(
        display = "Failed to generate private key from seed {:?} and path {}",
        seed, path
    )]
    FailedToGeneratePrivateKey { seed: Vec<u8>, path: String },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::StdIo(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Error {
        Error::NumParseInt(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::SerdeJson(error)
    }
}

impl From<ethsign::Error> for Error {
    fn from(error: ethsign::Error) -> Error {
        Error::EthSign(error)
    }
}

impl From<EnvVarError> for Error {
    fn from(error: EnvVarError) -> Error {
        Error::EnvVar(error)
    }
}
