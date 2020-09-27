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

/// The base trait for types that can be used for feattles.
///
/// This lib already implements this trait for many base types from the std lib, but the user can
/// make their own types compatible by providing their own logic.
///
/// For types that are string based, it suffices to implement the somewhat simpler
/// [`FeattleStringValue`] trait.
pub trait FeattleValue: Debug + Sized {
    /// Convert the value to its JSON representation.
    fn as_json(&self) -> Value;

    /// Return a short overview of the current value. This is meant to give an overall idea of the
    /// actual value. For example, it can choose to display only the first 3 items of a large list.
    fn overview(&self) -> String;

    /// Parse from a JSON representation of the value, if possible.
    fn try_from_json(value: &Value) -> Result<Self, FromJsonError>;

    /// Return a precise description of a feattle type. This will be consumed, for example, by the
    /// UI code to show an appropriate HTML form in the admin panel.
    fn serialized_format() -> SerializedFormat;
}

/// The base trait for string-types that can be used for feattles.
///
/// This trait should be used for types that behave like string. A blanked implementation of
/// [`FeattleValue`] for types that implement this trait will provide the necessary compatibility
/// to use them as feattles.
///
/// Note that this trait also requires that the type implements:
/// * [`Debug`]
/// * [`ToString`]
/// * [`FromStr`], with a compatible error
pub trait FeattleStringValue: FromStr + ToString + Debug
where
    <Self as FromStr>::Err: Error + Send + Sync + 'static,
{
    /// Return a precise description of a feattle type. This will be consumed, for example, by the
    /// UI code to show an appropriate HTML form in the admin panel.
    fn serialized_string_format() -> StringFormat;
}

impl<T> FeattleValue for T
where
    T: FeattleStringValue,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn converts<T: FeattleValue + PartialEq>(value: Value, parsed: T, overview: &str) {
        converts2(value.clone(), parsed, overview, value);
    }

    fn converts2<T: FeattleValue + PartialEq>(
        value: Value,
        parsed: T,
        overview: &str,
        converted: Value,
    ) {
        assert_eq!(parsed.as_json(), converted);
        assert_eq!(parsed.overview(), overview);
        assert_eq!(T::try_from_json(&value).ok(), Some(parsed));
    }

    fn fails<T: FeattleValue + PartialEq>(value: Value) {
        assert_eq!(T::try_from_json(&value).ok(), None);
    }

    #[test]
    fn bool() {
        converts(json!(true), true, "true");
        converts(json!(false), false, "false");

        fails::<bool>(json!(0));
        fails::<bool>(json!(null));

        assert_eq!(bool::serialized_format().kind, SerializedFormatKind::Bool);
    }

    #[test]
    fn int() {
        fn basic<T: FeattleValue + PartialEq>(parsed: T) {
            converts(json!(17), parsed, "17");
            fails::<T>(json!(17.5));
            fails::<T>(json!(null));
            assert_eq!(T::serialized_format().kind, SerializedFormatKind::Integer);
        }

        basic(17u8);
        basic(17i8);
        basic(17u16);
        basic(17i16);
        basic(17u32);
        basic(17i32);
        basic(17u64);
        basic(17i64);
        basic(17usize);
        basic(17isize);

        fails::<u8>(json!(-17));
        converts(json!(-17), -17i8, "-17");
        fails::<u16>(json!(-17));
        converts(json!(-17), -17i16, "-17");
        fails::<u32>(json!(-17));
        converts(json!(-17), -17i32, "-17");
        fails::<u64>(json!(-17));
        converts(json!(-17), -17i64, "-17");
        fails::<usize>(json!(-17));
        converts(json!(-17), -17isize, "-17");

        let overview = u32::MAX.to_string();
        fails::<u8>(json!(u32::MAX));
        fails::<i8>(json!(u32::MAX));
        fails::<u16>(json!(u32::MAX));
        fails::<i16>(json!(u32::MAX));
        converts(json!(u32::MAX), u32::MAX, &overview);
        fails::<i32>(json!(u32::MAX));
        converts(json!(u32::MAX), u32::MAX as u64, &overview);
        converts(json!(u32::MAX), u32::MAX as i64, &overview);
        converts(json!(u32::MAX), u32::MAX as usize, &overview);
        converts(json!(u32::MAX), u32::MAX as isize, &overview);
    }

    #[test]
    fn float() {
        converts2(json!(17), 17f32, "17", json!(17.0));
        converts2(json!(17), 17f64, "17", json!(17.0));
        converts(json!(17.5), 17.5f32, "17.5");
        converts(json!(17.5), 17.5f64, "17.5");

        fails::<bool>(json!(null));

        assert_eq!(f32::serialized_format().kind, SerializedFormatKind::Float);
        assert_eq!(f64::serialized_format().kind, SerializedFormatKind::Float);
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn uuid() {
        converts(
            json!("8886fc87-93e1-4d08-9722-9fc1411b6b96"),
            Uuid::parse_str("8886fc87-93e1-4d08-9722-9fc1411b6b96").unwrap(),
            "8886fc87-93e1-4d08-9722-9fc1411b6b96",
        );

        fails::<Uuid>(json!("yadayada"));
        let kind = Uuid::serialized_format().kind;
        match kind {
            SerializedFormatKind::String(StringFormatKind::Pattern(_)) => {}
            _ => panic!("invalid serialized format kind: {:?}", kind),
        }
    }

    #[test]
    fn string() {
        converts(json!("17"), "17".to_owned(), "17");
        converts(json!(""), "".to_owned(), "");
        fails::<String>(json!(17));
        fails::<String>(json!(null));
        assert_eq!(
            String::serialized_format().kind,
            SerializedFormatKind::String(StringFormatKind::Any)
        );
    }

    #[test]
    fn vec() {
        converts(json!([3, 14, 15]), vec![3i32, 14, 15], "[3, 14, 15]");
        converts(
            json!([3, 14, 15, 92]),
            vec![3i32, 14, 15, 92],
            "[3, 14, 15, ... 1 more]",
        );
        converts(
            json!([3, 14, 15, 92, 65, 35]),
            vec![3i32, 14, 15, 92, 65, 35],
            "[3, 14, 15, ... 3 more]",
        );
        fails::<Vec<i32>>(json!([3, 14, "15", 92]));
        assert_eq!(
            Vec::<i32>::serialized_format().kind,
            SerializedFormatKind::List(Box::new(SerializedFormatKind::Integer))
        )
    }

    #[test]
    fn set() {
        converts(
            json!([3, 14, 15]),
            vec![3, 14, 15].into_iter().collect::<BTreeSet<i32>>(),
            "[3, 14, 15]",
        );
        converts2(
            json!([1, 2, 4, 4, 3]),
            vec![1, 2, 3, 4].into_iter().collect::<BTreeSet<i32>>(),
            "[1, 2, 3, ... 1 more]",
            json!([1, 2, 3, 4]),
        );
        fails::<BTreeSet<i32>>(json!([3, 14, "15", 92]));
        assert_eq!(
            BTreeSet::<i32>::serialized_format().kind,
            SerializedFormatKind::Set(Box::new(SerializedFormatKind::Integer))
        )
    }

    #[test]
    fn map() {
        converts(
            json!({
                "a": 1,
                "b": 2,
                "x": 1,
            }),
            vec![
                ("a".to_owned(), 1),
                ("b".to_owned(), 2),
                ("x".to_owned(), 1),
            ]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
            "{a, x: 1, b: 2}",
        );
        fails::<BTreeMap<String, String>>(json!({
            "a": "1",
            "b": 2,
            "x": 1,
        }));
        assert_eq!(
            BTreeMap::<String, i32>::serialized_format().kind,
            SerializedFormatKind::Map(
                StringFormatKind::Any,
                Box::new(SerializedFormatKind::Integer)
            )
        )
    }

    #[test]
    fn option() {
        converts(json!(17), Some(17), "Some(17)");
        converts(json!(null), None::<i32>, "None");
        fails::<Option<i32>>(json!(17.5));
        assert_eq!(
            Option::<i32>::serialized_format().kind,
            SerializedFormatKind::Optional(Box::new(SerializedFormatKind::Integer))
        )
    }

    #[test]
    fn choices() {
        use crate::feattle_enum;
        feattle_enum! {enum Choices { Red, Green, Blue }};

        converts(json!("Red"), Choices::Red, "Red");
        fails::<Choices>(json!("Black"));
        assert_eq!(
            Choices::serialized_format().kind,
            SerializedFormatKind::String(StringFormatKind::Choices(&["Red", "Green", "Blue"]))
        )
    }
}
