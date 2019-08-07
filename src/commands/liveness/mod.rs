use futures::{future, Async, Future, Poll};
use log::LevelFilter;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::timer::Delay;

use crate::utils::{
    env_var::from_env,
    exit_code::{EXIT_FAILURE, EXIT_SUCCESS},
};

mod error;
use self::error::Error;

struct Context {
    liveness_timeout: Duration,
    ipc: String,
    http: String,
    websocket: String,
}

impl Context {
    fn from_env() -> Result<Context, Error> {
        Ok(Context {
            liveness_timeout: Duration::from_secs(
                from_env("LIVENESS_PROBE_TIMEOUT_SEC")
                    .unwrap_or("10".to_owned())
                    .parse()?,
            ),
            ipc: from_env("IPC_PATH")?,
            http: format!(
                "http://127.0.0.1:{}",
                from_env("HTTP_JSON_RPC_PORT")?.parse::<u16>()?
            ),
            websocket: format!(
                "ws://127.0.0.1:{}",
                from_env("WEBSOCKET_JSON_RPC_PORT")?.parse::<u16>()?
            ),
        })
    }
}

pub fn execute() -> i32 {
    simple_logging::log_to_stderr(LevelFilter::Info);

    let context = match Context::from_env() {
        Ok(context) => context,
        Err(err) => {
            error!("{:?}", err);
            return -1;
        }
    };

    let mut runtime = Runtime::new().unwrap();

    let mut ipc_transport = web3::transports::Ipc::new(context.ipc).unwrap();
    let mut http_transport = web3::transports::Http::new(&context.http).unwrap();
    let mut ws_transport = web3::transports::WebSocket::new(&context.websocket).unwrap();
    let web3_ipc = web3::Web3::new(ipc_transport.clone());
    let web3_http = web3::Web3::new(http_transport.clone());
    let web3_ws = web3::Web3::new(ws_transport.clone());
    let ipc_tester = web3_ipc.eth().block_number().map(|n| {
        info!("block number(IPC): {}", n);
    });
    let http_tester = web3_http.eth().block_number().map(|n| {
        info!("block number(HTTP JSON-RPC): {}", n);
    });
    let ws_tester = web3_ws.eth().block_number().map(|n| {
        info!("block number(WebSocket JSON-RPC): {}", n);
    });

    let mut tester = ipc_tester.join3(http_tester, ws_tester);
    let mut deadline = Delay::new(Instant::now() + context.liveness_timeout);

    let poll_fn = future::poll_fn(move || -> Poll<bool, Error> {
        loop {
            let _ = ipc_transport.poll();
            let _ = http_transport.poll();
            let _ = ws_transport.poll();

            match deadline.poll() {
                Ok(Async::Ready(_)) => return Err(Error::Timeout),
                Err(err) => return Err(Error::Timer(err)),
                _ => {}
            }

            match tester.poll() {
                Ok(Async::Ready(_)) => return Ok(Async::Ready(true)),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => return Err(err.into()),
            }
        }
    });

    match runtime.block_on(poll_fn) {
        Ok(true) => {
            info!("Ethereum node is alive");
            EXIT_SUCCESS
        }
        Ok(false) => {
            warn!("Ethereum node is not alive");
            EXIT_FAILURE
        }
        Err(err) => {
            error!("{:?}", err);
            EXIT_FAILURE
        }
    }
}
