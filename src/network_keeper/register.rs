use futures::{sync::mpsc, Async, Future, Poll, Stream};
use std::str::FromStr;
use url::Url;

use crate::primitives::{EthereumNodeUrl, EthereumProgram};

use super::{BootnodeClient, Error, Web3};

type UrlFetcher = Box<dyn Future<Item = EthereumNodeUrl, Error = Error> + Send>;
type UrlRegister = Box<dyn Future<Item = bool, Error = Error> + Send>;

pub struct Register {
    inner: Inner,

    event_receiver: mpsc::UnboundedReceiver<()>,
    event_sender: mpsc::UnboundedSender<()>,

    ethereum_program: EthereumProgram,
    network_name: String,
    web3: Web3,
    bootnode_client: BootnodeClient,

    http_jsonrpc_port: Option<u16>,
    ws_jsonrpc_port: Option<u16>,
}

impl Register {
    pub fn new(
        ethereum_program: EthereumProgram,
        network_name: String,
        web3: Web3,
        bootnode_client: BootnodeClient,
        http_jsonrpc_port: Option<u16>,
        ws_jsonrpc_port: Option<u16>,
    ) -> Register {
        let (event_sender, event_receiver) = mpsc::unbounded();
        Register {
            inner: Inner::Idle,
            event_receiver,
            event_sender,
            ethereum_program,
            network_name,
            web3,
            bootnode_client,
            http_jsonrpc_port,
            ws_jsonrpc_port,
        }
    }

    pub fn update(&self) {
        self.event_sender
            .unbounded_send(())
            .expect("receiver always existed; qed");
    }
}

impl Future for Register {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.inner = match self.inner {
                Inner::Idle => match self.event_receiver.poll() {
                    Ok(Async::Ready(Some(_))) => {
                        Inner::fetch_url(&self.web3, self.ethereum_program)
                    }
                    _ => return Ok(Async::NotReady),
                },
                Inner::FetchingUrl { ref mut fetcher } => match fetcher.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(enode_url)) => {
                        info!(
                            "Node Register: Ethereum Node URL fetched, URL: {}",
                            enode_url.to_string()
                        );

                        let host = &enode_url.host;
                        let http_jsonrpc_port = self.http_jsonrpc_port.map(|port| {
                            Url::parse(&format!("http://{}:{}", host, port))
                                .expect("the URL is valid")
                        });

                        let ws_jsonrpc_port = self.ws_jsonrpc_port.map(|port| {
                            Url::parse(&format!("ws://{}:{}", host, port))
                                .expect("the URL is valid")
                        });

                        Inner::register_url(
                            &self.bootnode_client,
                            &self.network_name,
                            enode_url,
                            http_jsonrpc_port,
                            ws_jsonrpc_port,
                        )
                    }
                    Err(err) => {
                        warn!(
                            "Node Register: Failed to fetch Ethereum Node URL, error: {:?}",
                            err
                        );
                        Inner::idle()
                    }
                },
                Inner::RegisteringUrl { ref mut register } => {
                    match register.poll() {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(_)) => {
                            info!("Node Register: Register Ethereum Node URL successfully, bootnode: {}",
                                  self.bootnode_client.remote_host());
                            Inner::idle()
                        }
                        Err(err) => {
                            warn!("Node Register: Failed to register Ethereum Node URL to {}, error: {:?}",
                                  self.bootnode_client.remote_host(), err);
                            Inner::idle()
                        }
                    }
                }
            }
        }
    }
}

enum Inner {
    Idle,
    FetchingUrl { fetcher: UrlFetcher },
    RegisteringUrl { register: UrlRegister },
}

impl Inner {
    fn idle() -> Self {
        Inner::Idle
    }

    fn fetch_url(web3: &Web3, ethereum_program: EthereumProgram) -> Self {
        let fetcher = match ethereum_program {
            EthereumProgram::Parity => Box::new(
                web3.parity()
                    .enode()
                    .from_err::<Error>()
                    .and_then(|enode_url| Ok(EthereumNodeUrl::from_str(&enode_url)?))
                    .from_err::<Error>(),
            ),
            EthereumProgram::GoEthereum => unimplemented!(),
        };

        Inner::FetchingUrl { fetcher }
    }

    fn register_url(
        bootnode_client: &BootnodeClient,
        network_name: &str,
        enode_url: EthereumNodeUrl,
        http_jsonrpc_endpoint: Option<Url>,
        ws_jsonrpc_endpoint: Option<Url>,
    ) -> Self {
        let mut futs: Vec<Box<Future<Item = bool, Error = Error> + Send>> = Vec::with_capacity(3);

        futs.push(Box::new(
            bootnode_client
                .add_enode_url(network_name, &enode_url)
                .from_err::<Error>(),
        ));

        if let Some(url) = http_jsonrpc_endpoint {
            futs.push(Box::new(
                bootnode_client
                    .add_http_jsonrpc_endpoint(network_name, &url)
                    .from_err::<Error>(),
            ));
        }

        if let Some(url) = ws_jsonrpc_endpoint {
            futs.push(Box::new(
                bootnode_client
                    .add_ws_jsonrpc_endpoint(network_name, &url)
                    .from_err::<Error>(),
            ));
        }

        let register = Box::new(
            futures::future::join_all(futs)
                .map(|_| true)
                .map_err(|_| Error::UnableToRegisterEthereumNodeInfo),
        );

        Inner::RegisteringUrl { register }
    }
}
