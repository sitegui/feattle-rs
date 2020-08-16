#[doc(hidden)]
pub mod __deps;
mod definition;
mod feattle_value;
pub mod models;
pub mod persist;

use crate::models::CurrentValues;
use chrono::{DateTime, Utc};
pub use definition::*;
pub use feattle_value::*;
use parking_lot::MappedRwLockReadGuard;
use serde_json::Value;
use std::error::Error;
pub use strum::VariantNames;

#[derive(Debug, Clone)]
pub struct InnerFeattle<T> {
    pub key: &'static str,
    pub description: String,
    pub value: T,
    pub default: T,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
}

impl<T: Clone + FeattleValue> InnerFeattle<T> {
    pub fn new(key: &'static str, description: String, default: T) -> Self {
        InnerFeattle {
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
            description: self.description.clone(),
            format: T::serialized_format(),
            value: self.value.as_json(),
            default: self.default.as_json(),
            modified_at: self.modified_at,
            modified_by: self.modified_by.clone(),
        }
    }
}

#[macro_export]
macro_rules! __init_field {
    ($default:expr) => {
        $default
    };
    () => {
        Default::default()
    };
}

pub trait Feattles<P>: Send + Sync + 'static {
    fn new(persistence: P) -> Self;
    fn last_reload(&self) -> Option<DateTime<Utc>>;
    fn current_values(&self) -> MappedRwLockReadGuard<Option<CurrentValues>>;
    fn persistence(&self) -> &P;

    fn keys(&self) -> &'static [&'static str];
    fn reload(&self) -> Result<(), Box<dyn Error>>;
    fn update(&self, key: &str, value: Value) -> Result<(), Box<dyn Error>>;

    fn definition(&self, key: &str) -> Option<FeatureDefinition>;
    fn definitions(&self) -> Vec<FeatureDefinition>;
}

#[macro_export]
macro_rules! feattles {
    (
    $name:ident {
        $(
            $(#[doc=$description:tt])*
            $key:ident: $type:ty $(= $default:expr)?
        ),*
        $(,)?
    }
    ) => {
        mod __inner_feattles {
            use super::*;

            #[derive(Debug, Clone)]
            pub struct InnerFeattles {
                pub __last_reload: Option<$crate::__deps::DateTime<$crate::__deps::Utc>>,
                pub __current_values: Option<$crate::models::CurrentValues>,
                $(
                    pub $key: $crate::InnerFeattle<$type>
                ),*
            }
        }

        #[derive(Debug)]
        struct $name<P> {
            persistence: P,
            inner: $crate::__deps::RwLock<__inner_feattles::InnerFeattles>
        }

        impl<P: $crate::persist::Persist> Feattles<P> for $name<P> {
            fn new(persistence: P) -> Self {
                let inner = __inner_feattles::InnerFeattles {
                    __last_reload: None,
                    __current_values: None,
                    $(
                        $key: $crate::InnerFeattle::new(
                            stringify!($key),
                            concat!($($description),*).trim().to_owned(),
                            $crate::__init_field!($($default)?),
                        )
                    ),*
                };
                Self {
                    persistence,
                    inner: $crate::__deps::RwLock::new(inner),
                }
            }

            fn last_reload(&self) -> Option<$crate::__deps::DateTime<$crate::__deps::Utc>> {
                self.inner.read().__last_reload
            }

            fn current_values(&self) -> $crate::__deps::MappedRwLockReadGuard<Option<$crate::models::CurrentValues>> {
                $crate::__deps::RwLockReadGuard::map(self.inner.read(), |inner| &inner.__current_values)
            }

            fn persistence(&self) -> &P {
                &self.persistence
            }

            fn keys(&self) -> &'static [&'static str] {
                &[$(
                    stringify!($key)
                ),*]
            }

            fn reload(&self) -> Result<(), Box<dyn ::std::error::Error>> {
                let current_values = self.persistence.load_current()?;
                let mut inner = self.inner.write();
                let now = $crate::__deps::Utc::now();
                inner.__last_reload = Some(now);
                match current_values {
                    None => {
                        inner.__current_values = Some($crate::models::CurrentValues {
                            version: 0,
                            date: now,
                            features: Default::default()
                        });
                    }
                    Some(mut current_values) => {
                        $(
                            let value = current_values.features.get(stringify!($key));
                            Self::update_single(&mut inner.$key, value, stringify!($key));
                        )*
                        inner.__current_values = Some(current_values);
                    }
                }
                Ok(())
            }

            fn update(&self, key: &str, value: $crate::__deps::Value) -> Result<(), Box<dyn std::error::Error>> {
                todo!()
            }

            fn definition(&self, key: &str) -> Option<$crate::FeatureDefinition> {
                let inner = self.inner.read();
                match key {
                    $(
                        stringify!($key) => Some(inner.$key.as_definition()),
                    )*
                    _ => None,
                }
            }

            fn definitions(&self) -> Vec<$crate::FeatureDefinition> {
                let inner = self.inner.read();
                let mut features = vec![
                    $(
                        inner.$key.as_definition()
                    ),*
                ];
                features
            }
        }

        impl<P> $name<P> {
            $(
                pub fn $key(&self) -> $crate::__deps::MappedRwLockReadGuard<$type> {
                    $crate::__deps::RwLockReadGuard::map(self.inner.read(), |inner| &inner.$key.value)
                }
            )*

            fn update_single<T: $crate::FeattleValue>(
                field: &mut $crate::InnerFeattle<T>,
                value: Option<&$crate::models::CurrentValue>,
                key: &str)
            {
                if let Some(value) = value {
                    match $crate::FeattleValue::try_from_json(&value.value) {
                        Some(x) => {
                            field.value = x;
                            field.modified_at = Some(value.modified_at);
                            field.modified_by = Some(value.modified_by.clone());
                        },
                        None => $crate::__deps::error!("Failed to parse {}", key),
                    }
                }
            }
        }
    }
}
