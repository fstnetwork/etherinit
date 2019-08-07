use futures::{future, Async, Future, Poll};
use log::LevelFilter;
use tokio::runtime::Runtime;
use web3::types::SyncState;

use crate::utils::{
    env_var::from_env,
    exit_code::{EXIT_FAILURE, EXIT_SUCCESS},
};

pub fn execute() -> i32 {
    simple_logging::log_to_stderr(LevelFilter::Info);

    let ethereum_node_endpoint = match from_env("IPC_PATH") {
        Ok(ipc_path) => ipc_path,
        Err(err) => {
            error!("{:?}", err);
            return -1;
        }
    };

    let mut runtime = Runtime::new().unwrap();

    let mut web3_ipc = web3::transports::Ipc::new(ethereum_node_endpoint).unwrap();
    let web3 = web3::Web3::new(web3_ipc.clone());

    let mut syncing_future = web3.eth().syncing();
    let poll_fn = future::poll_fn(move || -> Poll<bool, web3::Error> {
        loop {
            let _ = web3_ipc.poll();
            match syncing_future.poll() {
                Ok(Async::Ready(SyncState::NotSyncing)) => {
                    return Ok(Async::Ready(true));
                }
                Ok(Async::Ready(SyncState::Syncing(_))) => {
                    return Ok(Async::Ready(false));
                }
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => return Err(err),
            }
        }
    });

    match runtime.block_on(poll_fn) {
        Ok(true) => {
            info!("Blockchain is not syncing, Ethereum node is ready!");
            EXIT_SUCCESS
        }
        Ok(false) => {
            warn!("Blockchain is syncing, Ethereum node is not ready!");
            EXIT_FAILURE
        }
        Err(err) => {
            error!("{:?}", err);
            EXIT_FAILURE
        }
    }
}
