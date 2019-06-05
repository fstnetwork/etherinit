use parking_lot::Mutex;
use std::sync::Arc;
use url::Url;

use crate::primitives::EthereumNodeUrl;

use super::Tracker;

pub struct Service {
    tracker: Arc<Mutex<Tracker>>,
}

impl Service {
    pub fn with_tracker(tracker: Arc<Mutex<Tracker>>) -> Service {
        Service { tracker }
    }
}

impl Default for Service {
    fn default() -> Service {
        Service {
            tracker: Arc::new(Mutex::new(Tracker::new())),
        }
    }
}

impl_web! {
impl Service {
    #[get("/ethereum/networks")]
    #[content_type("application/json")]
    fn ethereum_networks(&self) -> Result<Vec<String>, ()> {
        Ok(self
            .tracker
            .lock()
            .ethereum()
            .iter()
            .fold(Vec::new(), |mut vec, (network, _value)| {
                vec.push(network.clone());
                vec
            }))
    }

    #[get("/ethereum/:network/system-info")]
    #[content_type("application/json")]
    fn ethereum_system_info(&self, network: String) -> Result<serde_json::Value, ()> {
        match self.tracker.lock().ethereum().get(&network) {
            Some(e) => Ok(serde_json::to_value(e.system_info())
                .expect("EthereumSystemInfo is serializable; qed")),
            None => Ok(serde_json::Value::default()),
        }
    }

    #[get("/ethereum/:network/chainspec")]
    #[content_type("application/json")]
    fn ethereum_chainspec(&self, network: String) -> Result<serde_json::Value, ()> {
        match self.tracker.lock().ethereum().get(&network) {
            Some(e) => Ok(e.chainspec_json()),
            None => Ok(serde_json::Value::default()),
        }
    }

    // #[get("/ethereum/:network/nodes")]
    // #[content_type("text/plain")]
    // fn ethereum_nodes_plain(&self, network: String) -> Result<String, ()> {
    //     let nodes = match self.tracker.lock().ethereum().get(&network) {
    //         Some(network) => network.nodes().map(ToString::to_string).collect(),
    //         None => vec![],
    //     };
    //     Ok(nodes.join("\n"))
    // }

    #[get("/ethereum/:network/nodes")]
    #[content_type("application/json")]
    fn ethereum_nodes_json(&self, network: String) -> Result<Vec<String>, ()> {
        let nodes = match self.tracker.lock().ethereum().get(&network) {
            Some(network) => network.nodes().map(ToString::to_string).collect(),
            None => vec![],
        };
        Ok(nodes)
    }

    #[get("/ethereum/:network/http-jsonrpc-endpoints")]
    #[content_type("application/json")]
    fn ethereum_nodes_http_endpoints(&self, network: String) -> Result<Vec<String>, ()> {
        let nodes = match self.tracker.lock().ethereum().get(&network) {
            Some(network) => network.http_jsonrpc_endpoints().map(ToString::to_string).collect(),
            None => vec![],
        };
        Ok(nodes)
    }

    #[get("/ethereum/:network/ws-jsonrpc-endpoints")]
    #[content_type("application/json")]
    fn ethereum_nodes_ws_endpoints(&self, network: String) -> Result<Vec<String>, ()> {
        let nodes = match self.tracker.lock().ethereum().get(&network) {
            Some(network) => network.ws_jsonrpc_endpoints().map(ToString::to_string).collect(),
            None => vec![],
        };
        Ok(nodes)
    }

    #[get("/ethereum/:network/miners")]
    #[content_type("application/json")]
    fn ethereum_miners(&self, network: String) -> Result<Vec<String>, ()> {
        match self.tracker.lock().ethereum().get(&network) {
            Some(network) => Ok(network
                .chainspec()
                .validators()
                .unwrap_or_else(||vec![])
                .iter()
                .map(|miner| format!("{:?}", miner))
                .collect()),
            None => Ok(vec![]),
        }
    }

    #[post("/ethereum/:network/nodes")]
    #[content_type("text/plain")]
    fn update_ethereum_node(&self, network: String, body: String) -> Result<String, ()> {
        use std::str::FromStr;
        match (
            self.tracker.lock().ethereum_mut().get_mut(&network),
            EthereumNodeUrl::from_str(body.as_str()),
        ) {
            (Some(e), Ok(url)) => {
                e.update_node(url.clone());
                Ok(url.to_string())
            }
            _ => {
                Ok(String::new())
            }
        }
    }

    #[post("/ethereum/:network/http-jsonrpc-endpoints")]
    #[content_type("text/plain")]
    fn update_http_jsonrpc_endpoints(&self, network: String, body: String) -> Result<String, ()> {
        use std::str::FromStr;
        match (
            self.tracker.lock().ethereum_mut().get_mut(&network),
            Url::from_str(body.as_str()),
        ) {
            (Some(e), Ok(url)) => {
                e.update_http_jsonrpc_endpoint(url.clone());
                Ok(url.to_string())
            }
            _ => {
                Ok(String::new())
            }
        }
    }

    #[post("/ethereum/:network/ws-jsonrpc-endpoints")]
    #[content_type("text/plain")]
    fn update_ws_jsonrpc_endpoints(&self, network: String, body: String) -> Result<String, ()> {
        use std::str::FromStr;
        match (
            self.tracker.lock().ethereum_mut().get_mut(&network),
            Url::from_str(body.as_str()),
        ) {
            (Some(e), Ok(url)) => {
                e.update_ws_jsonrpc_endpoint(url.clone());
                Ok(url.to_string())
            }
            _ => {
                Ok(String::new())
            }
        }
    }
}
}
