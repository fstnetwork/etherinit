use futures::{future, Async, Future, Poll, Stream};
use log::LevelFilter;
use std::str::FromStr;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::network_keeper::NetworkKeeper;
use crate::primitives::EthereumProgram;
use crate::utils::env_var::from_env;

mod error;
pub use self::error::Error;

#[derive(Debug, Clone)]
struct Context {
    network_name: String,
    ethereum_program: EthereumProgram,
    bootnode_service_host: String,
    bootnode_service_port: u16,
    ethereum_node_endpoint: String,
    http_jsonrpc_port: Option<u16>,
    ws_jsonrpc_port: Option<u16>,
}

impl Context {
    fn from_env() -> Result<Context, Error> {
        let network_name = from_env("NETWORK_NAME")?;

        let ethereum_program = EthereumProgram::from_str(from_env("ETHEREUM_PROGRAM")?.as_str())?;
        let ethereum_node_endpoint = from_env("IPC_PATH")?;

        let bootnode_service_host = from_env("BOOTNODE_SERVICE_HOST")?;
        let bootnode_service_port = from_env("BOOTNODE_SERVICE_PORT")?.parse()?;

        let http_jsonrpc_port = from_env("HTTP_JSON_RPC_PORT")
            .ok()
            .map(|port| port.parse().ok())
            .unwrap();

        let ws_jsonrpc_port = from_env("WEBSOCKET_JSON_RPC_PORT")
            .ok()
            .map(|port| port.parse().ok())
            .unwrap();

        Ok(Context {
            network_name,
            ethereum_program,
            ethereum_node_endpoint,
            bootnode_service_host,
            bootnode_service_port,
            http_jsonrpc_port,
            ws_jsonrpc_port,
        })
    }
}

pub fn execute() -> i32 {
    simple_logging::log_to_stderr(LevelFilter::Info);

    let mut runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            error!("{:?}", err);
            return -1;
        }
    };

    let ctx = match Context::from_env() {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("{:?}", err);
            return -1;
        }
    };

    info!("{:?}", ctx);

    let (mut network_keeper, mut ticker, mut ctrl_c) = {
        let mut network_keeper = NetworkKeeper::new(
            ctx.network_name,
            ctx.ethereum_program,
            ctx.bootnode_service_host,
            ctx.bootnode_service_port,
            &ctx.ethereum_node_endpoint,
            ctx.http_jsonrpc_port,
            ctx.ws_jsonrpc_port,
        );

        let ticker = tokio::timer::Interval::new_interval(Duration::from_secs(5));
        let ctrl_c = tokio_signal::ctrl_c().flatten_stream();

        network_keeper.register_enode();
        network_keeper.import_peers();

        (network_keeper, ticker, ctrl_c)
    };

    let poll_fn = future::poll_fn(move || -> Poll<(), Error> {
        loop {
            match ctrl_c.poll() {
                Ok(Async::Ready(_)) => {
                    info!("Signal received, quit");
                    return Ok(Async::Ready(()));
                }
                Ok(Async::NotReady) => {}
                Err(err) => return Err(Error::from(err)),
            }

            if let Err(err) = network_keeper.poll() {
                return Err(Error::from(err));
            }

            match ticker.poll() {
                Ok(Async::Ready(_)) => {
                    network_keeper.register_enode();
                    network_keeper.import_peers();
                }
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => return Err(Error::from(err)),
            }
        }
    });

    match runtime.block_on(poll_fn) {
        Ok(_) => 0,
        Err(err) => {
            error!("{:?}", err);
            -1
        }
    }
}
