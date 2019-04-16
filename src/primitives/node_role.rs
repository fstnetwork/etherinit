use ethsign::SecretKey;

use hdwallet::mnemonic::Mnemonic;

#[derive(Clone, Serialize, Deserialize)]
pub enum NodeRole {
    Miner {
        index: usize,
        sealer_mnemonic: Mnemonic,
    },
    Transactor,
    Syncer,
}

impl std::fmt::Debug for NodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeRole::Miner { index, .. } => write!(f, "Miner(index: {})", index),
            NodeRole::Transactor => write!(f, "Transactor"),
            NodeRole::Syncer => write!(f, "Syncer"),
        }
    }
}

impl NodeRole {
    pub fn identity(&self) -> String {
        match self {
            NodeRole::Miner { index, .. } => format!("miner-{:02}", index),
            NodeRole::Transactor => "transactor".to_owned(),
            NodeRole::Syncer => "syncer".to_owned(),
        }
    }

    pub fn is_miner(&self) -> bool {
        match self {
            NodeRole::Miner { .. } => true,
            _ => false,
        }
    }

    pub fn is_transactor(&self) -> bool {
        match self {
            NodeRole::Transactor => true,
            _ => false,
        }
    }

    pub fn is_syncer(&self) -> bool {
        match self {
            NodeRole::Syncer => true,
            _ => false,
        }
    }

    pub fn validator_keypair(&self) -> Option<SecretKey> {
        match self {
            NodeRole::Transactor | NodeRole::Syncer => None,
            NodeRole::Miner {
                index,
                sealer_mnemonic,
            } => match super::generate_keypair_with_index(sealer_mnemonic, *index) {
                Ok(kp) => Some(kp),
                _ => None,
            },
        }
    }
}
