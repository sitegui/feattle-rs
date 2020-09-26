//! Internal types and re-exports used by the macros

pub use crate::json_reading::FromJsonError;
pub use crate::persist::{CurrentValue, Persist};
pub use crate::FeattleDefinition;
pub use crate::Feattles;
pub use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard, RwLockWriteGuard};

use crate::persist::CurrentValues;
use crate::FeattleValue;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::error::Error;
use std::{fmt, mem};

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
pub struct Feattle<T> {
    key: &'static str,
    description: &'static str,
    pub value: T,
    default: T,
    current_value: Option<CurrentValue>,
}

#[derive(Copy, Clone, Debug)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matching variant not found")
    }
}

impl Error for ParseError {}

pub trait FeattleStruct {
    fn update(
        &mut self,
        key: &str,
        value: Option<CurrentValue>,
    ) -> Result<Option<CurrentValue>, FromJsonError>;
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

impl<T: Clone + FeattleValue> Feattle<T> {
    pub fn new(key: &'static str, description: &'static str, default: T) -> Self {
        Feattle {
            key,
            description,
            value: default.clone(),
            default,
            current_value: None,
        }
    }

    pub fn definition(&self) -> FeattleDefinition {
        FeattleDefinition {
            key: self.key,
            description: self.description.to_owned(),
            format: T::serialized_format(),
            value: self.value.as_json(),
            value_overview: self.value.overview(),
            default: self.default.as_json(),
            modified_at: self.current_value.as_ref().map(|v| v.modified_at),
            modified_by: self.current_value.as_ref().map(|v| v.modified_by.clone()),
        }
    }

    pub fn update(
        &mut self,
        value: Option<CurrentValue>,
    ) -> Result<Option<CurrentValue>, FromJsonError> {
        let old_value = mem::replace(&mut self.current_value, value);
        self.value = match &self.current_value {
            None => self.default.clone(),
            Some(value) => FeattleValue::try_from_json(&value.value)?,
        };
        Ok(old_value)
    }
}
