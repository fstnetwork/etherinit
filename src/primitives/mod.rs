use ethsign::SecretKey;
use std::str::FromStr;

use super::hdwallet;

mod consensus_engine;
mod enode_url;
mod error;
mod ethereum_chainspec;
mod node_info;
mod node_role;

pub use self::consensus_engine::ConsensusEngine;
pub use self::enode_url::{Error as EthereumNodeUrlError, EthereumNodeUrl};
pub use self::error::{Error, ErrorKind};
pub use self::ethereum_chainspec::EthereumChainSpec;
pub use self::node_info::NodeInfo;
pub use self::node_role::NodeRole;
pub use super::utils;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EthereumProgram {
    Parity,
    GoEthereum,
}

impl FromStr for EthereumProgram {
    type Err = Error;
    fn from_str(s: &str) -> Result<EthereumProgram, Self::Err> {
        match s.to_lowercase().as_str() {
            "parity" | "parity-ethereum" | "parityethereum" => Ok(EthereumProgram::Parity),
            "geth" | "go-ethereum" | "goethereum" => Ok(EthereumProgram::GoEthereum),
            _ => Err(Error::from(ErrorKind::InvalidEthereumProgramName(
                s.to_owned(),
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumSystemInfo {
    #[serde(rename = "consensusEngine")]
    pub consensus_engine: ConsensusEngine,

    #[serde(rename = "minerCount")]
    pub miner_count: usize,

    #[serde(rename = "nodeCount")]
    pub node_count: usize,
}

use super::hdwallet::{
    hdpath::{self, ChildNumber, HDPath},
    mnemonic::Mnemonic,
};

fn default_hdpath_with_index(index: u32) -> HDPath {
    HDPath(vec![
        ChildNumber::Hardened(44),
        ChildNumber::Hardened(60),
        ChildNumber::Hardened(0),
        ChildNumber::Normal(0),
        ChildNumber::Normal(index as u32),
    ])
}

fn generate_keypair_with_index(
    mnemonic: &Mnemonic,
    sealer_index: usize,
) -> Result<SecretKey, Error> {
    let seed = mnemonic.seed("");
    let path = default_hdpath_with_index(sealer_index as u32);
    match hdpath::generate_keypair(&path, &seed) {
        Ok(keypair) => Ok(keypair),
        Err(_err) => Err(Error::from(ErrorKind::FailedToGeneratePrivateKey(
            seed,
            format!("{:?}", path),
        ))),
    }
}
