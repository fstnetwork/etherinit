use futures::{Async, Future};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tower_web::ServiceBuilder;

use crate::bootnode::{BootnodeService, BootnodeTracker, EthereumNetwork};
use crate::primitives::EthereumChainSpec;
use crate::utils::env_var::from_env;

pub fn execute() -> i32 {
    env_logger::init();

    let mut runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            error!("{}", err);
            return -1;
        }
    };

    let tracker = {
        info!("Generating Ethereum chain spec from environment variables...");

        // 5 minutes
        let node_lifetime = Duration::from_secs(5 * 60);
        let spec = match EthereumChainSpec::from_env() {
            Ok(spec) => spec,
            Err(err) => {
                error!("{}", err);
                return -1;
            }
        };
        let network = EthereumNetwork::new(spec, node_lifetime);
        let mut tracker = BootnodeTracker::new();
        tracker.add_ethereum_network(network);

        Arc::new(Mutex::new(tracker))
    };

    let tcp_listener = {
        let socket_addr = match from_env("BOOTNODE_SOCKET") {
            Ok(addr) => match addr.parse() {
                Ok(addr) => addr,
                Err(err) => {
                    error!("failed to parse bootnode socket address, error: {}", err);
                    return -1;
                }
            },
            Err(err) => {
                error!("failed to get bootnode socket address, error: {}", err);
                return -1;
            }
        };

        info!("Listening on http://{}...", socket_addr);
        match TcpListener::bind(&socket_addr) {
            Ok(listener) => listener,
            Err(err) => {
                error!(
                    "failed to listen on socket address: {}, error: {}",
                    socket_addr, err
                );
                return -1;
            }
        }
    };

    let mut server = ServiceBuilder::new()
        .resource(BootnodeService::with_tracker(tracker.clone()))
        .serve(tcp_listener.incoming());

    let poll_fn = futures::future::poll_fn({
        move || loop {
            let _ = tracker.lock().poll();

            match server.poll() {
                Ok(Async::Ready(())) => return Ok(Async::Ready(())),
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                }
                Err(err) => {
                    return Err(err);
                }
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
