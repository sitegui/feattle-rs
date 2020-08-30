use crate::definition::{SerializedFormat, StringFormat};
use crate::json_reading::{
    extract_array, extract_bool, extract_f64, extract_i64, extract_object, extract_str,
    FromJsonError,
};
use crate::{SerializedFormatKind, StringFormatKind};
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
    fn overview(&self) -> String;
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError>;
    fn serialized_format() -> SerializedFormat;
}

pub trait FeattleStringValue: FromStr + ToString + Debug {
    fn serialized_string_format() -> StringFormat;
}

impl<T: FeattleStringValue> FeattleValue for T
where
    <T as FromStr>::Err: Error + Send + Sync + 'static,
{
    fn as_json(&self) -> Value {
        Value::String(self.to_string())
    }
    fn overview(&self) -> String {
        self.to_string()
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        extract_str(value)?.parse().map_err(FromJsonError::parsing)
    }
    fn serialized_format() -> SerializedFormat {
        let f = Self::serialized_string_format();
        SerializedFormat {
            kind: SerializedFormatKind::String(f.kind),
            tag: f.tag,
        }
    }
}

impl FeattleValue for bool {
    fn as_json(&self) -> Value {
        Value::Bool(*self)
    }
    fn overview(&self) -> String {
        self.to_string()
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        extract_bool(value)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat {
            kind: SerializedFormatKind::Bool,
            tag: "bool".to_owned(),
        }
    }
}

macro_rules! impl_try_from_value_i64 {
    ($kind:ty) => {
        impl FeattleValue for $kind {
            fn as_json(&self) -> Value {
                serde_json::to_value(*self).unwrap()
            }
            fn overview(&self) -> String {
                self.to_string()
            }
            fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
                extract_i64(value)?
                    .try_into()
                    .map_err(FromJsonError::parsing)
            }
            fn serialized_format() -> SerializedFormat {
                SerializedFormat {
                    kind: SerializedFormatKind::Integer,
                    tag: stringify!($kind).to_owned(),
                }
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
    fn overview(&self) -> String {
        self.to_string()
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
        SerializedFormat {
            kind: SerializedFormatKind::Float,
            tag: "f32".to_owned(),
        }
    }
}

impl FeattleValue for f64 {
    fn as_json(&self) -> Value {
        Value::Number(Number::from_f64(*self).unwrap())
    }
    fn overview(&self) -> String {
        self.to_string()
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        extract_f64(value)
    }
    fn serialized_format() -> SerializedFormat {
        SerializedFormat {
            kind: SerializedFormatKind::Float,
            tag: "f64".to_owned(),
        }
    }
}

#[cfg(feature = "uuid")]
impl FeattleStringValue for Uuid {
    fn serialized_string_format() -> StringFormat {
        StringFormat {
            kind: StringFormatKind::Pattern(
                "[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}",
            ),
            tag: "Uuid".to_owned(),
        }
    }
}

impl FeattleStringValue for String {
    fn serialized_string_format() -> StringFormat {
        StringFormat {
            kind: StringFormatKind::Any,
            tag: "String".to_owned(),
        }
    }
}

impl<T: FeattleValue> FeattleValue for Vec<T> {
    fn as_json(&self) -> Value {
        Value::Array(self.iter().map(|item| item.as_json()).collect())
    }
    fn overview(&self) -> String {
        format!("[{}]", iter_overview(self.iter()))
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        let mut list = Vec::new();
        for item in extract_array(value)? {
            list.push(T::try_from_json(item)?);
        }
        Ok(list)
    }
    fn serialized_format() -> SerializedFormat {
        let f = T::serialized_format();
        SerializedFormat {
            kind: SerializedFormatKind::List(Box::new(f.kind)),
            tag: format!("Vec<{}>", f.tag),
        }
    }
}

impl<T: FeattleValue + Ord> FeattleValue for BTreeSet<T> {
    fn as_json(&self) -> Value {
        Value::Array(self.iter().map(|item| item.as_json()).collect())
    }
    fn overview(&self) -> String {
        format!("[{}]", iter_overview(self.iter()))
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        let mut set = BTreeSet::new();
        for item in extract_array(value)? {
            set.insert(T::try_from_json(item)?);
        }
        Ok(set)
    }
    fn serialized_format() -> SerializedFormat {
        let f = T::serialized_format();
        SerializedFormat {
            kind: SerializedFormatKind::Set(Box::new(f.kind)),
            tag: format!("Set<{}>", f.tag),
        }
    }
}

impl<K: FeattleStringValue + Ord, V: FeattleValue> FeattleValue for BTreeMap<K, V>
where
    <K as FromStr>::Err: Error + Send + Sync + 'static,
{
    fn as_json(&self) -> Value {
        Value::Object(
            self.iter()
                .map(|(item_key, item_value)| (item_key.to_string(), item_value.as_json()))
                .collect(),
        )
    }
    fn overview(&self) -> String {
        // Group by value
        let mut keys_by_value: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for (key, value) in self {
            keys_by_value.entry(value.overview()).or_default().push(key);
        }

        let overview_by_value: Vec<_> = keys_by_value
            .into_iter()
            .map(|(value, keys)| format!("{}: {}", iter_overview(keys.into_iter()), value))
            .collect();

        format!("{{{}}}", iter_overview(overview_by_value.iter()))
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
        let fk = K::serialized_string_format();
        let fv = V::serialized_format();
        SerializedFormat {
            kind: SerializedFormatKind::Map(fk.kind, Box::new(fv.kind)),
            tag: format!("Map<{}, {}>", fk.tag, fv.tag),
        }
    }
}

impl<T: FeattleValue> FeattleValue for Option<T> {
    fn as_json(&self) -> Value {
        match self {
            None => Value::Null,
            Some(inner) => inner.as_json(),
        }
    }
    fn overview(&self) -> String {
        match self {
            None => "None".to_owned(),
            Some(s) => format!("Some({})", s.overview()),
        }
    }
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError> {
        match value {
            Value::Null => Ok(None),
            other => T::try_from_json(other).map(Some),
        }
    }
    fn serialized_format() -> SerializedFormat {
        let f = T::serialized_format();
        SerializedFormat {
            kind: SerializedFormatKind::Optional(Box::new(f.kind)),
            tag: format!("Option<{}>", f.tag),
        }
    }
}

fn iter_overview<'a, T: FeattleValue + 'a>(iter: impl Iterator<Item = &'a T>) -> String {
    const MAX_ITEMS: usize = 3;
    let mut overview = String::new();
    let mut iter = iter.enumerate();

    while let Some((i, value)) = iter.next() {
        if i == MAX_ITEMS {
            overview += &format!(", ... {} more", iter.count() + 1);
            break;
        } else if i > 0 {
            overview += ", ";
        }
        overview += &value.overview();
    }

    overview
}
