pub mod __internal;
mod definition;
mod feattle_value;
pub mod json_reading;
pub mod macros;
pub mod persist;

use crate::__internal::{FeaturesStruct, InnerFeattles};
use crate::json_reading::FromJsonError;
use crate::persist::{CurrentValues, Persist};
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
                inner.current_values = Some(CurrentValues {
                    version: 0,
                    date: now,
                    features: Default::default(),
                });
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
        let current_values = inner
            .current_values
            .as_ref()
            .ok_or(UpdateError::NeverReloaded)?;

        // Prepare updated state
        let mut new_values = current_values.clone();
        let now = Utc::now();
        new_values.version += 1;
        new_values.date = now;
        let feature = new_values
            .features
            .get_mut(key)
            .ok_or_else(|| UpdateError::UnknownKey(key.to_owned()))?;
        feature.modified_at = now;
        feature.modified_by = modified_by;
        feature.value = value;

        // Update in-memory
        inner.feattles_struct.update(key, feature)?;

        // Update persistent storage
        self.persistence()
            .save_current(&new_values)
            .map_err(UpdateError::FailedPersistence)?;

        inner.current_values = Some(new_values);

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
