use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentValues {
    pub version: i32,
    pub date: DateTime<Utc>,
    pub features: BTreeMap<String, CurrentValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentValue {
    pub modified_at: DateTime<Utc>,
    pub modified_by: String,
    pub value: Value,
}
