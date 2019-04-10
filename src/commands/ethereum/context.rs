use std::time::Duration;

use super::ethereum_controller::RestartPolicy;
use super::hdwallet::mnemonic::{Language, Mnemonic};
use super::primitives::NodeRole;
use super::utils::env_var::from_env;
use super::{Error, ErrorKind};

#[derive(Debug, Clone)]
pub struct Context {
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

    /// Parity Ethereum logging options
    pub parity_logging: Option<String>,

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
                            return Err(Error::from(ErrorKind::InvalidMnemonicPhrase(seed)));
                        }
                    };

                    NodeRole::Miner {
                        sealer_mnemonic: mnemonic,
                        index,
                    }
                }
                _ => return Err(Error::from(ErrorKind::UnknownNodeRole(node_role))),
            }
        };

        let parity_logging = env::var("PARITY_LOGGING").ok();

        Ok(Context {
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
                    .unwrap_or("5".into())
                    .parse()?,
            ),

            parity_logging,
        })
    }

    #[inline]
    pub fn is_first_miner(&self) -> bool {
        match self.node_role {
            NodeRole::Miner { index, .. } => return 0 == index,
            NodeRole::Transactor => false,
            NodeRole::Syncer => false,
        }
    }
}
