//! Helper free functions to read Rust values from `serde_json::Value`

use crate::Error;
use serde_json::{Map, Value};

/// Indicate an error that occurred while trying to read a feattle value from JSON
#[derive(thiserror::Error, Debug)]
pub enum FromJsonError {
    #[error("wrong JSON kind, got {actual} and was expecting {expected}")]
    WrongKind {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("failed to parse")]
    ParseError {
        cause: Box<dyn Error + Send + Sync + 'static>,
    },
}

impl FromJsonError {
    /// Create a new [`FromJsonError::ParseError`] variant
    pub fn parsing<E: Error + Send + Sync + 'static>(error: E) -> FromJsonError {
        FromJsonError::ParseError {
            cause: Box::new(error),
        }
    }
}

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "Null",
        Value::Bool(_) => "Bool",
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Array(_) => "Array",
        Value::Object(_) => "Object",
    }
}

macro_rules! impl_extract_json {
    ($fn_name:ident, $output:ty, $method:ident, $expected:expr) => {
        #[doc = "Try to read as"]
        #[doc = $expected]
        pub fn $fn_name(value: &Value) -> Result<$output, FromJsonError> {
            value.$method().ok_or_else(|| FromJsonError::WrongKind {
                expected: $expected,
                actual: json_kind(value),
            })
        }
    };
}

impl_extract_json! { extract_array, &Vec<Value>, as_array, "Array" }
impl_extract_json! { extract_bool, bool, as_bool, "Bool" }
impl_extract_json! { extract_f64, f64, as_f64, "Number::f64" }
impl_extract_json! { extract_i64, i64, as_i64, "Number::i64" }
impl_extract_json! { extract_null, (), as_null, "Null" }
impl_extract_json! { extract_object, &Map<String, Value>, as_object, "Object" }
impl_extract_json! { extract_str, &str, as_str, "String" }
impl_extract_json! { extract_u64, u64, as_u64, "Number::u64" }
