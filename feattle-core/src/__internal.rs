pub use crate::json_reading::FromJsonError;
pub use crate::persist::{CurrentValue, Persist};
pub use crate::Feattles;
pub use crate::FeatureDefinition;
pub use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard, RwLockWriteGuard};
pub use strum_macros::{Display, EnumString, EnumVariantNames};

use crate::persist::CurrentValues;
use crate::FeattleValue;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

#[derive(Debug)]
pub struct FeattlesImpl<P, FS> {
    pub persistence: P,
    pub inner_feattles: RwLock<InnerFeattles<FS>>,
}

#[derive(Debug, Clone)]
pub struct InnerFeattles<FS> {
    pub(crate) last_reload: Option<DateTime<Utc>>,
    pub(crate) current_values: Option<CurrentValues>,
    pub feattles_struct: FS,
}

#[derive(Debug, Clone)]
pub struct Feature<T> {
    key: &'static str,
    description: &'static str,
    pub value: T,
    default: T,
    modified_at: Option<DateTime<Utc>>,
    modified_by: Option<String>,
}

pub trait FeaturesStruct {
    fn update(&mut self, key: &str, value: &CurrentValue) -> Result<(), FromJsonError>;
}

impl<P, FS> FeattlesImpl<P, FS> {
    pub fn new(persistence: P, feattles_struct: FS) -> Self {
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

    pub fn definition(&self) -> FeatureDefinition {
        FeatureDefinition {
            key: self.key,
            description: self.description.to_owned(),
            format: T::serialized_format(),
            value: self.value.as_json(),
            value_overview: self.value.overview(),
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
