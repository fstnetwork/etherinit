use futures::{sync::mpsc, Async, Future, Poll, Stream};
use std::collections::HashSet;

use crate::primitives::{EthereumNodeUrl, EthereumProgram};

use super::{BootnodeClient, Error, Web3};

type PeerFetcher = Box<dyn Future<Item = Vec<EthereumNodeUrl>, Error = Error> + Send>;
type PeerImporter = Box<dyn Future<Item = usize, Error = ()> + Send>;

struct PeerCache {
    cache: HashSet<EthereumNodeUrl>,
    count: u32,
    refresh_limit: u32,
}

impl PeerCache {
    fn new() -> PeerCache {
        PeerCache {
            cache: Default::default(),
            count: 0,
            refresh_limit: 100,
        }
    }

    #[allow(unused)]
    fn clear(&mut self) {
        self.cache.clear();
        self.count = 0;
    }

    fn add(&mut self, peers: &[EthereumNodeUrl]) -> Vec<EthereumNodeUrl> {
        let new_peers: Vec<_> = if self.count < self.refresh_limit {
            self.count += 1;
            peers
                .iter()
                .cloned()
                .filter(|url| !self.cache.contains(url))
                .collect()
        } else {
            self.count = 0;
            self.cache.clear();
            peers.iter().cloned().collect()
        };

        for peer in &new_peers {
            self.cache.insert(peer.clone());
        }

        new_peers
    }
}

pub struct Importer {
    inner: Inner,

    event_receiver: mpsc::UnboundedReceiver<()>,
    event_sender: mpsc::UnboundedSender<()>,

    ethereum_program: EthereumProgram,
    network_name: String,
    web3: Web3,
    bootnode_client: BootnodeClient,

    peer_cache: PeerCache,
}

impl Importer {
    pub fn new(
        ethereum_program: EthereumProgram,
        network_name: String,
        web3: Web3,
        bootnode_client: BootnodeClient,
    ) -> Importer {
        let (event_sender, event_receiver) = mpsc::unbounded();

        Importer {
            inner: Inner::Idle,

            event_receiver,
            event_sender,

            ethereum_program,
            network_name,
            web3,
            bootnode_client,

            peer_cache: PeerCache::new(),
        }
    }

    pub fn import(&self) {
        self.event_sender
            .unbounded_send(())
            .expect("receiver always existed; qed");
    }
}

impl Future for Importer {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.inner = match self.inner {
                Inner::Idle => match self.event_receiver.poll() {
                    Ok(Async::NotReady) | Err(_) => return Ok(Async::NotReady),
                    Ok(Async::Ready(_)) => {
                        Inner::fetch_peers(&self.bootnode_client, &self.network_name)
                    }
                },
                Inner::FetchingPeers { ref mut fetcher } => match fetcher.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(peer_urls)) => {
                        // retrieve new peers
                        let new_peers = self.peer_cache.add(&peer_urls);

                        match new_peers.len() {
                            0 => {
                                info!("Node Importer: No new Ethereum node fetched");
                                Inner::idle()
                            }
                            n => {
                                info!("Node Importer: {} new Ethereum node(s) fetched", n);
                                Inner::import_peers(&self.web3, self.ethereum_program, &new_peers)
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Node Importer: Failed to fetch Ethereum Node URL from bootnode service: {:?}, error: {:?}",
                              self.bootnode_client.remote_host(), err);
                        Inner::idle()
                    }
                },
                Inner::ImportingPeers { ref mut importer } => match importer.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(n)) => {
                        info!("Node Importer: {} Ethereum node(s) imported", n);
                        Inner::idle()
                    }
                    Err(err) => {
                        warn!("Node Importer: Failed to import peers, error: {:?}", err);
                        Inner::idle()
                    }
                },
            }
        }
    }
}

enum Inner {
    Idle,
    FetchingPeers { fetcher: PeerFetcher },
    ImportingPeers { importer: PeerImporter },
}

impl Inner {
    fn idle() -> Self {
        Inner::Idle
    }

    fn fetch_peers(bootnode_client: &BootnodeClient, network_name: &String) -> Self {
        Inner::FetchingPeers {
            fetcher: Box::new(
                bootnode_client
                    .fetch_enodes(network_name)
                    .from_err::<Error>(),
            ),
        }
    }

    fn import_peers(
        web3: &Web3,
        ethereum_program: EthereumProgram,
        peers: &[EthereumNodeUrl],
    ) -> Self {
        let futures: Vec<_> = peers
            .iter()
            .map(|enode_url| {
                let enode_url = enode_url.clone();
                match ethereum_program {
                    EthereumProgram::Parity => web3
                        .parity_set()
                        .add_reserved_peer(&enode_url.to_string())
                        .map({
                            let url = enode_url.clone();
                            move |ok| {
                                if ok {
                                    info!("Node Importer: Add peer {:?} to Ethereum Node", url);
                                }
                                ok
                            }
                        })
                        .from_err::<Error>(),
                    EthereumProgram::GoEthereum => {
                        unimplemented!();
                    }
                }
            })
            .collect();

        let importer = Box::new(
            futures::future::join_all(futures)
                .map(|results| {
                    results.iter().fold(0, |n, ok| match ok {
                        true => n + 1,
                        false => n,
                    })
                })
                .map_err(|_| ()),
        );

        Inner::ImportingPeers { importer }
    }
}
