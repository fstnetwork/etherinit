use futures::{Async, Future, IntoFuture, Poll, Stream};
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

pub fn execute() -> i32 {
    env_logger::init();

    let ctx = match Context::from_system() {
        Ok(ctx) => {
            info!("Context: {:?}", ctx);
            ctx
        }
        Err(err) => {
            error!("{:?}", err);
            return -1;
        }
    };

    let mut runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            error!("{:?}", err);
            return -1;
        }
    };

    let (ethereum_program, chainspec, static_nodes) = {
        let client =
            BootnodeClient::new(ctx.bootnode_service_host.clone(), ctx.bootnode_service_port);

        match runtime.block_on(fetch_initial_data(&ctx, client)) {
            Ok(data) => data,
            Err(err) => {
                error!("Failed to fetch initialization data, error: {}", err);
                return -1;
            }
        }
    };

    let (ethereum_controller, ethereum_node_endpoint) = {
        let launcher = EthereumLauncher {
            program: ethereum_program,
            chainspec,

            running_mode: ctx.running_mode,

            node_role: ctx.node_role.clone(),
            bootnodes: static_nodes,

            network_port: ctx.network_port,
            http_jsonrpc_port: ctx.http_jsonrpc_port,
            websocket_jsonrpc_port: ctx.websocket_jsonrpc_port,

            parity_logging: ctx.parity_logging,
        };

        match launcher.initialize() {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to initial launcher, error: {:?}", err);
                return -1;
            }
        }

        let ipc_path = launcher.ipc_path();
        (
            EthereumController::new(launcher, ctx.restart_policy),
            ipc_path,
        )
    };

    let network_keeper = NetworkKeeper::new(
        ctx.network_name,
        ethereum_program,
        ctx.bootnode_service_host,
        ctx.bootnode_service_port,
        &ethereum_node_endpoint,
        Some(ctx.http_jsonrpc_port),
        Some(ctx.websocket_jsonrpc_port),
    );

    match runtime.block_on(EthereumService::new(
        ethereum_controller,
        network_keeper,
        ctx.bootnode_update_interval,
    )) {
        Ok(_) => 0,
        Err(err) => {
            error!("{:?}", err);
            -1
        }
    }
}
