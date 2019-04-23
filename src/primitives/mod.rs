use ethsign::SecretKey;
use std::str::FromStr;

mod consensus_engine;
mod enode_url;
mod error;
mod ethereum_chainspec;
mod node_info;
mod node_role;

pub use self::consensus_engine::ConsensusEngine;
pub use self::enode_url::{Error as EthereumNodeUrlError, EthereumNodeUrl};
pub use self::error::Error;
pub use self::ethereum_chainspec::EthereumChainSpec;
pub use self::node_info::NodeInfo;
pub use self::node_role::NodeRole;

pub const DEFAULT_PARITY_GAS_CAP: &str = "10000000";
pub const DEFAULT_PARITY_GAS_FLOOR_TARGET: &str = "8000000";

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
            _ => Err(Error::InvalidEthereumProgramName(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumSystemInfo {
    pub consensus_engine: ConsensusEngine,
    pub miner_count: usize,
    pub node_count: usize,
}

use hdwallet::{
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
        Err(_err) => Err(Error::FailedToGeneratePrivateKey {
            seed,
            path: format!("{:?}", path),
        }),
    }
}
