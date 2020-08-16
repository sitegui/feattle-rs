use crate::reflection::{SerializedFormat, StringFormat};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::str::FromStr;
use uuid::Uuid;

#[macro_export]
macro_rules! feattle_enum {
    ($key:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
        #[derive($crate::deps::EnumString)]
        #[derive($crate::deps::EnumVariantNames)]
        enum $key { $($variant),* }

        impl FeattleStringValue for $key {
            fn serialized_string_format() -> StringFormat {
                StringFormat::Choices(&$key::VARIANTS)
            }
        }
    }
}

pub trait FeattleValue
where
    Self: Sized,
{
    fn try_from_json(value: Value) -> Option<Self>;
    fn serialized_format() -> SerializedFormat;
}

pub trait FeattleStringValue: FromStr
where
    Self: Sized,
{
    fn serialized_string_format() -> StringFormat;
}

impl<T: FeattleStringValue> FeattleValue for T {
    fn try_from_json(value: Value) -> Option<Self> {
        value.as_str().and_then(|s| s.parse().ok())
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::String(Self::serialized_string_format())
    }
}

impl FeattleValue for bool {
    fn try_from_json(value: Value) -> Option<Self> {
        value.as_bool()
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Bool
    }
}

macro_rules! impl_try_from_value_i64 {
    ($kind:ty) => {
        impl FeattleValue for $kind {
            fn try_from_json(value: Value) -> Option<Self> {
                value.as_i64().and_then(|n| n.try_into().ok())
            }
            fn serialized_format() -> SerializedFormat {
                SerializedFormat::Number
            }
        }
    };
}

impl_try_from_value_i64! {u8}
impl_try_from_value_i64! {i8}
impl_try_from_value_i64! {u16}
impl_try_from_value_i64! {i16}
impl_try_from_value_i64! {u32}
impl_try_from_value_i64! {i32}
impl_try_from_value_i64! {u64}
impl_try_from_value_i64! {i64}
impl_try_from_value_i64! {u128}
impl_try_from_value_i64! {i128}
impl_try_from_value_i64! {usize}
impl_try_from_value_i64! {isize}

impl FeattleValue for f32 {
    fn try_from_json(value: Value) -> Option<Self> {
        value.as_f64().and_then(|n_64| {
            let n_32 = n_64 as f32;
            if n_64 != n_32 as f64 {
                None
            } else {
                Some(n_32)
            }
        })
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Number
    }
}

impl FeattleValue for f64 {
    fn try_from_json(value: Value) -> Option<Self> {
        value.as_f64()
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Number
    }
}

impl FeattleStringValue for Uuid {
    fn serialized_string_format() -> StringFormat {
        StringFormat::Pattern(
            "[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}",
        )
    }
}

impl FeattleStringValue for String {
    fn serialized_string_format() -> StringFormat {
        StringFormat::Any
    }
}

impl<T: FeattleValue> FeattleValue for Vec<T> {
    fn try_from_json(value: Value) -> Option<Self> {
        match value {
            Value::Array(items) => {
                let mut list = Vec::new();
                for item in items {
                    list.push(T::try_from_json(item)?);
                }
                Some(list)
            }
            _ => None,
        }
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::List(Box::new(T::serialized_format()))
    }
}

impl<T: FeattleValue + Ord> FeattleValue for BTreeSet<T> {
    fn try_from_json(value: Value) -> Option<Self> {
        match value {
            Value::Array(items) => {
                let mut set = BTreeSet::new();
                for item in items {
                    set.insert(T::try_from_json(item)?);
                }
                Some(set)
            }
            _ => None,
        }
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Set(Box::new(T::serialized_format()))
    }
}

impl<K: FeattleStringValue + Ord, V: FeattleValue> FeattleValue for BTreeMap<K, V> {
    fn try_from_json(value: Value) -> Option<Self> {
        match value {
            Value::Object(items) => {
                let mut map = BTreeMap::new();
                for (item_key, item_value) in items {
                    map.insert(item_key.parse().ok()?, V::try_from_json(item_value)?);
                }
                Some(map)
            }
            _ => None,
        }
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Map(
            K::serialized_string_format(),
            Box::new(V::serialized_format()),
        )
    }
}
