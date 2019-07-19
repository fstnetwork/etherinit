use ethereum_types::U256;
use std::collections::BTreeMap;
use std::str::FromStr;

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
        let balance = try_decode_number(&state["balance"]);
        let nonce = try_decode_number(&state["nonce"]);
        let version = try_decode_number(&state["version"]);
        let storage = match state["storage"] {
            serde_json::Value::Object(ref map) => Some(map.iter().fold(
                BTreeMap::default(),
                |mut map, (key, value)| {
                    let key = match if key.starts_with("0x") {
                        U256::from_str(&key[2..]).ok()
                    } else {
                        U256::from_dec_str(key).ok()
                    } {
                        Some(key) => key,
                        None => return map,
                    };
                    let value = match try_decode_number(value) {
                        Some(v) => v,
                        None => return map,
                    };

                    map.insert(key, value);
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

fn try_decode_number(value: &serde_json::Value) -> Option<U256> {
    match value {
        serde_json::Value::String(value) => match value {
            value if value.starts_with("0x") => U256::from_str(&value[2..]).ok(),
            value => U256::from_dec_str(value).ok(),
        },
        serde_json::Value::Number(n) => n.as_u64().map(U256::from),
        _ => None,
    }
}
