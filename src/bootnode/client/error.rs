error_chain! {
    foreign_links {
        AddrParse(std::net::AddrParseError);
        Hyper(hyper::Error);
        Timer(tokio_timer::Error);
        Json(serde_json::Error);
        Web3(web3::Error);
        EthereumNodeUrl(super::primitives::EthereumNodeUrlError);
    }

    errors {
        InvalidStateTransfer(current_state: String, expected_state: String) {
            description("Invalid state transfer")
            display("Invalid state transfer, current: {}, expected: {}", current_state, expected_state)
        }
    }
}
