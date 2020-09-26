//! Define the interface with some external persistence logic
//!
//! This core module does not provide any concrete implementation for persisting the current and
//! historical values for the feature toggles. Instead, it defines this extension point that can be
//! used to create your own custom logic, however some implementors are available in the package
//! `feattle-sync`.

use crate::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Responsible for storing and loading data from a permanent storage.
///
/// # Async
/// The methods on this trait are async and can be implemented with the help of the `async_trait`
/// crate:
///
/// ```
/// use async_trait::async_trait;
/// use feattle_core::persist::*;
/// use feattle_core::Error;
///
/// struct MyPersistenceLogic;
///
/// #[async_trait]
/// impl Persist for MyPersistenceLogic {
///     async fn save_current(&self, value: &CurrentValues) -> Result<(), Error> {
///         unimplemented!()
///     }
///
///     async fn load_current(&self) -> Result<Option<CurrentValues>, Error> {
///         unimplemented!()
///     }
///
///     async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Error> {
///         unimplemented!()
///     }
///
///     async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Error> {
///         unimplemented!()
///     }
/// }
/// ```
///
/// # Errors
/// The methods can indicate failures by returning [`type@crate::Error`], read more about on its own
/// page.
#[async_trait]
pub trait Persist: Send + Sync + 'static {
    /// Save current state of all feattles.
    async fn save_current(&self, value: &CurrentValues) -> Result<(), Error>;

    /// Load the current state of all feattles. With no previous state existed, `Ok(None)` should be
    /// returned.
    async fn load_current(&self) -> Result<Option<CurrentValues>, Error>;

    /// Save the full history of a single feattle.
    async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Error>;

    /// Load the full history of a single feattle. With the feattle has no history, `Ok(None)`
    /// should be returned.
    async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Error>;
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
    pub feattles: BTreeMap<String, CurrentValue>,
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
