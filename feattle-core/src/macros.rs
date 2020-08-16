mod __deps {
    pub use chrono::{DateTime, Utc};
    pub use log::error;
    pub use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
    pub use serde_json::Value;
    pub use strum_macros::{Display, EnumString, EnumVariantNames};
}

#[macro_export]
macro_rules! feattle_enum {
    ($key:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
        #[derive($crate::__deps::EnumString)]
        #[derive($crate::__deps::EnumVariantNames)]
        #[derive($crate::__deps::Display)]
        pub enum $key { $($variant),* }

        impl $crate::FeattleStringValue for $key {
            fn serialized_string_format() -> StringFormat {
                StringFormat::Choices(&$key::VARIANTS)
            }
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
