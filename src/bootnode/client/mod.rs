use futures::{Future, Stream};
use hyper::{Body, Client as HyperClient, Request, Uri};
use serde_json::Value as JsonValue;
use std::str::FromStr;
use url::Url;

use crate::primitives::{EthereumNodeUrl, EthereumSystemInfo};

mod error;

pub use self::error::Error;

#[derive(Debug, Clone)]
pub struct Client {
    client: HyperClient<hyper::client::HttpConnector, Body>,
    bootnode_host: String,
    bootnode_port: u16,
    bootnode_authority: String,
}

impl Client {
    pub fn new(bootnode_host: String, bootnode_port: u16) -> Client {
        let bootnode_authority = format!("{}:{}", bootnode_host, bootnode_port);
        Client {
            client: HyperClient::builder().keep_alive(true).build_http(),
            bootnode_host,
            bootnode_port,
            bootnode_authority,
        }
    }

    pub fn remote_host(&self) -> String {
        format!("{}:{}", self.bootnode_host, self.bootnode_port)
    }

    pub fn post_plain<F, Out>(
        &self,
        path_and_query: &str,
        body: String,
        extract: F,
    ) -> impl Future<Item = Out, Error = Error>
    where
        F: Fn(hyper::Chunk) -> Result<Out, Error>,
    {
        let uri = Uri::builder()
            .scheme("http")
            .authority(self.bootnode_authority.as_str())
            .path_and_query(path_and_query)
            .build()
            .expect("uri builder");

        let req = Request::builder()
            .uri(&uri)
            .method("POST")
            .header("Content-Type", "text/plain")
            .body(Body::from(body))
            .expect("request builder");

        self.client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .from_err::<Error>()
            .and_then(extract)
            .from_err()
    }

    pub fn get_json<F, Out>(
        &self,
        path_and_query: &str,
        extract: F,
    ) -> impl Future<Item = Out, Error = Error>
    where
        F: Fn(hyper::Chunk) -> Result<Out, Error>,
    {
        let uri = Uri::builder()
            .scheme("http")
            .authority(self.bootnode_authority.as_str())
            .path_and_query(path_and_query)
            .build()
            .expect("uri builder");

        let req = Request::builder()
            .uri(&uri)
            .method("GET")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .expect("request builder");

        self.client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .from_err::<Error>()
            .and_then(extract)
            .from_err()
    }

    pub fn add_enode_url(
        &self,
        network_name: &str,
        url: &EthereumNodeUrl,
    ) -> impl Future<Item = bool, Error = Error> {
        self.post_plain(
            &format!("/ethereum/{}/nodes", network_name),
            url.to_string(),
            |_| Ok(true),
        )
    }

    pub fn add_http_jsonrpc_endpoint(
        &self,
        network_name: &str,
        url: &Url,
    ) -> impl Future<Item = bool, Error = Error> {
        self.post_plain(
            &format!("/ethereum/{}/http-jsonrpc-endpoints", network_name),
            url.to_string(),
            |_| Ok(true),
        )
    }

    pub fn add_ws_jsonrpc_endpoint(
        &self,
        network_name: &str,
        url: &Url,
    ) -> impl Future<Item = bool, Error = Error> {
        self.post_plain(
            &format!("/ethereum/{}/ws-jsonrpc-endpoints", network_name),
            url.to_string(),
            |_| Ok(true),
        )
    }

    pub fn fetch_system_info(
        &self,
        network_name: &str,
    ) -> impl Future<Item = EthereumSystemInfo, Error = Error> {
        self.get_json(&format!("/ethereum/{}/system-info", network_name), |data| {
            let system_info: EthereumSystemInfo =
                serde_json::from_slice(&data).expect("EthereumSystemInfo is deserializable; qed");
            Ok(system_info)
        })
    }

    pub fn fetch_chainspec(
        &self,
        network_name: &str,
    ) -> impl Future<Item = JsonValue, Error = Error> {
        self.get_json(&format!("/ethereum/{}/chainspec", network_name), |data| {
            match serde_json::from_slice(&data) {
                Ok(value) => Ok(value),
                _ => Ok(JsonValue::default()),
            }
        })
    }

    pub fn fetch_enodes(
        &self,
        network_name: &str,
    ) -> impl Future<Item = Vec<EthereumNodeUrl>, Error = Error> {
        self.get_json(&format!("/ethereum/{}/nodes", network_name), |data| {
            match serde_json::from_slice(&data) {
                Ok(JsonValue::Array(arr)) => {
                    let vec = Vec::with_capacity(arr.len());
                    Ok(arr.iter().fold(vec, |mut vec, value| {
                        if let JsonValue::String(url) = value {
                            if let Ok(enode_url) = EthereumNodeUrl::from_str(&url) {
                                vec.push(enode_url);
                            }
                        };
                        vec
                    }))
                }
                _ => Ok(vec![]),
            }
        })
    }

    pub fn fetch_rpc_endpoints(
        &self,
        network_name: &str,
        endpoint_path: &str,
    ) -> impl Future<Item = Vec<Url>, Error = Error> {
        self.get_json(
            &format!("/ethereum/{}/{}", network_name, endpoint_path),
            |data| match serde_json::from_slice(&data) {
                Ok(JsonValue::Array(arr)) => {
                    let vec = Vec::with_capacity(arr.len());
                    Ok(arr.iter().fold(vec, |mut vec, value| {
                        if let JsonValue::String(url) = value {
                            if let Ok(url) = Url::from_str(&url) {
                                vec.push(url);
                            }
                        };
                        vec
                    }))
                }
                _ => Ok(vec![]),
            },
        )
    }

    pub fn fetch_http_rpc_endpoints(
        &self,
        network_name: &str,
    ) -> impl Future<Item = Vec<Url>, Error = Error> {
        self.fetch_rpc_endpoints(network_name, "http-jsonrpc-endpoints")
    }

    pub fn fetch_ws_jsonrpc_endpoints(
        &self,
        network_name: &str,
    ) -> impl Future<Item = Vec<Url>, Error = Error> {
        self.fetch_rpc_endpoints(network_name, "ws-jsonrpc-endpoints")
    }
}
