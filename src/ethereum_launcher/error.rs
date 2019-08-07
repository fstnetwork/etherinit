#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid running mode: {}", _0)]
    InvalidRunningMode(String),

    #[fail(display = "Io error: {}", _0)]
    StdIo(std::io::Error),

    #[fail(display = "JSON error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[fail(display = "EthSign error: {}", _0)]
    EthSign(ethsign::Error),

    #[fail(display = "Primitives error: {}", _0)]
    Primitives(crate::primitives::Error),

    #[fail(display = "Failed to import key file")]
    FailedToImportKeyFile,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::StdIo(error)
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

impl From<crate::primitives::Error> for Error {
    fn from(error: crate::primitives::Error) -> Error {
        Error::Primitives(error)
    }
}
