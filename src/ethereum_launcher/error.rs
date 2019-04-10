error_chain! {
    foreign_links {
        StdIo(std::io::Error);
        SerdeJson(serde_json::Error);
        Primitives(super::primitives::Error);
        EthSign(ethsign::Error);
    }

    errors {

    }
}
