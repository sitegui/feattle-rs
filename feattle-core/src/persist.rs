use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;

pub trait Persist: Send + Sync + 'static {
    fn save_current(&self, value: &CurrentValues) -> Result<(), Box<dyn Error>>;
    fn load_current(&self) -> Result<Option<CurrentValues>, Box<dyn Error>>;

    fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Box<dyn Error>>;
    fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Box<dyn Error>>;
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValueHistory {
    pub entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub value: Value,
    pub value_overview: String,
    pub modified_at: DateTime<Utc>,
    pub modified_by: String,
}
