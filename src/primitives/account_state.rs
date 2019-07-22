use ethereum_types::U256;
use std::collections::BTreeMap;

use crate::utils;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<U256>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub constructor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<U256>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<BTreeMap<U256, U256>>,
}

impl AccountState {
    pub fn from_json_value(state: &serde_json::Value) -> AccountState {
        let constructor = state["constructor"].as_str().map(String::from);
        let code = state["code"].as_str().map(String::from);
        let balance = utils::maybe_u256_from_json_value(&state["balance"]);
        let nonce = utils::maybe_u256_from_json_value(&state["nonce"]);
        let version = utils::maybe_u256_from_json_value(&state["version"]);
        let storage = match state["storage"] {
            serde_json::Value::Object(ref map) => Some(map.iter().fold(
                BTreeMap::default(),
                |mut map, (key, value)| {
                    if let (Some(key), Some(value)) = (
                        utils::maybe_u256(key),
                        utils::maybe_u256_from_json_value(value),
                    ) {
                        map.insert(key, value);
                    }
                    map
                },
            )),
            _ => None,
        };

        AccountState {
            balance,
            constructor,
            nonce,
            version,
            code,
            storage,
        }
    }
}
