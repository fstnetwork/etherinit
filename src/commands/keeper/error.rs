use crate::network_keeper::Error as NetworkKeeperError;
use crate::primitives::Error as PrimitivesError;
use crate::utils::env_var::Error as EnvVarError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Io error: {}", _0)]
    StdIo(std::io::Error),

    #[fail(display = "Parse integration error: {}", _0)]
    StdNum(std::num::ParseIntError),

    #[fail(display = "Environment variable error: {}", _0)]
    EnvVar(EnvVarError),

    #[fail(display = "Crate primitives error: {}", _0)]
    Primitives(PrimitivesError),

    #[fail(display = "NetworkKeeper error: {}", _0)]
    NetworkKeeper(NetworkKeeperError),

    #[fail(display = "Tokio timer error: {}", _0)]
    Timer(tokio::timer::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::StdIo(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Error {
        Error::StdNum(error)
    }
}

impl From<tokio::timer::Error> for Error {
    fn from(error: tokio::timer::Error) -> Error {
        Error::Timer(error)
    }
}

impl From<EnvVarError> for Error {
    fn from(error: crate::utils::env_var::Error) -> Error {
        Error::EnvVar(error)
    }
}

impl From<PrimitivesError> for Error {
    fn from(error: PrimitivesError) -> Error {
        Error::Primitives(error)
    }
}

impl From<NetworkKeeperError> for Error {
    fn from(error: NetworkKeeperError) -> Error {
        Error::NetworkKeeper(error)
    }
}
