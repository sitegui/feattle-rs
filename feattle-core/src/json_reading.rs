use serde_json::{Map, Value};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FromJsonError {
    #[error("wrong JSON kind, got {actual} and was expecting {expected}")]
    WrongKind {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("failed to parse")]
    ParseError { cause: Box<dyn Error> },
}

impl FromJsonError {
    pub fn parsing<E: Error + 'static>(error: E) -> FromJsonError {
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
