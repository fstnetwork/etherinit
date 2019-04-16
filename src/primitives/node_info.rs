use crate::primitives::{EthereumNodeUrl, NodeRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    role: NodeRole,
    enode_url: EthereumNodeUrl,
}
