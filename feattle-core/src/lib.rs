//! This crate is the core implementation of the feature flags (called "feattles", for short).
//!
//! Its main parts are the macro [`feattles!`] together with the trait [`Feattles`].
//!
//! # Usage example
//! ```
//! use feattle_core::{feattles, Feattles};
//! use feattle_core::persist::NoPersistence;
//!
//! /// Declare the struct
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
//! /// Create a new instance (`NoPersistence` is just a mock for the persistence layer)
//! let my_feattles = MyFeattles::new(NoPersistence);
//!
//! /// Read values (note the use of `*`)
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
//! The feature `"uuid"` will add support for [`uuid::Uuid`].

#[doc(hidden)]
pub mod __internal;
mod definition;
mod feattle_value;
pub mod json_reading;
/// This module only contains exported macros, that are documented at the root level.
#[doc(hidden)]
pub mod macros;
pub mod persist;

use crate::__internal::{FeattlesStruct, InnerFeattles};
use crate::json_reading::FromJsonError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
pub use definition::*;
pub use feattle_value::*;
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard, RwLockWriteGuard};
use persist::*;
use serde_json::Value;
use std::error::Error;
use thiserror::Error;

/// The error type returned by [`Feattles::update()`]
#[derive(Error, Debug)]
pub enum UpdateError<PersistError: Error + Send + Sync + 'static> {
    #[error("cannot update because current values were never successfully loaded from the persist layer")]
    NeverReloaded,
    #[error("the key {0} is unknown")]
    UnknownKey(String),
    #[error(transparent)]
    FailedParsing(#[from] FromJsonError),
    #[error("failed to persist new state")]
    FailedPersistence(#[source] PersistError),
}

/// The error type returned by [`Feattles::history()`]
#[derive(Error, Debug)]
pub enum HistoryError<PersistError: Error + Send + Sync + 'static> {
    #[error("the key {0} is unknown")]
    UnknownKey(String),
    #[error("failed to load persisted state")]
    FailedPersistence(#[source] PersistError),
}

/// The main trait of this crate.
///
/// The struct created with [`feattles!`] will implement this trait in addition to a method for each
/// feattle (see its documentation for details).
#[async_trait]
pub trait Feattles<P: Persist>: FeattlesPrivate<P> + Send + Sync + 'static {
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

    /// The date when the feattle values were last updated from the persistence layer, if ever.
    fn last_reload(&self) -> Option<DateTime<Utc>> {
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
    async fn reload(&self) -> Result<(), P::Error> {
        let current_values = self.persistence().load_current().await?;
        let mut inner = self._write();
        let now = Utc::now();
        inner.last_reload = Some(now);
        match current_values {
            None => {
                let empty = CurrentValues {
                    version: 0,
                    date: now,
                    feattles: Default::default(),
                };
                inner.current_values = Some(empty);
            }
            Some(mut current_values) => {
                for &key in self.keys() {
                    let value = current_values.feattles.remove(key);
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
    async fn update(
        &self,
        key: &str,
        value: Value,
        modified_by: String,
    ) -> Result<(), UpdateError<P::Error>> {
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

            // Step 1
            let old_value = inner
                .feattles_struct
                .try_update(key, Some(new_value.clone()))?;

            (new_values, old_value)
        };

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
                FailedPersistence(err)
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
                FailedPersistence(err)
            })?;

        // Step 3
        if let Err(err) = persistence.save_current(&new_values).await {
            rollback_step_1();
            if let Err(err) = self.persistence().save_history(key, &old_history).await {
                log::warn!("Failed to rollback history for {}: {:?}", key, err);
            }
            return Err(FailedPersistence(err));
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
    async fn history(&self, key: &str) -> Result<ValueHistory, HistoryError<P::Error>> {
        // Assert the key exists
        if !self.keys().contains(&key) {
            return Err(HistoryError::UnknownKey(key.to_owned()));
        }

        let history = self
            .persistence()
            .load_history(key)
            .await
            .map_err(HistoryError::FailedPersistence)?;

        Ok(history.unwrap_or_default())
    }
}

/// This struct is `pub` because the macro must have access to it, but should be otherwise invisible
/// to the users of this crate.
#[doc(hidden)]
pub trait FeattlesPrivate<P: Persist> {
    type FeattleStruct: FeattlesStruct;
    fn _read(&self) -> RwLockReadGuard<InnerFeattles<Self::FeattleStruct>>;
    fn _write(&self) -> RwLockWriteGuard<InnerFeattles<Self::FeattleStruct>>;
}
