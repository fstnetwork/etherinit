use ethereum_types::{Address, U256};
use ethsign::SecretKey;
use hdwallet::mnemonic::{Language, Mnemonic};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use crate::utils::{self, env_var::from_env};

use super::error::Error;
use super::{generate_keypair_with_index, AccountState, ConsensusEngine};

lazy_static! {
    static ref PARITY_DEFAULT_SEAL: serde_json::Value = json!({
        "authorityRound": {
            "step": "0x0",
            "signature": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}});
}

#[derive(Debug, Clone)]
pub struct EthereumChainSpec {
    /// name of Ethereum Blockchain
    pub name: String,

    /// network ID of Ethereum Blockchain
    pub network_id: U256,

    /// Minimum gas limit of a block
    pub min_gas_limit: U256,

    /// gas limit of genesis block
    pub genesis_block_gas_limit: U256,

    /// consensus engine type and its parameters
    pub consensus_engine: ConsensusEngine,

    pub account_states: HashMap<Address, AccountState>,
}

impl Default for EthereumChainSpec {
    fn default() -> EthereumChainSpec {
        EthereumChainSpec {
            name: "ethereum".to_owned(),
            network_id: U256::from(0x1234),
            min_gas_limit: U256::from(0x1388),
            genesis_block_gas_limit: U256::from(5) * U256::from(10).pow(U256::from(18)),
            consensus_engine: ConsensusEngine::ParityAura {
                block_period: 5,
                block_reward: U256::from(5) * U256::from(10).pow(U256::from(18)),
                validators: vec![],
            },
            account_states: Default::default(),
        }
    }
}

impl EthereumChainSpec {
    fn validators_from_env() -> Result<Vec<Address>, Error> {
        let seed = from_env("SEALER_MNEMONIC_PHRASE")?;
        let miner_count: usize = from_env("MINER_COUNT")?.parse()?;
        let keypairs = keypair_from_sealer_mnemonic(&seed, miner_count)?;
        let validators = keypairs
            .iter()
            .map(|sec| Address::from(*sec.public().address()))
            .collect();
        Ok(validators)
    }

    fn account_states_from_env() -> Result<HashMap<Address, AccountState>, Error> {
        let account_states_file_path = match from_env("ACCOUNT_STATES_FILE") {
            Ok(file_path) => PathBuf::from(file_path),
            Err(_) => return Ok(HashMap::default()),
        };

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(account_states_file_path)?;

        let raw_value: serde_json::Value = serde_json::from_reader(&file)?;
        match raw_value {
            serde_json::Value::Object(states) => {
                Ok(states
                    .into_iter()
                    .fold(HashMap::default(), |mut states, (address, state)| {
                        let address = match Address::from_str(utils::clean_0x(&address)) {
                            Ok(addr) => addr,
                            Err(_) => {
                                println!("invalid ethereum address: {}", address);
                                warn!("invalid ethereum address: {}", address);
                                return states;
                            }
                        };

                        states.insert(address, AccountState::from_json_value(&state));
                        states
                    }))
            }
            _ => Err(Error::InvalidAccountStateData(raw_value.to_string())),
        }
    }

    pub fn from_env() -> Result<EthereumChainSpec, Error> {
        let name = from_env("NETWORK_NAME")?;

        let genesis_block_gas_limit = {
            let raw_value = from_env("GENESIS_BLOCK_GAS_LIMIT")?;
            match U256::from_str(utils::clean_0x(raw_value.as_str())) {
                Ok(v) => v,
                Err(_) => return Err(Error::InvalidGasLimitValue(raw_value)),
            }
        };

        let min_gas_limit = {
            let raw_value = from_env("MIN_GAS_LIMIT").unwrap_or_else(|_| "0x1388".to_string());
            match U256::from_str(utils::clean_0x(raw_value.as_str())) {
                Ok(v) => v,
                Err(_) => return Err(Error::InvalidMinimumGasLimitValue(raw_value)),
            }
        };

        let mut account_states = Self::account_states_from_env()?;

        let consensus_engine = {
            use serde_json::Value as JsonValue;
            let sealer_intrinsic_balance = U256::from_dec_str(
                from_env("SEALER_INTRINSIC_BALANCE")
                    .unwrap_or_else(|_| "0".into())
                    .as_str(),
            )
            .unwrap_or_default();

            let engine = from_env("CONSENSUS_ENGINE")?;
            match engine.to_lowercase().as_ref() {
                "ethash" => {
                    let engine_parameters: JsonValue =
                        serde_json::from_str(from_env("ETHASH_CONSENSUS_PARAMETERS")?.as_str())?;
                    let genesis_difficulty: U256 = engine_parameters["genesisBlockDifficulty"]
                        .as_u64()
                        .unwrap_or(16384)
                        .into();
                    ConsensusEngine::Ethash { genesis_difficulty }
                }
                "aura" => {
                    let engine_parameters: JsonValue =
                        serde_json::from_str(from_env("AURA_CONSENSUS_PARAMETERS")?.as_str())?;
                    let block_period = engine_parameters["blockPeriod"].as_u64().unwrap_or(7);

                    let block_reward = U256::from_dec_str(
                        engine_parameters["blockReward"]
                            .as_str()
                            .unwrap_or_else(|| "5000000000000000000"),
                    )
                    .unwrap_or(U256::from(5) * U256::from(10).pow(18.into()));

                    let validators = Self::validators_from_env()?;
                    for validator_address in validators.iter() {
                        account_states.insert(
                            validator_address.clone(),
                            AccountState {
                                balance: Some(sealer_intrinsic_balance),
                                ..Default::default()
                            },
                        );
                    }

                    ConsensusEngine::ParityAura {
                        block_period,
                        block_reward,
                        validators,
                    }
                }
                "tendermint" => {
                    let engine_parameters: JsonValue = serde_json::from_str(
                        from_env("TENDERMINT_CONSENSUS_PARAMETERS")?.as_str(),
                    )?;

                    let propose_timeout = engine_parameters["proposeTimeout"]
                        .as_u64()
                        .unwrap_or(10000);
                    let prevote_timeout = engine_parameters["prevoteTimeout"]
                        .as_u64()
                        .unwrap_or(10000);
                    let precommit_timeout = engine_parameters["precommitTimeout"]
                        .as_u64()
                        .unwrap_or(10000);
                    let commit_timeout =
                        engine_parameters["commitTimeout"].as_u64().unwrap_or(10000);

                    let block_reward = U256::from_dec_str(
                        engine_parameters["blockReward"]
                            .as_str()
                            .unwrap_or_else(|| "5000000000000000000"),
                    )
                    .unwrap_or(U256::from(5) * U256::from(10).pow(18.into()));

                    let validators = Self::validators_from_env()?;
                    for validator_address in validators.iter() {
                        account_states.insert(
                            validator_address.clone(),
                            AccountState {
                                balance: Some(sealer_intrinsic_balance),
                                ..Default::default()
                            },
                        );
                    }

                    ConsensusEngine::ParityTendermint {
                        propose_timeout,
                        prevote_timeout,
                        precommit_timeout,
                        commit_timeout,
                        block_reward,
                        validators,
                    }
                }
                "clique" => {
                    let engine_parameters: JsonValue =
                        serde_json::from_str(from_env("CLIQUE_CONSENSUS_PARAMETERS")?.as_str())?;
                    let block_period = engine_parameters["blockPeriod"].as_u64().unwrap_or(7);
                    let block_reward = U256::from_dec_str(
                        engine_parameters["blockReward"]
                            .as_str()
                            .unwrap_or_else(|| "5000000000000000000"),
                    )
                    .unwrap_or(U256::from(5) * U256::from(10).pow(18.into()));

                    let validators = Self::validators_from_env()?;
                    for validator_address in validators.iter() {
                        account_states.insert(
                            validator_address.clone(),
                            AccountState {
                                balance: Some(sealer_intrinsic_balance),
                                ..Default::default()
                            },
                        );
                    }

                    ConsensusEngine::GethClique {
                        block_period,
                        block_reward,
                        validators,
                    }
                }
                _ => {
                    return Err(Error::InvalidConsensusEngineType(engine));
                }
            }
        };

        Ok(EthereumChainSpec {
            name,
            network_id: U256::from(0x2323),
            min_gas_limit,
            genesis_block_gas_limit,
            consensus_engine,
            account_states,
        })
    }

    pub fn validators(&self) -> Option<Vec<Address>> {
        self.consensus_engine.validators()
    }

    pub fn as_json(&self) -> serde_json::Value {
        let (engine, seal) = match self.consensus_engine {
            ConsensusEngine::ParityAura {
                block_period,
                block_reward,
                ref validators,
            } => (
                json!({
                    "authorityRound": {
                        "params": {
                            "stepDuration": block_period.to_string(),
                            "blockReward": block_reward.to_string(),
                            "validators": {
                                "list": validators
                            }
                        }
                    }
                }),
                PARITY_DEFAULT_SEAL.clone(),
            ),
            ConsensusEngine::ParityTendermint {
                block_reward,
                propose_timeout,
                prevote_timeout,
                precommit_timeout,
                commit_timeout,
                ref validators,
            } => (
                json! ({
                    "tendermint": {
                        "params": {
                            "blockReward": block_reward.to_string(),
                            "timeoutPropose": propose_timeout,
                            "timeoutPrevote": prevote_timeout,
                            "timeoutPrecommit": precommit_timeout,
                            "timeoutCommit": commit_timeout,
                            "validators": {
                                "list": validators
                            }
                        }
                    }
                }),
                PARITY_DEFAULT_SEAL.clone(),
            ),
            _ => {
                unimplemented!();
            }
        };

        let mut spec = json!({
            "name": self.name,
            "genesis": {
                "difficulty": "0x1",
                "gasLimit": to_0xhex(&self.genesis_block_gas_limit),
                "seal": seal
            } ,
            "params": {
                "maximumExtraDataSize": "0x20",
                "minGasLimit": to_0xhex(&self.min_gas_limit),
                "gasLimitBoundDivisor": "0x400",
                "networkID": to_0xhex(&self.network_id),
                "eip155Transition": 0,
                "maxCodeSize": 24576,
                "maxCodeSizeTransition": 0,
                "maxTransactionSize": usize::max_value(),
                "validateChainIdTransition": 0,
                "validateReceiptsTransition": 0,
                "eip1014Transition": 0,
                "eip1052Transition": 0,
                "eip1283DisableTransition": 0,
                "eip1283Transition": 0,
                "eip140Transition": 0,
                "eip145Transition": 0,
                "eip150Transition": 0,
                "eip155Transition": 0,
                "eip160Transition": 0,
                "eip161abcTransition": 0,
                "eip161dTransition": 0,
                "eip211Transition": 0,
                "eip214Transition": 0,
                "eip658Transition": 0,
                "eip98Transition": 0,
                "kip4Transition": 0,
                "kip6Transition": 0,
                "wasmActivationTransition": 0
            },
            "engine": engine,
            "accounts": {
                "0x0000000000000000000000000000000000000001": {
                    "balance": "1",
                    "builtin": {
                        "name": "ecrecover",
                        "pricing": {
                            "linear": {
                                "base": 3000,
                                "word": 0
                            }
                        }
                    }
                },
                "0x0000000000000000000000000000000000000002": {
                    "balance": "1",
                    "builtin": {
                        "name": "sha256",
                        "pricing": {
                            "linear": {
                                "base": 60,
                                "word": 12
                            }
                        }
                    }
                },
                "0x0000000000000000000000000000000000000003": {
                    "balance": "1",
                    "builtin": {
                        "name": "ripemd160",
                        "pricing": {
                            "linear": {
                                "base": 600,
                                "word": 120
                            }
                        }
                    }
                },
                "0x0000000000000000000000000000000000000004": {
                    "balance": "1",
                    "builtin": {
                        "name": "identity",
                        "pricing": {
                            "linear": {
                                "base": 15,
                                "word": 3
                            }
                        }
                    }
                }
            }
        });

        // insert account states
        spec["accounts"] =
            self.account_states
                .iter()
                .fold(json!({}), |mut spec_accounts, (address, state)| {
                    let address = to_0xhex(address);
                    let state =
                        serde_json::to_value(&state).expect("AccountState is serializable; qed");
                    spec_accounts
                        .as_object_mut()
                        .expect("spec_accounts is an object; qed")
                        .insert(address, state);
                    spec_accounts
                });
        spec
    }
}

pub fn keypair_from_sealer_mnemonic(
    sealer_mnemonic: &str,
    sealer_count: usize,
) -> Result<Vec<SecretKey>, Error> {
    let mnemonic = match Mnemonic::try_from(Language::English, sealer_mnemonic) {
        Ok(m) => m,
        Err(_) => {
            return Err(Error::InvalidMnemonicPhrase(sealer_mnemonic.to_owned()));
        }
    };

    let mut keypairs = Vec::with_capacity(sealer_count);
    for i in 0..sealer_count {
        keypairs.push(generate_keypair_with_index(&mnemonic, i)?);
    }

    Ok(keypairs)
}

fn to_0xhex<V: std::fmt::LowerHex>(value: &V) -> String {
    format!("0x{:x}", value)
}
