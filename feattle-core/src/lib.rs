mod definition;
mod feattle_value;
pub mod json_reading;
pub mod macros;
pub mod persist;

use crate::json_reading::FromJsonError;
use crate::persist::{CurrentValue, CurrentValues, Persist};
use chrono::{DateTime, Utc};
pub use definition::*;
pub use feattle_value::*;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde_json::Value;
use std::error::Error;
pub use strum::VariantNames;
use thiserror::Error;

#[derive(Debug)]
pub struct FeattlesImpl<P, FS> {
    persistence: P,
    inner_feattles: RwLock<InnerFeattles<FS>>,
}

#[derive(Debug, Clone)]
pub struct InnerFeattles<FS> {
    last_reload: Option<DateTime<Utc>>,
    current_values: Option<CurrentValues>,
    feattles_struct: FS,
}

#[derive(Debug, Clone)]
pub struct Feature<T> {
    key: &'static str,
    description: &'static str,
    value: T,
    default: T,
    modified_at: Option<DateTime<Utc>>,
    modified_by: Option<String>,
}

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

pub trait __FeaturesStruct {
    fn __update(&mut self, key: &str, value: &CurrentValue) -> Result<(), FromJsonError>;
}

impl<P, FS> FeattlesImpl<P, FS> {
    fn new(persistence: P, feattles_struct: FS) -> Self {
        FeattlesImpl {
            persistence,
            inner_feattles: RwLock::new(InnerFeattles {
                last_reload: None,
                current_values: None,
                feattles_struct,
            }),
        }
    }
}

impl<T: Clone + FeattleValue> Feature<T> {
    pub fn new(key: &'static str, description: &'static str, default: T) -> Self {
        Feature {
            key,
            description,
            value: default.clone(),
            default,
            modified_at: None,
            modified_by: None,
        }
    }

    pub fn as_definition(&self) -> FeatureDefinition {
        FeatureDefinition {
            key: self.key,
            description: self.description.to_owned(),
            format: T::serialized_format(),
            value: self.value.as_json(),
            default: self.default.as_json(),
            modified_at: self.modified_at,
            modified_by: self.modified_by.clone(),
        }
    }

    pub fn update(&mut self, value: &CurrentValue) -> Result<(), FromJsonError> {
        self.value = FeattleValue::try_from_json(&value.value)?;
        self.modified_at = Some(value.modified_at);
        self.modified_by = Some(value.modified_by.clone());
        Ok(())
    }
}

pub trait Feattles<P: Persist>: Send + Sync + 'static {
    type FeatureStruct: __FeaturesStruct;
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
                        if let Err(error) = inner.feattles_struct.__update(key, value) {
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
        inner.feattles_struct.__update(key, feature)?;

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
