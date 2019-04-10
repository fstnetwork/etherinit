use ethereum_types::{Address, U256};

use super::EthereumProgram;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusEngine {
    Ethash {
        genesis_difficulty: U256,
    },
    ParityAura {
        block_period: u64,
        block_reward: U256,
        validators: Vec<Address>,
    },
    ParityTendermint {
        propose_timeout: u64,
        prevote_timeout: u64,
        precommit_timeout: u64,
        commit_timeout: u64,
        block_reward: U256,
        validators: Vec<Address>,
    },
    GethClique {
        block_period: u64,
        block_reward: U256,
        validators: Vec<Address>,
    },
}

impl ConsensusEngine {
    pub fn program(&self) -> EthereumProgram {
        match self {
            ConsensusEngine::Ethash { .. } => EthereumProgram::Parity,
            ConsensusEngine::ParityAura { .. } => EthereumProgram::Parity,
            ConsensusEngine::ParityTendermint { .. } => EthereumProgram::Parity,
            ConsensusEngine::GethClique { .. } => EthereumProgram::GoEthereum,
        }
    }

    pub fn validators(&self) -> Option<Vec<Address>> {
        match self {
            ConsensusEngine::Ethash { .. } => None,
            ConsensusEngine::ParityAura { validators, .. } => Some(validators.clone()),
            ConsensusEngine::ParityTendermint { validators, .. } => Some(validators.clone()),
            ConsensusEngine::GethClique { validators, .. } => Some(validators.clone()),
        }
    }

    pub fn validator_count(&self) -> usize {
        match self {
            ConsensusEngine::Ethash { .. } => 0,
            ConsensusEngine::ParityAura { validators, .. } => validators.len(),
            ConsensusEngine::ParityTendermint { validators, .. } => validators.len(),
            ConsensusEngine::GethClique { validators, .. } => validators.len(),
        }
    }
}
