use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::str::FromStr;
use uuid::Uuid;

pub trait TryFromValue
where
    Self: Sized,
{
    fn try_from_value(value: Value) -> Option<Self>;
}

impl TryFromValue for bool {
    fn try_from_value(value: Value) -> Option<Self> {
        value.as_bool()
    }
}

macro_rules! impl_try_from_value_i64 {
    ($kind:ty) => {
        impl TryFromValue for $kind {
            fn try_from_value(value: Value) -> Option<Self> {
                value.as_i64().and_then(|n| n.try_into().ok())
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

impl TryFromValue for f32 {
    fn try_from_value(value: Value) -> Option<Self> {
        value.as_f64().and_then(|n_64| {
            let n_32 = n_64 as f32;
            if n_64 != n_32 as f64 {
                None
            } else {
                Some(n_32)
            }
        })
    }
}

impl TryFromValue for f64 {
    fn try_from_value(value: Value) -> Option<Self> {
        value.as_f64()
    }
}

impl TryFromValue for Uuid {
    fn try_from_value(value: Value) -> Option<Self> {
        value.as_str().and_then(|s| s.parse().ok())
    }
}

impl TryFromValue for String {
    fn try_from_value(value: Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

impl<T: TryFromValue> TryFromValue for Vec<T> {
    fn try_from_value(value: Value) -> Option<Self> {
        match value {
            Value::Array(items) => {
                let mut list = Vec::new();
                for item in items {
                    list.push(T::try_from_value(item)?);
                }
                Some(list)
            }
            _ => None,
        }
    }
}

impl<T: TryFromValue + Ord> TryFromValue for BTreeSet<T> {
    fn try_from_value(value: Value) -> Option<Self> {
        match value {
            Value::Array(items) => {
                let mut set = BTreeSet::new();
                for item in items {
                    set.insert(T::try_from_value(item)?);
                }
                Some(set)
            }
            _ => None,
        }
    }
}

impl<K: FromStr + Ord, V: TryFromValue> TryFromValue for BTreeMap<K, V> {
    fn try_from_value(value: Value) -> Option<Self> {
        match value {
            Value::Object(items) => {
                let mut map = BTreeMap::new();
                for (item_key, item_value) in items {
                    map.insert(item_key.parse().ok()?, V::try_from_value(item_value)?);
                }
                Some(map)
            }
            _ => None,
        }
    }
}
