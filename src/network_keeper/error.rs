error_chain! {
    foreign_links {
        Web3(web3::Error);
        BootnodeClient(super::bootnode::BootnodeClientError);
        EthereumNodeUrl(super::primitives::EthereumNodeUrlError);
    }
}
