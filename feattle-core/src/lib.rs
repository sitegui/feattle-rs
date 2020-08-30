//! The base implementation of _Feattle_, based on the main macro `feattles!`. Consult the doc on
//! the main package for more.

#[doc(hidden)]
pub mod __internal;
mod definition;
mod feattle_value;
pub mod json_reading;
/// This module only contains exported macros, that are documented at the root level.
#[doc(hidden)]
pub mod macros;
pub mod persist;

use crate::__internal::{FeaturesStruct, InnerFeattles};
use crate::json_reading::FromJsonError;
use crate::persist::{CurrentValue, CurrentValues, HistoryEntry, Persist, ValueHistory};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
pub use definition::*;
pub use feattle_value::*;
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard, RwLockWriteGuard};
use serde_json::Value;
pub use strum::VariantNames;
use thiserror::Error;

/// A boxed error, conveniently compatible with `anyhow::Error`
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("cannot update because current values were never successfully loaded from the persist layer")]
    NeverReloaded,
    #[error("the key {0} is unknown")]
    UnknownKey(String),
    #[error(transparent)]
    FailedParsing(#[from] FromJsonError),
    #[error("failed to persist new state")]
    FailedPersistence(#[source] Error),
}

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("the key {0} is unknown")]
    UnknownKey(String),
    #[error("failed to load persisted state")]
    FailedPersistence(#[source] Error),
}

#[async_trait]
pub trait Feattles<P: Persist>: Send + Sync + 'static {
    type FeatureStruct: FeaturesStruct;
    fn _read(&self) -> RwLockReadGuard<InnerFeattles<Self::FeatureStruct>>;
    fn _write(&self) -> RwLockWriteGuard<InnerFeattles<Self::FeatureStruct>>;
    fn new(persistence: P) -> Self;
    fn persistence(&self) -> &P;
    fn keys(&self) -> &'static [&'static str];
    fn definition(&self, key: &str) -> Option<FeatureDefinition>;

    fn last_reload(&self) -> Option<DateTime<Utc>> {
        self._read().last_reload
    }

    fn current_values(&self) -> Option<MappedRwLockReadGuard<CurrentValues>> {
        let inner = self._read();
        match inner.current_values.as_ref() {
            None => None,
            Some(_) => Some(RwLockReadGuard::map(inner, |x| {
                x.current_values.as_ref().unwrap()
            })),
        }
    }

    async fn reload(&self) -> Result<(), Error> {
        let current_values = self.persistence().load_current().await?;
        let mut inner = self._write();
        let now = Utc::now();
        inner.last_reload = Some(now);
        match current_values {
            None => {
                let empty = CurrentValues {
                    version: 0,
                    date: now,
                    features: Default::default(),
                };
                inner.current_values = Some(empty);
            }
            Some(mut current_values) => {
                for &key in self.keys() {
                    let value = current_values.features.remove(key);
                    if let Err(error) = inner.feattles_struct.update(key, value) {
                        log::error!("Failed to update {}: {:?}", key, error);
                    }
                }
                inner.current_values = Some(current_values);
            }
        }
        Ok(())
    }

    async fn update(
        &self,
        key: &str,
        value: Value,
        modified_by: String,
    ) -> Result<(), UpdateError> {
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
                .features
                .insert(key.to_owned(), new_value.clone());

            // Step 1
            let old_value = inner.feattles_struct.update(key, Some(new_value.clone()))?;

            (new_values, old_value)
        };

        let rollback_step_1 = || {
            self._write()
                .feattles_struct
                .update(key, old_value.clone())
                .expect("it should work because it was the previous value for it");
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

    fn definitions(&self) -> Vec<FeatureDefinition> {
        self.keys()
            .iter()
            .map(|&key| {
                self.definition(key)
                    .expect("since we iterate over the list of known keys, this should always work")
            })
            .collect()
    }

    async fn history(&self, key: &str) -> Result<ValueHistory, HistoryError> {
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
