use ethereum_types::{Address, U256};
use ethsign::{keyfile::KeyFile, Protected, SecretKey};
use std::io::Write;
use std::path::PathBuf;

use crate::primitives::{EthereumNodeUrl, NodeRole};

use super::{Error, RunningMode};

pub fn create_key_directory(config_dir_path: &PathBuf) -> Result<PathBuf, Error> {
    let mut path = PathBuf::from(config_dir_path);
    path.push("keys");

    std::fs::create_dir_all(path.clone())?;
    Ok(path)
}

pub fn create_key_file(
    key_dir_path: &PathBuf,
    private_key: &SecretKey,
    passphrase: &str,
) -> Result<PathBuf, Error> {
    let passphrase = Protected::from(passphrase.as_bytes());
    let keyfile = KeyFile {
        id: "6845de15-c9d1-4af6-8386-da01205284d7".to_owned(),
        version: 3,
        crypto: private_key.to_crypto(
            &passphrase,
            std::num::NonZeroU32::new(1024).expect("1024 is none zero; qed"),
        )?,
        address: Some(ethsign::keyfile::Bytes(
            private_key.public().address().to_vec(),
        )),
    };

    let mut path = PathBuf::from(key_dir_path);
    path.push("signer_keyfile.json");

    serde_json::to_writer(
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())?,
        &keyfile,
    )?;

    Ok(path)
}

pub fn create_passphrase_file(config_dir: &PathBuf, passphrase: &str) -> Result<PathBuf, Error> {
    let mut path = PathBuf::from(config_dir);
    path.push("sealer_passphrase");

    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path.clone())?
        .write_all(passphrase.as_bytes())?;

    Ok(path)
}

pub fn create_reserverd_peers_file(
    config_dir: &PathBuf,
    bootnodes: &[EthereumNodeUrl],
) -> Result<PathBuf, Error> {
    let mut path = PathBuf::from(config_dir);
    path.push("reserved_peers");

    let data = bootnodes
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path.clone())?
        .write_all(data.as_bytes())?;

    Ok(path)
}

pub fn create_spec_file(
    config_dir: &PathBuf,
    chainspec: &serde_json::Value,
) -> Result<PathBuf, Error> {
    let mut path = PathBuf::from(config_dir);
    path.push("spec.json");

    serde_json::to_writer(
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())?,
        &chainspec,
    )?;

    Ok(path)
}

#[derive(Debug, Clone)]
pub struct ParityMinerOptions {
    pub force_sealing: bool,
    pub gas_cap: String,
    pub gas_floor_target: String,
    pub sealer_address: Address,
    pub sealer_passphrase_file_path: String,
}

#[derive(Debug, Clone)]
pub struct ParityConfig {
    pub running_mode: RunningMode,

    pub db_path: String,
    pub node_role: NodeRole,
    pub miner_options: Option<ParityMinerOptions>,

    pub identity: String,
    pub spec_path: String,

    pub bootnodes: Vec<EthereumNodeUrl>,
    pub reserved_peers_file_path: String,

    pub ipc_path: String,
    pub network_port: u16,
    pub http_jsonrpc_port: u16,
    pub websocket_jsonrpc_port: u16,

    pub tx_queue_size: u32,
    pub tx_queue_mem_limit: u32,
    pub tx_queue_per_sender: u32,

    pub logging: Option<String>,
}

impl ParityConfig {
    pub fn toml_config(&self) -> toml::Value {
        let db_path = self.db_path.clone();
        let chain = self.spec_path.clone();
        let identity = self.identity.clone();
        let logging = match (&self.logging, &self.miner_options) {
            (Some(l), _) => l.clone(),
            // logging options for miner
            (None, Some(_)) => "network=info,miner=info,mode=info".to_owned(),
            // logging options for transactor
            (None, None) => {
                "txqueue=debug,own_tx=debug,network=info,miner=info,mode=info".to_owned()
            }
        };

        let (http_apis, ws_apis) = {
            let apis = match self.running_mode {
                RunningMode::Production => vec!["eth", "net", "parity", "web3"],
                RunningMode::Development => vec!["all"],
            };
            (apis.clone(), apis)
        };

        let (engine_signer, author, unlock, force_sealing, password, gas_cap, gas_floor_target) = {
            match self.miner_options.clone() {
                Some(options) => {
                    let engine_signer = format!("{:x?}", options.sealer_address);
                    (
                        engine_signer.clone(),
                        engine_signer.clone(),
                        engine_signer,
                        options.force_sealing,
                        options.sealer_passphrase_file_path,
                        options.gas_cap,
                        options.gas_floor_target,
                    )
                }
                None => (
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ),
            }
        };

        let bootnodes: Vec<_> = self
            .bootnodes
            .iter()
            .map(EthereumNodeUrl::to_string)
            .collect();
        let reserved_peers = self.reserved_peers_file_path.clone();
        let ipc_path = self.ipc_path.clone();
        let network_port = self.network_port;
        let http_jsonrpc_port = self.http_jsonrpc_port;
        let websocket_jsonrpc_port = self.websocket_jsonrpc_port;
        let pruning = match self.node_role {
            NodeRole::Miner { .. } | NodeRole::Transactor => "fast",
            NodeRole::Syncer => "archive",
        };

        let tx_queue_size = self.tx_queue_size;
        let tx_queue_mem_limit = self.tx_queue_mem_limit;
        let tx_queue_per_sender = self.tx_queue_per_sender;

        let tx_gas_limit = U256::max_value().to_string();

        match self.node_role {
            NodeRole::Miner { .. } => {
                toml! {
                    [parity]
                    db_path = db_path
                    chain = chain
                    identity = identity
                    no_persistent_txqueue = false
                    no_download = true
                    light = false

                    [network]
                    bootnodes = bootnodes
                    port = network_port
                    reserved_peers = reserved_peers
                    reserved_only = false
                    min_peers = 32
                    max_peers = 128
                    snapshot_peers = 16
                    max_pending_peers = 32

                    [account]
                    unlock = [ unlock ]
                    password = [ password ]

                    [mining]
                    author = author
                    engine_signer = engine_signer
                    reseal_on_txs = "none"
                    usd_per_tx = "0"
                    force_sealing = force_sealing
                    gas_floor_target = gas_floor_target
                    gas_cap = gas_cap
                    tx_queue_size = tx_queue_size
                    tx_queue_mem_limit = tx_queue_mem_limit
                    tx_queue_per_sender = tx_queue_per_sender
                    tx_gas_limit = tx_gas_limit

                    [websockets]
                    interface = "0.0.0.0"
                    port = websocket_jsonrpc_port
                    hosts = ["all"]
                    apis = ws_apis
                    origins = ["all"]

                    [rpc]
                    interface = "0.0.0.0"
                    port = http_jsonrpc_port
                    hosts = ["all"]
                    apis = http_apis
                    max_payload = 128

                    [ipc]
                    disable = false
                    path = ipc_path
                    apis = ["all"]

                    [footprint]
                    db_compaction = "ssd"
                    pruning = pruning

                    [misc]
                    logging = logging
                    color = true
                }
            }
            NodeRole::Transactor | NodeRole::Syncer => {
                toml! {
                    [parity]
                    chain = chain
                    identity = identity
                    no_persistent_txqueue = false
                    no_download = true
                    light = false

                    [network]
                    bootnodes = bootnodes
                    port = network_port
                    reserved_peers = reserved_peers
                    reserved_only = false
                    min_peers = 32
                    max_peers = 256
                    snapshot_peers = 16
                    max_pending_peers = 32

                    [mining]
                    tx_queue_size = tx_queue_size
                    tx_queue_mem_limit = tx_queue_mem_limit
                    tx_queue_per_sender = tx_queue_per_sender

                    [websockets]
                    interface = "0.0.0.0"
                    port = websocket_jsonrpc_port
                    hosts = ["all"]
                    apis = ws_apis
                    origins = ["all"]

                    [rpc]
                    interface = "0.0.0.0"
                    port = http_jsonrpc_port
                    hosts = ["all"]
                    apis = http_apis
                    max_payload = 128

                    [ipc]
                    disable = false
                    path = ipc_path
                    apis = ["all"]

                    [footprint]
                    db_compaction = "ssd"
                    pruning = pruning

                    [misc]
                    logging = logging
                    color = true
                }
            }
        }
    }

    pub fn save_as_file(&self, config_file_path: &PathBuf) -> Result<PathBuf, Error> {
        let config = self.toml_config();
        let data = toml::to_string(&config).expect("config is serializable; qed");
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(config_file_path)?
            .write_all(data.as_bytes())?;
        Ok(config_file_path.clone())
    }
}
