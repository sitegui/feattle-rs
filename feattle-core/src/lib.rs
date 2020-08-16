mod definition;
#[doc(hidden)]
pub mod deps;
mod feattle_value;

pub use definition::*;
pub use feattle_value::*;

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
        #[derive(Debug)]
        struct $name {
            $(
                $key: $crate::deps::RwLock<$type>
            ),*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $(
                        $key: $crate::deps::RwLock::new($crate::__init_field!($($default)?))
                    ),*
                }
            }

            $(
                pub fn $key(&self) -> $crate::deps::RwLockReadGuard<$type> {
                    self.$key.read()
                }
            )*

            fn update(&self, mut values: BTreeMap<String, $crate::deps::Value>) {
                $(
                    Self::update_single(&self.$key, values.remove(stringify!($key)), stringify!($key));
                )*
            }

            fn update_single<T: $crate::FeattleValue>(field: &$crate::deps::RwLock<T>, value: Option<$crate::deps::Value>, key: &str) {
                if let Some(value) = value {
                    match $crate::FeattleValue::try_from_json(value) {
                        Some(x) => *field.write() = x,
                        None => $crate::deps::error!("Failed to parse {}", key),
                    }
                }
            }

            fn definitions(&self) -> Vec<$crate::FeatureDefinition> {
                let mut features = vec![];
                $(
                    features.push($crate::FeatureDefinition {
                        key: stringify!($key),
                        description: concat!($($description),*).trim().to_owned(),
                        format: <$type>::serialized_format(),
                        value: self.$key.read().as_json()
                    });
                )*
                features
            }
        }
    }
}
