use crate::primitives::EthereumChainSpec;

pub fn generate_chainspec() -> i32 {
    let spec = match EthereumChainSpec::from_env() {
        Ok(spec) => spec,
        Err(err) => {
            eprintln!("{}", err);
            return -1;
        }
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&spec.as_json())
            .expect("serde_json::Value is serializable; qed")
    );
    0
}
