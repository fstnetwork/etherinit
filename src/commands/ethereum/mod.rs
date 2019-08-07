use futures::{Async, Future, IntoFuture, Poll, Stream};
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio_signal::unix as UnixSignal;
use tokio_timer::Interval;

use self::context::Context;
pub use self::error::Error;

use crate::bootnode::BootnodeClient;
use crate::ethereum_controller::EthereumController;
use crate::ethereum_launcher::EthereumLauncher;
use crate::network_keeper::NetworkKeeper;
use crate::primitives::{EthereumNodeUrl, EthereumProgram};
use crate::utils::RetryFuture;

mod context;
mod error;

type InitialData = (EthereumProgram, serde_json::Value, Vec<EthereumNodeUrl>);

pub struct EthereumService {
    ethereum_controller: EthereumController,
    network_keeper: NetworkKeeper,
    network_keeper_ticker: Interval,
    shutdown_signal: Box<dyn Future<Item = (), Error = ()> + Send>,
}

impl EthereumService {
    pub fn new(
        ethereum_controller: EthereumController,
        network_keeper: NetworkKeeper,
        network_keeper_update_interval: Duration,
    ) -> EthereumService {
        let network_keeper_ticker = Interval::new_interval(network_keeper_update_interval);

        // force update
        let mut network_keeper = network_keeper;
        network_keeper.import_peers();
        network_keeper.register_enode();

        let shutdown_signal = {
            let signals: Vec<_> = [UnixSignal::SIGINT, UnixSignal::SIGTERM]
                .iter()
                .map(|sig| UnixSignal::Signal::new(*sig).flatten_stream().into_future())
                .collect();

            Box::new(
                futures::future::join_all(signals)
                    .map(|_| ())
                    .map_err(|_| ()),
            )
        };

        EthereumService {
            ethereum_controller,
            network_keeper,
            network_keeper_ticker,
            shutdown_signal,
        }
    }
}

impl Future for EthereumService {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(Async::Ready(_)) = self.shutdown_signal.poll() {
            self.ethereum_controller.close();
        }

        match self.ethereum_controller.poll() {
            Ok(Async::Ready(_)) => {
                return Ok(Async::Ready(()));
            }
            Ok(Async::NotReady) => {}
            Err(err) => return Err(Error::from(err)),
        }

        if let Err(err) = self.network_keeper.poll() {
            return Err(Error::from(err));
        }

        match self.network_keeper_ticker.poll() {
            Ok(Async::Ready(_)) => {
                self.network_keeper.register_enode();
                self.network_keeper.import_peers();
                Ok(Async::NotReady)
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Err(Error::from(err)),
        }
    }
}

pub fn fetch_initial_data(
    ctx: &Context,
    bootinfo_fetcher: BootnodeClient,
) -> Box<dyn Future<Item = InitialData, Error = Error> + Send> {
    let timeout = Duration::from_secs(5);
    let retry_limit = 100;
    let network_name = ctx.network_name.clone();

    let system_info = RetryFuture::new(
        Some("fetch system info".to_owned()),
        timeout,
        retry_limit,
        Box::new({
            let network_name = network_name.clone();
            let fetcher = bootinfo_fetcher.clone();
            move || {
                Box::new(
                    fetcher
                        .fetch_system_info(&network_name)
                        .then(|data| match data {
                            Ok(info) => {
                                info!("System info: {:?}", info);
                                Ok(info.consensus_engine.program())
                            }
                            Err(_err) => Err(Error::FailedToFetchSystemInfo),
                        }),
                )
            }
        }),
    );

    let chainspec = RetryFuture::new(
        Some("fetch chain specification".to_owned()),
        timeout,
        retry_limit,
        Box::new({
            let fetcher = bootinfo_fetcher.clone();
            let network_name = network_name.clone();
            move || {
                Box::new(
                    fetcher
                        .fetch_chainspec(&network_name)
                        .then(|data| match data {
                            Ok(spec) => Ok(spec),
                            Err(_err) => Err(Error::FailedToFetchChainSpec),
                        }),
                )
            }
        }),
    );

    let nodes = RetryFuture::new(
        Some("fetch peer info".to_owned()),
        timeout,
        retry_limit,
        Box::new({
            let is_first_miner = ctx.is_first_miner();
            let fetcher = bootinfo_fetcher.clone();
            let network_name = network_name.clone();
            move || {
                Box::new(
                    fetcher
                        .fetch_enodes(&network_name)
                        .then(move |data| match data {
                            Ok(nodes) => match (nodes.len(), is_first_miner) {
                                (0, true) => Ok(vec![]),
                                (0, false) => {
                                    info!("No node fetched, try again later...");
                                    Err(Error::FailedToFetchPeers)
                                }
                                _ => {
                                    info!("{} node(s) fetched", nodes.len());
                                    Ok(nodes)
                                }
                            },
                            Err(_err) => Err(Error::FailedToFetchPeers),
                        }),
                )
            }
        }),
    );

    Box::new(system_info.join3(chainspec, nodes).into_future())
}

struct Payload {
    runtime: Runtime,
    context: Context,
    ethereum_controller: EthereumController,
    ethereum_program: EthereumProgram,
    ethereum_node_endpoint: PathBuf,
}

impl Payload {
    fn new() -> Option<Payload> {
        let mut runtime = match Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => {
                error!("{:?}", err);
                return None;
            }
        };

        let context = match Context::from_system() {
            Ok(context) => {
                info!("Context: {:?}", context);
                context
            }
            Err(err) => {
                error!("{:?}", err);
                return None;
            }
        };

        let (ethereum_program, chainspec, static_nodes) = {
            let client = BootnodeClient::new(
                context.bootnode_service_host.clone(),
                context.bootnode_service_port,
            );

            match runtime.block_on(fetch_initial_data(&context, client)) {
                Ok(data) => data,
                Err(err) => {
                    error!("Failed to fetch initialization data, error: {}", err);
                    return None;
                }
            }
        };

        let launcher = EthereumLauncher {
            program: ethereum_program,
            chainspec,

            running_mode: context.running_mode,

            node_role: context.node_role.clone(),
            bootnodes: static_nodes,

            network_port: context.network_port,
            http_jsonrpc_port: context.http_jsonrpc_port,
            websocket_jsonrpc_port: context.websocket_jsonrpc_port,
            ipc_path: context.ipc_path.clone(),

            config_file_path: context.config_file_path.clone(),

            parity_tx_mem_limit: context.parity_tx_queue_mem_limit,
            parity_tx_queue_size: context.parity_tx_queue_size,
            parity_tx_queue_per_sender: context.parity_tx_queue_per_sender,

            parity_logging: context.parity_logging.clone(),
        };

        match launcher.initialize() {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to initial launcher, error: {:?}", err);
                return None;
            }
        }

        let ipc_path = launcher.ipc_path();
        let restart_policy = context.restart_policy.clone();
        Some(Payload {
            runtime,
            context,
            ethereum_controller: EthereumController::new(launcher, restart_policy),
            ethereum_program,
            ethereum_node_endpoint: ipc_path,
        })
    }
}

pub fn run_init() -> i32 {
    env_logger::init();

    Payload::new().map(|_| 0).unwrap_or(-1)
}

pub fn run_exec() -> i32 {
    env_logger::init();

    Payload::new()
        .map(|payload| payload.ethereum_controller.unix_exec())
        .unwrap_or(-1)
}

pub fn run_full() -> i32 {
    env_logger::init();

    let Payload {
        mut runtime,
        context,
        ethereum_controller,
        ethereum_program,
        ethereum_node_endpoint,
    } = match Payload::new() {
        Some(payload) => payload,
        None => return -1,
    };

    let network_keeper = NetworkKeeper::new(
        context.network_name,
        ethereum_program,
        context.bootnode_service_host,
        context.bootnode_service_port,
        &ethereum_node_endpoint,
        Some(context.http_jsonrpc_port),
        Some(context.websocket_jsonrpc_port),
    );

    let ethereum_service = EthereumService::new(
        ethereum_controller,
        network_keeper,
        context.bootnode_update_interval,
    );

    runtime
        .block_on(ethereum_service)
        .map(|_| 0)
        .map_err(|err| {
            error!("{:?}", err);
            -1
        })
        .unwrap()
}
