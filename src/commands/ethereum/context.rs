use hdwallet::mnemonic::{Language, Mnemonic};
use std::str::FromStr;
use std::time::Duration;

use crate::ethereum_controller::RestartPolicy;
use crate::ethereum_launcher::RunningMode;
use crate::primitives::NodeRole;
use crate::utils::env_var::from_env;

use super::Error;

#[derive(Debug, Clone)]
pub struct Context {
    /// running mode of current context
    pub running_mode: RunningMode,

    /// name of this ethereum network
    pub network_name: String,

    /// node type of this container
    pub node_role: NodeRole,

    /// restart policy
    pub restart_policy: RestartPolicy,

    /// Ethereum P2P Network port
    pub network_port: u16,

    /// Ethereum Client HTTP JSON-RPC port
    pub http_jsonrpc_port: u16,

    /// Ethereum Client WebSocket JSON-RPC port
    pub websocket_jsonrpc_port: u16,

    /// Parity Ethereum: logging options
    pub parity_logging: Option<String>,

    /// Parity Ethereum: Maximum amount of memory that can be used by the transaction queue in MiB
    pub parity_tx_queue_mem_limit: Option<u32>,

    /// Parity Ethereum: Maximum amount of transactions in the queue
    pub parity_tx_queue_size: Option<u32>,

    /// Parity Ethereum: Maximum number of transactions per sender in the queue.
    pub parity_tx_queue_per_sender: Option<u32>,

    /// hostname of bootnode service
    pub bootnode_service_host: String,

    /// port of bootnode service
    pub bootnode_service_port: u16,

    /// interval for update enode URL to bootnode service
    pub bootnode_update_interval: Duration,
}

impl Context {
    pub fn from_system() -> Result<Context, Error> {
        use std::env;
        let network_name = from_env("NETWORK_NAME")?;

        let running_mode =
            RunningMode::from_str(&from_env("RUNNING_MODE").unwrap_or("production".to_owned()))
                .unwrap_or(RunningMode::Production);

        let node_role = {
            let node_role = from_env("NODE_ROLE")?;

            match node_role.to_lowercase().as_ref() {
                "syncer" => NodeRole::Syncer,
                "transactor" => NodeRole::Transactor,
                "miner" => {
                    let index: usize = from_env("MINER_INDEX")?.parse()?;
                    let seed = from_env("SEALER_MNEMONIC_PHRASE")?;
                    let mnemonic = match Mnemonic::try_from(Language::English, seed.as_str()) {
                        Ok(m) => m,
                        Err(_err) => {
                            return Err(Error::InvalidMnemonicPhrase(seed));
                        }
                    };

                    let parity_gas_floor_target = from_env("PARITY_GAS_FLOOR_TARGET").ok();
                    let parity_gas_cap = from_env("PARITY_GAS_CAP").ok();

                    NodeRole::Miner {
                        sealer_mnemonic: mnemonic,
                        index,
                        parity_gas_cap,
                        parity_gas_floor_target,
                    }
                }
                _ => return Err(Error::UnknownNodeRole(node_role)),
            }
        };

        let parity_logging = env::var("PARITY_LOGGING").ok();

        Ok(Context {
            running_mode,

            network_name,

            node_role,

            network_port: from_env("P2P_NETWORK_SERVICE_PORT")?.parse()?,
            http_jsonrpc_port: from_env("HTTP_JSON_RPC_PORT")?.parse()?,
            websocket_jsonrpc_port: from_env("WEBSOCKET_JSON_RPC_PORT")?.parse()?,

            restart_policy: RestartPolicy::Always,

            bootnode_service_host: from_env("BOOTNODE_SERVICE_HOST")?,
            bootnode_service_port: from_env("BOOTNODE_SERVICE_PORT")?.parse()?,
            bootnode_update_interval: Duration::from_secs(
                from_env("BOOTNODE_SERVICE_UPDATE_INTERVAL")
                    .unwrap_or_else(|_| "5".into())
                    .parse()?,
            ),

            parity_tx_queue_mem_limit: from_env("PARITY_TX_QUEUE_MEM_LIMIT")
                .map(|s| s.parse().unwrap_or(4))
                .ok(),
            parity_tx_queue_per_sender: from_env("PARITY_TX_QUEUE_PER_SENDER")
                .map(|s| s.parse().unwrap_or(16))
                .ok(),
            parity_tx_queue_size: from_env("PARITY_TX_QUEUE_SIZE")
                .map(|s| s.parse().unwrap_or(8192))
                .ok(),

            parity_logging,
        })
    }

    #[inline]
    pub fn is_first_miner(&self) -> bool {
        match self.node_role {
            NodeRole::Miner { index, .. } => 0 == index,
            NodeRole::Transactor => false,
            NodeRole::Syncer => false,
        }
    }
}
