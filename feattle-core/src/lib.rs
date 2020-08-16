mod definition;
#[doc(hidden)]
pub mod deps;
mod feattle_value;

pub use definition::*;
pub use feattle_value::*;

// struct InternalStorage {
//     extrude_mesh_terrain: RwLock<bool>,
// }
//
// impl InternalStorage {
//     read! { extrude_mesh_terrain, bool }
//
//     fn update(&self, mut values: BTreeMap<String, Value>) {
//         write!(self, values, extrude_mesh_terrain);
//     }
// }

#[macro_export]
macro_rules! feattles {
    (
    $name:ident {
        $(
            $(#[doc=$description:tt])*
            $key:ident: $type:ty
        ),*
        $(,)?
    }
    ) => {
        struct $name {
            $(
                $key: $crate::deps::RwLock<$type>
            ),*
        }

        impl $name {
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

            fn definitions() -> Vec<$crate::FeatureDefinition> {
                let mut features = vec![];
                $(
                    features.push($crate::FeatureDefinition {
                        key: stringify!($key),
                        description: concat!($($description),*).trim().to_owned(),
                        format: <$type>::serialized_format()
                    });
                )*
                features
            }
        }
    }
}
