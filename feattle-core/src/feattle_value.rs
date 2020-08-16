use crate::definition::{SerializedFormat, StringFormat};
use crate::json_reading::{
    extract_array, extract_bool, extract_f64, extract_i64, extract_object, extract_str,
    FromJsonError,
};
use serde_json::{Number, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
#[cfg(feature = "uuid")]
use uuid::Uuid;

pub trait FeattleValue: Debug + Sized {
    fn as_json(&self) -> Value;
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError>;
    fn serialized_format() -> SerializedFormat;
}

pub trait FeattleStringValue: FromStr + ToString + Debug {
    fn serialized_string_format() -> StringFormat;
}

impl<T: FeattleStringValue> FeattleValue for T
where
    <T as FromStr>::Err: Error + 'static,
{
    fn as_json(&self) -> Value {
        Value::String(self.to_string())
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        extract_str(value)?.parse().map_err(FromJsonError::parsing)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::String(Self::serialized_string_format())
    }
}

impl FeattleValue for bool {
    fn as_json(&self) -> Value {
        Value::Bool(*self)
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        extract_bool(value)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Bool
    }
}

macro_rules! impl_try_from_value_i64 {
    ($kind:ty) => {
        impl FeattleValue for $kind {
            fn as_json(&self) -> Value {
                serde_json::to_value(*self).unwrap()
            }
            fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
                extract_i64(value)?
                    .try_into()
                    .map_err(FromJsonError::parsing)
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
    fn as_json(&self) -> Value {
        Value::Number(Number::from_f64(*self as f64).unwrap())
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        let n_64 = extract_f64(value)?;
        let n_32 = n_64 as f32;
        if n_64 != n_32 as f64 {
            Err(FromJsonError::WrongKind {
                actual: "Number::f64",
                expected: "Number::f32",
            })
        } else {
            Ok(n_32)
        }
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Number
    }
}

impl FeattleValue for f64 {
    fn as_json(&self) -> Value {
        Value::Number(Number::from_f64(*self).unwrap())
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        extract_f64(value)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Number
    }
}

#[cfg(feature = "uuid")]
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
    fn as_json(&self) -> Value {
        Value::Array(self.iter().map(|item| item.as_json()).collect())
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        let mut list = Vec::new();
        for item in extract_array(value)? {
            list.push(T::try_from_json(item)?);
        }
        Ok(list)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::List(Box::new(T::serialized_format()))
    }
}

impl<T: FeattleValue + Ord> FeattleValue for BTreeSet<T> {
    fn as_json(&self) -> Value {
        Value::Array(self.iter().map(|item| item.as_json()).collect())
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        let mut set = BTreeSet::new();
        for item in extract_array(value)? {
            set.insert(T::try_from_json(item)?);
        }
        Ok(set)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Set(Box::new(T::serialized_format()))
    }
}

impl<K: FeattleStringValue + Ord, V: FeattleValue> FeattleValue for BTreeMap<K, V>
where
    <K as FromStr>::Err: Error + 'static,
{
    fn as_json(&self) -> Value {
        Value::Object(
            self.iter()
                .map(|(item_key, item_value)| (item_key.to_string(), item_value.as_json()))
                .collect(),
        )
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        let mut map = BTreeMap::new();
        for (item_key, item_value) in extract_object(value)? {
            map.insert(
                item_key.parse().map_err(FromJsonError::parsing)?,
                V::try_from_json(item_value)?,
            );
        }
        Ok(map)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Map(
            K::serialized_string_format(),
            Box::new(V::serialized_format()),
        )
    }
}

impl<T: FeattleValue> FeattleValue for Option<T> {
    fn as_json(&self) -> Value {
        match self {
            None => Value::Null,
            Some(inner) => inner.as_json(),
        }
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        match value {
            Value::Null => Ok(None),
            other => T::try_from_json(other).map(Some),
        }
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat::Optional(Box::new(T::serialized_format()))
    }
}
