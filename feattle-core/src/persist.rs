//! Define the interface with some external persistence logic
//!
//! This core module does not provide any concrete implementation for persisting the current and
//! historical values for the feature toggles. Instead, it defines this extension point that can be
//! used to create your own custom logic, however some implementors are available in the package
//! `feattle-sync`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;

/// Responsible for storing and loading data from a permanent storage.
pub trait Persist: Send + Sync + 'static {
    fn save_current(&self, value: &CurrentValues) -> Result<(), Box<dyn Error>>;
    fn load_current(&self) -> Result<Option<CurrentValues>, Box<dyn Error>>;

    fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Box<dyn Error>>;
    fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Box<dyn Error>>;
}

/// Store the current values of all feature toggles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentValues {
    /// A monotonically increasing version, that can be used to detect race conditions
    pub version: i32,
    /// When this version was created
    pub date: DateTime<Utc>,
    /// Data for each feature. Some features may not be present in this map, since they were never
    /// modified. Also, some extra features may be present in this map because they were used in a
    /// previous invocation of feattles.
    pub features: BTreeMap<String, CurrentValue>,
}

/// Store the current value of a single feature toggle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentValue {
    /// When this modification was made
    pub modified_at: DateTime<Utc>,
    /// Who did that modification
    pub modified_by: String,
    /// The value, expressed in JSON
    pub value: Value,
}

/// Store the history of modification of a single feature toggle
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValueHistory {
    /// The entries are not necessarily stored in any specific order
    pub entries: Vec<HistoryEntry>,
}

/// Store the value at a given point in time of a single feature toggle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The value, expressed in JSON
    pub value: Value,
    /// A human-readable description of the value
    pub value_overview: String,
    /// When this modification was made
    pub modified_at: DateTime<Utc>,
    /// Who did that modification
    pub modified_by: String,
}
