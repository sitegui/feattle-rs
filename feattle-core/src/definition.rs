use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct SerializedFormat {
    /// An exact and machine-readable description of the format
    pub kind: SerializedFormatKind,
    /// A human-readable description of the format
    pub tag: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum SerializedFormatKind {
    Bool,
    Number,
    String(StringFormatKind),
    List(Box<SerializedFormatKind>),
    Set(Box<SerializedFormatKind>),
    Map(StringFormatKind, Box<SerializedFormatKind>),
    Optional(Box<SerializedFormatKind>),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct StringFormat {
    /// An exact and machine-readable description of the format
    pub kind: StringFormatKind,
    /// A human-readable description of the format
    pub tag: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum StringFormatKind {
    Any,
    Pattern(&'static str),
    Choices(&'static [&'static str]),
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureDefinition {
    pub key: &'static str,
    pub description: String,
    pub format: SerializedFormat,
    pub value: Value,
    pub default: Value,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
}

impl fmt::Display for SerializedFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

impl fmt::Display for StringFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}
