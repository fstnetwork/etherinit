use futures::{Async, Future, Poll, Stream};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tokio::timer::Interval;
use url::Url;

use crate::primitives::{EthereumChainSpec, EthereumNodeUrl, EthereumSystemInfo};

const MINIMUM_NODE_LIFETIME: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct EthereumNetwork {
    spec: EthereumChainSpec,
    spec_json: JsonValue,
    nodes: HashMap<EthereumNodeUrl, Instant>,
    http_jsonrpc_endpoints: HashSet<Url>,
    ws_jsonrpc_endpoints: HashSet<Url>,
    node_lifetime: Duration,
}

impl EthereumNetwork {
    pub fn new(spec: EthereumChainSpec, node_lifetime: Duration) -> EthereumNetwork {
        let spec_json = spec.as_json();
        EthereumNetwork {
            spec,
            spec_json,
            nodes: Default::default(),
            http_jsonrpc_endpoints: Default::default(),
            ws_jsonrpc_endpoints: Default::default(),
            node_lifetime,
        }
    }

    #[inline]
    #[allow(unused)]
    pub fn set_chainspec(&mut self, chainspec: EthereumChainSpec) {
        self.spec_json = chainspec.as_json();
        self.spec = chainspec;
    }

    #[inline]
    #[allow(unused)]
    pub fn set_node_lifetime(&mut self, node_lifetime: Duration) {
        self.node_lifetime = node_lifetime;
    }

    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = &EthereumNodeUrl> {
        self.nodes.keys()
    }

    #[inline]
    pub fn http_jsonrpc_endpoints(&self) -> impl Iterator<Item = &Url> {
        self.http_jsonrpc_endpoints.iter()
    }

    #[inline]
    pub fn ws_jsonrpc_endpoints(&self) -> impl Iterator<Item = &Url> {
        self.ws_jsonrpc_endpoints.iter()
    }

    pub fn drain_outdated_nodes(&mut self) {
        let node_lifetime = if self.node_lifetime < MINIMUM_NODE_LIFETIME {
            MINIMUM_NODE_LIFETIME
        } else {
            self.node_lifetime
        };

        let now = Instant::now();
        let outdated_nodes: HashSet<_> = self
            .nodes
            .iter()
            .filter(|(_, last_seen)| now - **last_seen > node_lifetime)
            .map(|(url, _)| url.clone())
            .collect();

        if !outdated_nodes.is_empty() {
            info!("Remove outdated nodes: {:?}", outdated_nodes);
            self.nodes.retain(|url, _| !outdated_nodes.contains(url));
        }
    }

    #[inline]
    pub fn chainspec(&self) -> EthereumChainSpec {
        self.spec.clone()
    }

    #[inline]
    pub fn chainspec_json(&self) -> JsonValue {
        self.spec_json.clone()
    }

    #[inline]
    pub fn system_info(&self) -> EthereumSystemInfo {
        let consensus_engine = self.spec.consensus_engine.clone();
        EthereumSystemInfo {
            node_count: self.nodes.len(),
            miner_count: consensus_engine.validator_count(),
            consensus_engine,
        }
    }

    #[inline]
    pub fn update_node(&mut self, enode_url: EthereumNodeUrl) {
        self.nodes.insert(enode_url, Instant::now());
    }

    #[inline]
    pub fn update_http_jsonrpc_endpoint(&mut self, endpoint: Url) {
        self.http_jsonrpc_endpoints.insert(endpoint);
    }

    #[inline]
    pub fn update_ws_jsonrpc_endpoint(&mut self, endpoint: Url) {
        self.ws_jsonrpc_endpoints.insert(endpoint);
    }
}

#[derive(Debug)]
pub struct Tracker {
    ethereum: HashMap<String, EthereumNetwork>,
    ticker: Interval,
}

impl Default for Tracker {
    fn default() -> Tracker {
        Tracker {
            ethereum: HashMap::new(),
            ticker: Interval::new_interval(MINIMUM_NODE_LIFETIME / 2),
        }
    }
}

impl Tracker {
    pub fn new() -> Tracker {
        Tracker::default()
    }

    #[allow(dead_code)]
    pub fn with_chainspecs(specs: &[EthereumChainSpec], node_lifetime: Duration) -> Tracker {
        let mut tracker = Tracker::new();
        specs
            .iter()
            .cloned()
            .fold(&mut tracker.ethereum, |e, spec| {
                e.insert(spec.name.clone(), EthereumNetwork::new(spec, node_lifetime));
                e
            });

        tracker
    }

    pub fn ethereum(&self) -> &HashMap<String, EthereumNetwork> {
        &self.ethereum
    }

    pub fn ethereum_mut(&mut self) -> &mut HashMap<String, EthereumNetwork> {
        &mut self.ethereum
    }

    pub fn add_ethereum_network(&mut self, network: EthereumNetwork) {
        let network_name = network.chainspec().name.clone();
        self.ethereum.insert(network_name, network);
    }

    #[allow(unused)]
    pub fn remove_ethereum_network(&mut self, network_name: &str) {
        self.ethereum.remove(network_name);
    }
}

impl Future for Tracker {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.ticker.poll() {
                Ok(Async::Ready(Some(_))) => {
                    self.ethereum_mut()
                        .iter_mut()
                        .for_each(|(network_name, ethereum)| {
                            debug!(
                                "Ethereum network: {}, nodes: {:?}",
                                network_name,
                                ethereum.nodes().collect::<Vec<_>>()
                            );
                            ethereum.drain_outdated_nodes();
                        });
                }
                _ => {
                    return Ok(Async::NotReady);
                }
            }
        }
    }
}
