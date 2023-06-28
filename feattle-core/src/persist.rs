//! Define the interface with some external persistence logic
//!
//! This core module does not provide any concrete implementation for persisting the current and
//! historical values for the feattles. Instead, it defines this extension point that can be
//! used to create your own custom logic, however some implementors are available in the package
//! `feattle-sync`.

use crate::BoxError;
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
/// use feattle_core::BoxError;
/// use feattle_core::persist::*;
///
/// struct MyPersistenceLogic;
///
/// #[async_trait]
/// impl Persist for MyPersistenceLogic {
///     async fn save_current(&self, value: &CurrentValues) -> Result<(), BoxError> {
///         unimplemented!()
///     }
///
///     async fn load_current(&self) -> Result<Option<CurrentValues>, BoxError> {
///         unimplemented!()
///     }
///
///     async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), BoxError> {
///         unimplemented!()
///     }
///
///     async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, BoxError> {
///         unimplemented!()
///     }
/// }
/// ```
///
/// # Errors
/// The persistence layer can return an error, that will be bubbled up by other error
/// types, like [`super::UpdateError`] and [`super::HistoryError`].
#[async_trait]
pub trait Persist: Send + Sync {
    /// Save current state of all feattles.
    async fn save_current(&self, value: &CurrentValues) -> Result<(), BoxError>;

    /// Load the current state of all feattles. With no previous state existed, `Ok(None)` should be
    /// returned.
    async fn load_current(&self) -> Result<Option<CurrentValues>, BoxError>;

    /// Save the full history of a single feattle.
    async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), BoxError>;

    /// Load the full history of a single feattle. With the feattle has no history, `Ok(None)`
    /// should be returned.
    async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, BoxError>;
}

/// Store the current values of all feattles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrentValues {
    /// A monotonically increasing version, that can be used to detect race conditions
    pub version: i32,
    /// When this version was created
    pub date: DateTime<Utc>,
    /// Data for each feattle. Some feattles may not be present in this map, since they were never
    /// modified. Also, some extra feattles may be present in this map because they were used in a
    /// previous invocation of feattles.
    pub feattles: BTreeMap<String, CurrentValue>,
}

/// Store the current value of a single featttle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrentValue {
    /// When this modification was made
    pub modified_at: DateTime<Utc>,
    /// Who did that modification
    pub modified_by: String,
    /// The value, expressed in JSON
    pub value: Value,
}

/// Store the history of modification of a single feattle
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ValueHistory {
    /// The entries are not necessarily stored in any specific order
    pub entries: Vec<HistoryEntry>,
}

/// Store the value at a given point in time of a single feattle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// A mock implementation that does not store the information anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoPersistence;

#[async_trait]
impl Persist for NoPersistence {
    async fn save_current(&self, _value: &CurrentValues) -> Result<(), BoxError> {
        Ok(())
    }

    async fn load_current(&self) -> Result<Option<CurrentValues>, BoxError> {
        Ok(None)
    }

    async fn save_history(&self, _key: &str, _value: &ValueHistory) -> Result<(), BoxError> {
        Ok(())
    }

    async fn load_history(&self, _key: &str) -> Result<Option<ValueHistory>, BoxError> {
        Ok(None)
    }
}
