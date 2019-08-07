use ethereum_types::U256;
use std::str::FromStr;

pub mod env_var;
pub mod exit_code;
mod retry_future;

pub use self::retry_future::RetryFuture;

pub fn clean_0x(s: &str) -> &str {
    if s.starts_with("0x") {
        &s[2..]
    } else {
        s
    }
}

pub fn to_0xhex<V: std::fmt::LowerHex>(value: &V) -> String {
    format!("0x{:x}", value)
}

pub fn maybe_u256(value: &str) -> Option<U256> {
    if value.starts_with("0x") {
        U256::from_str(&value[2..]).ok()
    } else {
        U256::from_dec_str(value).ok()
    }
}

pub fn maybe_u256_from_json_value(value: &serde_json::Value) -> Option<U256> {
    match value {
        serde_json::Value::String(value) => maybe_u256(value),
        serde_json::Value::Number(n) => n.as_u64().map(U256::from),
        _ => None,
    }
}
