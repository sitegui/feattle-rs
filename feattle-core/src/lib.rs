//! This crate is the core implementation of the feature flags (called "feattles", for short).
//!
//! Its main parts are the macro [`feattles!`] together with the trait [`Feattles`]. Please refer to
//! the [main package - `feattle`](https://crates.io/crates/feattle) for more information.
//!
//! # Usage example
//! ```
//! use feattle_core::{feattles, Feattles};
//! use feattle_core::persist::NoPersistence;
//!
//! // Declare the struct
//! feattles! {
//!     struct MyFeattles {
//!         /// Is this usage considered cool?
//!         is_cool: bool = true,
//!         /// Limit the number of "blings" available.
//!         /// This will not change the number of "blengs", though!
//!         max_blings: i32,
//!         /// List the actions that should not be available
//!         blocked_actions: Vec<String>,
//!     }
//! }
//!
//! // Create a new instance (`NoPersistence` is just a mock for the persistence layer)
//! let my_feattles = MyFeattles::new(NoPersistence);
//!
//! // Read values (note the use of `*`)
//! assert_eq!(*my_feattles.is_cool(), true);
//! assert_eq!(*my_feattles.max_blings(), 0);
//! assert_eq!(*my_feattles.blocked_actions(), Vec::<String>::new());
//! ```
//!
//! # How it works
//!
//! The macro will generate a struct with the given name and visibility modifier (assuming private
//! by default). The generated struct implements [`Feattles`] and also exposes one method for each
//! feattle.
//!
//! The methods created for each feattle allow reading their current value. For example, for a
//! feattle `is_cool: bool`, there will be a method like
//! `pub fn is_cool(&self) -> MappedRwLockReadGuard<bool>`. Note the use of
//! [`parking_lot::MappedRwLockReadGuard`] because the interior of the struct is stored behind a `RwLock` to
//! control concurrent access.
//!
//! A feattle is created with the syntax `$key: $type [= $default]`. You can use doc coments (
//! starting with `///`) to describe nicely what they do in your system. You can use any type that
//! implements [`FeattleValue`] and optionally provide a default. If not provided, the default
//! will be created with `Default::default()`.
//!
//! # Updating values
//! This crate only disposes of low-level methods to load current feattles with [`Feattles::reload()`]
//! and update their values with [`Feattles::update()`]. Please look for the crates
//! [feattle-sync](https://crates.io/crates/feattle-sync) and
//! [feattle-ui](https://crates.io/crates/feattle-ui) for higher-level functionalities.
//!
//! # Limitations
//! Due to some restrictions on how the macro is written, you can only use [`feattles!`] once per
//! module. For example, the following does not compile:
//!
//! ```compile_fail
//! use feattle_core::feattles;
//!
//! feattles! { struct A { } }
//! feattles! { struct B { } }
//! ```
//!
//! You can work around this limitation by creating a sub-module and then re-exporting the generated
//! struct. Note the use of `pub struct` in the second case.
//! ```
//! use feattle_core::feattles;
//!
//! feattles! { struct A { } }
//!
//! mod b {
//!     use feattle_core::feattles;
//!     feattles! { pub struct B { } }
//! }
//!
//! use b::B;
//! ```
//!
//! # Optional features
//!
//! - **uuid**: will add support for [`uuid::Uuid`].

#[doc(hidden)]
pub mod __internal;
mod definition;
mod feattle_value;
pub mod json_reading;
pub mod last_reload;
/// This module only contains exported macros, that are documented at the root level.
#[doc(hidden)]
pub mod macros;
pub mod persist;

use crate::__internal::{FeattlesStruct, InnerFeattles};
use crate::json_reading::FromJsonError;
use crate::last_reload::LastReload;
use async_trait::async_trait;
use chrono::Utc;
pub use definition::*;
pub use feattle_value::*;
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard, RwLockWriteGuard};
use persist::*;
use serde_json::Value;
use std::error::Error;
use std::fmt::Debug;
use thiserror::Error;

/// The error type returned by [`Feattles::update()`]
#[derive(Error, Debug)]
pub enum UpdateError<PersistError: Error + Send + Sync + 'static> {
    /// Cannot update because current values were never successfully loaded from the persist layer
    #[error("cannot update because current values were never successfully loaded from the persist layer")]
    NeverReloaded,
    /// The key is unknown
    #[error("the key {0} is unknown")]
    UnknownKey(String),
    /// Failed to parse the value from JSON
    #[error("failed to parse the value from JSON")]
    Parsing(
        #[source]
        #[from]
        FromJsonError,
    ),
    /// Failed to persist new state
    #[error("failed to persist new state")]
    Persistence(#[source] PersistError),
}

/// The error type returned by [`Feattles::history()`]
#[derive(Error, Debug)]
pub enum HistoryError<PersistError: Error + Send + Sync + 'static> {
    /// The key is unknown
    #[error("the key {0} is unknown")]
    UnknownKey(String),
    /// Failed to load persisted state
    #[error("failed to load persisted state")]
    Persistence(#[source] PersistError),
}

/// The main trait of this crate.
///
/// The struct created with [`feattles!`] will implement this trait in addition to a method for each
/// feattle. Read more at the [crate documentation](crate).
#[async_trait]
pub trait Feattles<P>: FeattlesPrivate<P> {
    /// Create a new feattles instance, using the given persistence layer logic.
    ///
    /// All feattles will start with their default values. You can force an initial synchronization
    /// with [`Feattles::update`].
    fn new(persistence: P) -> Self;

    /// Return a shared reference to the persistence layer.
    fn persistence(&self) -> &P;

    /// The list of all available keys.
    fn keys(&self) -> &'static [&'static str];

    /// Describe one specific feattle, returning `None` if the feattle with the given name does not
    /// exist.
    fn definition(&self, key: &str) -> Option<FeattleDefinition>;

    /// Return details of the last time the data was synchronized by calling [`Feattles::reload()`].
    fn last_reload(&self) -> LastReload {
        self._read().last_reload
    }

    /// Return a reference to the last synchronized data. The reference is behind a
    /// read-write lock and will block any update until it is dropped. `None` is returned if a
    /// successful synchronization have never happened.
    fn current_values(&self) -> Option<MappedRwLockReadGuard<CurrentValues>> {
        let inner = self._read();
        match inner.current_values.as_ref() {
            None => None,
            Some(_) => Some(RwLockReadGuard::map(inner, |x| {
                x.current_values.as_ref().unwrap()
            })),
        }
    }

    /// Reload the current feattles' data from the persistence layer, propagating any errors
    /// produced by it.
    ///
    /// If any of the feattle values fail to be parsed from previously persisted values, their
    /// updates will be skipped. Other feattles that parsed successfully will still be updated.
    /// In this case, a [`log::error!`] will be generated for each time it occurs.
    async fn reload(&self) -> Result<(), P::Error>
    where
        P: Persist + Sync + 'static,
    {
        let current_values = self.persistence().load_current().await?;
        let mut inner = self._write();
        let now = Utc::now();
        match current_values {
            None => {
                inner.last_reload = LastReload::NoData { reload_date: now };
                let empty = CurrentValues {
                    version: 0,
                    date: now,
                    feattles: Default::default(),
                };
                inner.current_values = Some(empty);
            }
            Some(current_values) => {
                inner.last_reload = LastReload::Data {
                    reload_date: now,
                    version: current_values.version,
                    version_date: current_values.date,
                };
                for &key in self.keys() {
                    let value = current_values.feattles.get(key).cloned();
                    log::debug!("Will update {} with {:?}", key, value);
                    if let Err(error) = inner.feattles_struct.try_update(key, value) {
                        log::error!("Failed to update {}: {:?}", key, error);
                    }
                }
                inner.current_values = Some(current_values);
            }
        }
        Ok(())
    }

    /// Update a single feattle, passing the new value (in JSON representation) and the user that
    /// is associated with this change. The change will be persisted directly.
    ///
    /// While the update is happening, the new value will already be observable from other
    /// execution tasks or threads. However, if the update fails, the change will be rolled back.
    ///
    /// # Consistency
    ///
    /// To avoid operating on stale data, before doing an update the caller should usually call
    /// [`Feattles::reload()`] to ensure data is current.
    async fn update(
        &self,
        key: &str,
        value: Value,
        modified_by: String,
    ) -> Result<(), UpdateError<P::Error>>
    where
        P: Persist + Sync + 'static,
    {
        use UpdateError::*;

        // The update operation is made of 4 steps, each of which may fail:
        // 1. parse and update the inner generic struct
        // 2. persist the new history entry
        // 3. persist the new current values
        // 4. update the copy of the current values
        // If any step fails, the others will be rolled back

        // Assert the key exists
        if !self.keys().contains(&key) {
            return Err(UnknownKey(key.to_owned()));
        }

        let new_value = CurrentValue {
            modified_at: Utc::now(),
            modified_by,
            value,
        };

        let (new_values, old_value) = {
            let mut inner = self._write();

            // Check error condition for step 4 and prepare the new instance
            let mut new_values = inner.current_values.clone().ok_or(NeverReloaded)?;
            new_values
                .feattles
                .insert(key.to_owned(), new_value.clone());
            new_values.version += 1;

            // Step 1
            let old_value = inner
                .feattles_struct
                .try_update(key, Some(new_value.clone()))?;

            (new_values, old_value)
        };

        log::debug!("new_values = {:?}", new_values);

        let rollback_step_1 = || {
            // Note that if the old value was failing to parse, then the update will be final.
            let _ = self
                ._write()
                .feattles_struct
                .try_update(key, old_value.clone());
        };

        // Step 2: load + modify + save history
        let persistence = self.persistence();
        let old_history = persistence
            .load_history(key)
            .await
            .map_err(|err| {
                rollback_step_1();
                Persistence(err)
            })?
            .unwrap_or_default();

        // Prepare updated history
        let new_definition = self
            .definition(key)
            .expect("the key is guaranteed to exist");
        let mut new_history = old_history.clone();
        new_history.entries.push(HistoryEntry {
            value: new_value.value.clone(),
            value_overview: new_definition.value_overview,
            modified_at: new_value.modified_at,
            modified_by: new_value.modified_by.clone(),
        });

        persistence
            .save_history(key, &new_history)
            .await
            .map_err(|err| {
                rollback_step_1();
                Persistence(err)
            })?;

        // Step 3
        if let Err(err) = persistence.save_current(&new_values).await {
            rollback_step_1();
            if let Err(err) = self.persistence().save_history(key, &old_history).await {
                log::warn!("Failed to rollback history for {}: {:?}", key, err);
            }
            return Err(Persistence(err));
        }

        // Step 4
        self._write().current_values = Some(new_values);

        Ok(())
    }

    /// Return the definition for all the feattles.
    fn definitions(&self) -> Vec<FeattleDefinition> {
        self.keys()
            .iter()
            .map(|&key| {
                self.definition(key)
                    .expect("since we iterate over the list of known keys, this should always work")
            })
            .collect()
    }

    /// Return the history for a single feattle. It can be potentially empty (not entries).
    async fn history(&self, key: &str) -> Result<ValueHistory, HistoryError<P::Error>>
    where
        P: Persist + Sync + 'static,
    {
        // Assert the key exists
        if !self.keys().contains(&key) {
            return Err(HistoryError::UnknownKey(key.to_owned()));
        }

        let history = self
            .persistence()
            .load_history(key)
            .await
            .map_err(HistoryError::Persistence)?;

        Ok(history.unwrap_or_default())
    }
}

/// This struct is `pub` because the macro must have access to it, but should be otherwise invisible
/// to the users of this crate.
#[doc(hidden)]
pub trait FeattlesPrivate<P> {
    type FeattleStruct: FeattlesStruct;
    fn _read(&self) -> RwLockReadGuard<InnerFeattles<Self::FeattleStruct>>;
    fn _write(&self) -> RwLockWriteGuard<InnerFeattles<Self::FeattleStruct>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::Mutex;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[derive(Debug, thiserror::Error)]
    #[error("Some error")]
    struct SomeError;

    #[derive(Default, Clone)]
    struct MockPersistence(Arc<Mutex<MockPersistenceInner>>);

    #[derive(Default)]
    struct MockPersistenceInner {
        current: Option<CurrentValues>,
        history: BTreeMap<String, ValueHistory>,
        next_error: Option<SomeError>,
    }

    impl MockPersistence {
        fn put_error(&self) {
            let previous = self.0.lock().next_error.replace(SomeError);
            assert!(previous.is_none());
        }

        fn get_error(&self) -> Result<(), SomeError> {
            match self.0.lock().next_error.take() {
                None => Ok(()),
                Some(e) => Err(e),
            }
        }

        fn unwrap_current(&self) -> CurrentValues {
            self.0.lock().current.clone().unwrap()
        }

        fn unwrap_history(&self, key: &str) -> ValueHistory {
            self.0.lock().history.get(key).cloned().unwrap()
        }
    }

    #[async_trait]
    impl Persist for MockPersistence {
        type Error = SomeError;

        async fn save_current(&self, value: &CurrentValues) -> Result<(), Self::Error> {
            self.get_error().map(|_| {
                self.0.lock().current = Some(value.clone());
            })
        }

        async fn load_current(&self) -> Result<Option<CurrentValues>, Self::Error> {
            self.get_error().map(|_| self.0.lock().current.clone())
        }

        async fn save_history(&self, key: &str, value: &ValueHistory) -> Result<(), Self::Error> {
            self.get_error().map(|_| {
                self.0.lock().history.insert(key.to_owned(), value.clone());
            })
        }

        async fn load_history(&self, key: &str) -> Result<Option<ValueHistory>, Self::Error> {
            self.get_error()
                .map(|_| self.0.lock().history.get(key).cloned())
        }
    }

    #[tokio::test]
    async fn test() {
        feattles! {
            struct Config {
                /// A
                a: i32,
                b: i32 = 17
            }
        }

        let persistence = MockPersistence::default();
        let config = Config::new(persistence.clone());

        // Initial state
        assert_eq!(*config.a(), 0);
        assert_eq!(*config.b(), 17);
        assert!(Arc::ptr_eq(&config.persistence().0, &persistence.0));
        assert_eq!(config.keys(), &["a", "b"]);
        assert!(config.last_reload() == LastReload::Never);
        assert!(config.current_values().is_none());

        // Load from empty storage
        config.reload().await.unwrap();
        assert_eq!(*config.a(), 0);
        assert_eq!(*config.b(), 17);
        let last_reload = config.last_reload();
        assert!(matches!(last_reload, LastReload::NoData { .. }));
        assert!(config.current_values().is_some());

        // Load from failing storage
        persistence.put_error();
        config.reload().await.unwrap_err();
        assert_eq!(config.last_reload(), last_reload);

        // Update value
        config
            .update("a", json!(27i32), "somebody".to_owned())
            .await
            .unwrap();
        assert_eq!(*config.a(), 27);
        let values = persistence.unwrap_current();
        assert_eq!(values.version, 1);
        let value = values.feattles.get("a").unwrap();
        assert_eq!(value.modified_by, "somebody");
        assert_eq!(value.value, json!(27i32));
        let history = persistence.unwrap_history("a");
        assert_eq!(history.entries.len(), 1);
        assert_eq!(&history.entries[0].value, &json!(27i32));
        assert_eq!(&history.entries[0].value_overview, "27");
        assert_eq!(&history.entries[0].modified_by, "somebody");

        // Failed to update
        persistence.put_error();
        config
            .update("a", json!(207i32), "somebody else".to_owned())
            .await
            .unwrap_err();
        assert_eq!(*config.a(), 27);
        let values = persistence.unwrap_current();
        assert_eq!(values.version, 1);
        let value = values.feattles.get("a").unwrap();
        assert_eq!(value.modified_by, "somebody");
        assert_eq!(value.value, json!(27i32));
        let history = persistence.unwrap_history("a");
        assert_eq!(history.entries.len(), 1);
        assert_eq!(&history.entries[0].value, &json!(27i32));
        assert_eq!(&history.entries[0].value_overview, "27");
        assert_eq!(&history.entries[0].modified_by, "somebody");
    }
}
