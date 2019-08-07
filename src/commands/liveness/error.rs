use crate::utils::env_var;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{:?}", _0)]
    EnvVar(env_var::Error),

    #[fail(display = "Parse integrate error: {}", _0)]
    NumParseInt(std::num::ParseIntError),

    #[fail(display = "Timeout")]
    Timeout,

    #[fail(display = "Timer error: {}", _0)]
    Timer(tokio::timer::Error),

    #[fail(display = "Web3 error: {}", _0)]
    Web3(String),
}

impl From<env_var::Error> for Error {
    fn from(err: env_var::Error) -> Error {
        Error::EnvVar(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Error {
        Error::NumParseInt(error)
    }
}

impl From<web3::Error> for Error {
    fn from(error: web3::Error) -> Error {
        Error::Web3(error.to_string())
    }
}
