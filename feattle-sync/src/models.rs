use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentValues {
    version: i32,
    features: BTreeMap<String, Value>,
}
