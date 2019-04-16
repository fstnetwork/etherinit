use crate::primitives::EthereumNodeUrlError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "EthereumNodeUrl error: {}", _0)]
    EthereumNodeUrl(EthereumNodeUrlError),

    #[fail(display = "address parse error: {}", _0)]
    AddrParse(std::net::AddrParseError),

    #[fail(display = "hyper error: {}", _0)]
    Hyper(hyper::Error),

    #[fail(display = "JSON error: {}", _0)]
    Json(serde_json::Error),

    #[fail(display = "Tokio Timer error: {}", _0)]
    Timer(tokio_timer::Error),

    // FIXME use Web3(web3::Error),
    #[fail(display = "Web3 error: {}", _0)]
    Web3(String),
}

impl From<std::net::AddrParseError> for Error {
    fn from(error: std::net::AddrParseError) -> Error {
        Error::AddrParse(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::Hyper(error)
    }
}

impl From<tokio_timer::Error> for Error {
    fn from(error: tokio_timer::Error) -> Error {
        Error::Timer(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Json(error)
    }
}

impl From<web3::Error> for Error {
    fn from(error: web3::Error) -> Error {
        Error::Web3(error.to_string())
    }
}

impl From<EthereumNodeUrlError> for Error {
    fn from(error: EthereumNodeUrlError) -> Error {
        Error::EthereumNodeUrl(error)
    }
}
