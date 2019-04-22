use ethereum_types::Address;
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use std::process::Command;
use tokio_process::{Child as ChildProcess, CommandExt};

use crate::primitives::{EthereumNodeUrl, EthereumProgram, NodeRole};

mod error;
mod geth;
mod parity;

pub use self::error::Error;

const PARITY_EXECUTABLE_PATH: &str = "parity";
const GETH_EXECUTABLE_PATH: &str = "geth";
const DEFAULT_SEALER_KEYFILE_PASSPHRASE: &str = "0123456789";

pub struct EthereumLauncher {
    pub program: EthereumProgram,
    pub chainspec: JsonValue,

    pub node_role: NodeRole,
    pub bootnodes: Vec<EthereumNodeUrl>,

    pub network_port: u16,
    pub http_jsonrpc_port: u16,
    pub websocket_jsonrpc_port: u16,

    pub parity_logging: Option<String>,
}

impl EthereumLauncher {
    pub fn chain_data_dir_path(&self) -> PathBuf {
        PathBuf::from(std::env::var("CHAIN_DATA_ROOT").unwrap_or_else(|_| "/chain-data".into()))
    }

    pub fn config_dir_path(&self) -> PathBuf {
        let mut path = PathBuf::from(std::env::var("CONFIG_ROOT").unwrap_or_else(|_| "/".into()));
        path.push(match self.program {
            EthereumProgram::Parity => PathBuf::from("parity-config"),
            EthereumProgram::GoEthereum => PathBuf::from("geth-config"),
        });
        path
    }

    #[allow(unused)]
    pub fn local_jsonrpc_url(&self) -> String {
        format!("http://127.0.0.1:{}/", self.http_jsonrpc_port)
    }

    #[allow(unused)]
    pub fn local_websocket_jsonrpc_url(&self) -> String {
        format!("ws://127.0.0.1:{}/", self.websocket_jsonrpc_port)
    }

    pub fn ipc_path(&self) -> PathBuf {
        let mut path = self.config_dir_path();
        path.push(match self.program {
            EthereumProgram::Parity => "parity.ipc",
            EthereumProgram::GoEthereum => "geth.ipc",
        });
        path
    }

    pub fn config_file_path(&self) -> PathBuf {
        let mut path_buf = self.config_dir_path();
        path_buf.push(match self.program {
            EthereumProgram::Parity => "config.toml",
            EthereumProgram::GoEthereum => "config.toml",
        });
        path_buf
    }

    pub fn initialize(&self) -> Result<String, Error> {
        match self.program {
            EthereumProgram::Parity => self.initialize_parity(),
            EthereumProgram::GoEthereum => self.initialize_geth(),
        }
    }

    fn initialize_parity(&self) -> Result<String, Error> {
        let config_dir = self.config_dir_path();
        std::fs::create_dir_all(config_dir.clone())?;

        let db_path = self.chain_data_dir_path();
        std::fs::create_dir_all(db_path.clone())?;

        let spec_file_path = parity::create_spec_file(&config_dir, &self.chainspec)?;
        let reserved_peers_file_path =
            parity::create_reserverd_peers_file(&config_dir, &self.bootnodes)?;

        match self.node_role.clone() {
            NodeRole::Miner { .. } => {
                let passphrase = String::from(DEFAULT_SEALER_KEYFILE_PASSPHRASE);

                let sealer_key = self
                    .node_role
                    .validator_keypair()
                    .expect("index must be valid");
                let sealer_address = Address::from(*sealer_key.public().address());

                let key_dir = parity::create_key_directory(&config_dir)?;
                let key_file_path = parity::create_key_file(&key_dir, &sealer_key, &passphrase)?;

                info!(target: "launcher", "create key file {:?} for {:?}",
                      key_file_path, sealer_address);

                let sealer_password_file_path =
                    parity::create_passphrase_file(&config_dir, &passphrase)?;

                let config_file_path: String = {
                    let config = parity::ParityConfig {
                        miner_options: Some(parity::ParityMinerOptions {
                            force_sealing: true,
                            sealer_address,
                            sealer_passphrase_file_path: sealer_password_file_path
                                .to_str()
                                .expect("sealer passphrase file path")
                                .to_owned(),
                        }),

                        db_path: db_path.to_str().expect("db directory path").to_owned(),
                        node_role: self.node_role.clone(),

                        identity: self.node_role.identity(),
                        spec_path: spec_file_path.to_str().expect("spec file path").to_owned(),
                        bootnodes: self.bootnodes.clone(),
                        reserved_peers_file_path: reserved_peers_file_path
                            .to_str()
                            .expect("reserved peers file")
                            .to_owned(),

                        ipc_path: self.ipc_path().to_str().expect("ipc path").to_owned(),
                        network_port: self.network_port,
                        http_jsonrpc_port: self.http_jsonrpc_port,
                        websocket_jsonrpc_port: self.websocket_jsonrpc_port,

                        logging: self.parity_logging.clone(),
                    };

                    config
                        .save_as_file(&self.config_file_path())?
                        .to_str()
                        .expect("config file path")
                        .into()
                };

                Command::new(PARITY_EXECUTABLE_PATH)
                    .arg(format!("--config={}", config_file_path))
                    .arg("account")
                    .arg("import")
                    .arg(key_dir.to_str().expect("key directory"))
                    .spawn()?;

                Ok(config_file_path)
            }
            NodeRole::Transactor | NodeRole::Syncer => {
                let config_file_path: String = {
                    let config = parity::ParityConfig {
                        miner_options: None,

                        db_path: db_path.to_str().expect("db directory path").to_owned(),
                        node_role: self.node_role.clone(),

                        identity: self.node_role.identity(),
                        spec_path: spec_file_path.to_str().expect("spec file path").to_owned(),
                        bootnodes: self.bootnodes.clone(),
                        reserved_peers_file_path: reserved_peers_file_path
                            .to_str()
                            .expect("reserved peers file")
                            .to_owned(),

                        ipc_path: self.ipc_path().to_str().expect("ipc path").to_owned(),
                        network_port: self.network_port,
                        http_jsonrpc_port: self.http_jsonrpc_port,
                        websocket_jsonrpc_port: self.websocket_jsonrpc_port,

                        logging: self.parity_logging.clone(),
                    };

                    config
                        .save_as_file(&self.config_file_path())?
                        .to_str()
                        .expect("config file path")
                        .into()
                };

                Ok(config_file_path)
            }
        }
    }

    fn initialize_geth(&self) -> Result<String, Error> {
        unimplemented!();
    }

    fn execute_command(&self) -> (Command, Vec<String>) {
        let config_file_path =
            String::from(self.config_file_path().to_str().expect("config file path"));
        match self.program {
            EthereumProgram::Parity => (
                Command::new(PARITY_EXECUTABLE_PATH),
                vec![
                    format!("--config={}", config_file_path),
                    "--no-hardware-wallets".into(),
                ],
            ),
            EthereumProgram::GoEthereum => (Command::new(GETH_EXECUTABLE_PATH), vec![]),
        }
    }

    pub fn execute_async(&self) -> Result<ChildProcess, std::io::Error> {
        let (mut cmd, args) = self.execute_command();
        cmd.args(args).spawn_async()
    }
}
