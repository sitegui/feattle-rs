pub mod __internal;
mod definition;
mod feattle_value;
pub mod json_reading;
pub mod macros;
pub mod persist;

use crate::__internal::{FeaturesStruct, InnerFeattles};
use crate::json_reading::FromJsonError;
use crate::persist::{CurrentValue, CurrentValues, Persist};
use chrono::{DateTime, Utc};
pub use definition::*;
pub use feattle_value::*;
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard, RwLockWriteGuard};
use serde_json::Value;
use std::error::Error;
pub use strum::VariantNames;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("cannot update because current values were never successfully loaded from the persist layer")]
    NeverReloaded,
    #[error("the key {0} is unknown")]
    UnknownKey(String),
    #[error(transparent)]
    FailedParsing(#[from] FromJsonError),
    #[error("failed to persist new state")]
    FailedPersistence(#[source] Box<dyn Error>),
}

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

    fn reload(&self) -> Result<(), Box<dyn Error>> {
        let current_values = self.persistence().load_current()?;
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
                self.persistence().save_current(&empty)?;
                inner.current_values = Some(empty);
            }
            Some(current_values) => {
                for &key in self.keys() {
                    if let Some(value) = current_values.features.get(key) {
                        if let Err(error) = inner.feattles_struct.update(key, value) {
                            log::error!("Failed to update {}: {:?}", key, error);
                        }
                    }
                }
                inner.current_values = Some(current_values);
            }
        }
        Ok(())
    }

    fn update(&self, key: &str, value: Value, modified_by: String) -> Result<(), UpdateError> {
        // Load current state
        let mut inner = self._write();

        // Assert the key exists
        if !self.keys().contains(&key) {
            return Err(UpdateError::UnknownKey(key.to_owned()));
        }

        // Update in-memory
        let now = Utc::now();
        let current_value = CurrentValue {
            modified_at: now,
            modified_by,
            value,
        };
        inner.feattles_struct.update(key, &current_value)?;

        // Prepare storage
        let current_values = inner
            .current_values
            .as_mut()
            .ok_or(UpdateError::NeverReloaded)?;
        current_values.version += 1;
        current_values.date = now;
        current_values
            .features
            .insert(key.to_owned(), current_value);

        // Update persistent storage
        self.persistence()
            .save_current(current_values)
            .map_err(UpdateError::FailedPersistence)?;

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
}
