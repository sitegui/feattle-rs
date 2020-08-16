#[doc(hidden)]
pub mod __deps;
mod definition;
mod feattle_value;
pub mod models;

use crate::models::CurrentValues;
use chrono::{DateTime, Utc};
pub use definition::*;
pub use feattle_value::*;
pub use strum::VariantNames;

#[derive(Debug, Clone)]
pub struct InnerFeattle<T> {
    pub value: T,
    pub default: T,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
}

impl<T: Clone> InnerFeattle<T> {
    pub fn new(default: T) -> Self {
        InnerFeattle {
            value: default.clone(),
            default,
            modified_at: None,
            modified_by: None,
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

pub trait Feattles: Send + Sync + 'static {
    fn new() -> Self;
    fn current_version(&self) -> Option<(i32, DateTime<Utc>)>;
    fn last_update(&self) -> Option<DateTime<Utc>>;
    fn update(&self, current_values: CurrentValues);
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
                pub __current_version: Option<(i32, $crate::__deps::DateTime<$crate::__deps::Utc>)>,
                pub __last_update: Option<$crate::__deps::DateTime<$crate::__deps::Utc>>,
                $(
                    pub $key: $crate::InnerFeattle<$type>
                ),*
            }
        }

        #[derive(Debug)]
        struct $name {
            inner: $crate::__deps::RwLock<__inner_feattles::InnerFeattles>
        }

        impl Feattles for $name {
            fn new() -> Self {
                let inner = __inner_feattles::InnerFeattles {
                    __current_version: None,
                    __last_update: None,
                    $(
                        $key: $crate::InnerFeattle::new($crate::__init_field!($($default)?))
                    ),*
                };
                Self {
                    inner: $crate::__deps::RwLock::new(inner),
                }
            }

            fn current_version(&self) -> Option<(i32, $crate::__deps::DateTime<$crate::__deps::Utc>)> {
                self.inner.read().__current_version
            }

            fn last_update(&self) -> Option<$crate::__deps::DateTime<$crate::__deps::Utc>> {
                self.inner.read().__last_update
            }

            fn update(&self, mut current_values: $crate::models::CurrentValues) {
                let mut inner = self.inner.write();
                inner.__current_version = Some((current_values.version, current_values.date));
                inner.__last_update = Some($crate::__deps::Utc::now());
                $(
                    let value = current_values.features.remove(stringify!($key));
                    Self::update_single(&mut inner.$key, value, stringify!($key));
                )*
            }

            fn definitions(&self) -> Vec<$crate::FeatureDefinition> {
                let inner = self.inner.read();
                let mut features = vec![];
                $(
                    features.push($crate::FeatureDefinition {
                        key: stringify!($key),
                        description: concat!($($description),*).trim().to_owned(),
                        format: <$type>::serialized_format(),
                        value: inner.$key.value.as_json(),
                        default: inner.$key.default.as_json(),
                        modified_at: inner.$key.modified_at,
                        modified_by: inner.$key.modified_by.clone(),
                    });
                )*
                features
            }
        }

        impl $name {
            $(
                pub fn $key(&self) -> $crate::__deps::MappedRwLockReadGuard<$type> {
                    $crate::__deps::RwLockReadGuard::map(self.inner.read(), |inner| &inner.$key.value)
                }
            )*

            fn update_single<T: $crate::FeattleValue>(
                field: &mut $crate::InnerFeattle<T>,
                value: Option<$crate::models::CurrentValue>,
                key: &str)
            {
                if let Some(value) = value {
                    match $crate::FeattleValue::try_from_json(value.value) {
                        Some(x) => {
                            field.value = x;
                            field.modified_at = Some(value.modified_at);
                            field.modified_by = Some(value.modified_by);
                        },
                        None => $crate::__deps::error!("Failed to parse {}", key),
                    }
                }
            }
        }
    }
}
