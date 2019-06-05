use crate::bootnode::BootnodeClientError;
use crate::primitives::EthereumNodeUrlError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Web3 error: {}", _0)]
    // FIXME use Web3(web3::Error),
    Web3(String),

    #[fail(display = "BootnodeClient error: {}", _0)]
    BootnodeClient(BootnodeClientError),

    #[fail(display = "EthereumNodeUrl error: {}", _0)]
    EthereumNodeUrl(EthereumNodeUrlError),

    #[fail(display = "Unable to register Ethereum node info")]
    UnableToRegisterEthereumNodeInfo,
}

impl From<web3::Error> for Error {
    fn from(error: web3::Error) -> Error {
        Error::Web3(error.to_string())
    }
}

impl From<BootnodeClientError> for Error {
    fn from(error: BootnodeClientError) -> Error {
        Error::BootnodeClient(error)
    }
}

impl From<EthereumNodeUrlError> for Error {
    fn from(error: EthereumNodeUrlError) -> Error {
        Error::EthereumNodeUrl(error)
    }
}
