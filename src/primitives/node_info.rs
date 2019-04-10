use super::{EthereumNodeUrl, NodeRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    role: NodeRole,
    enode_url: EthereumNodeUrl,
}
