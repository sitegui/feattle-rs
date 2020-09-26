use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::fmt;

/// A precise description of a feattle type
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct SerializedFormat {
    /// An exact and machine-readable description of the format
    pub kind: SerializedFormatKind,
    /// A human-readable description of the format, shown by `Display`
    pub tag: String,
}

/// An exact and machine-readable description of a feattle type.
///
/// This type can be used to create a nice human interface, like a HTML form, to edit the value
/// of a feattle, for example. It can also be used to validate user input.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "tag", content = "content")]
pub enum SerializedFormatKind {
    Bool,
    Integer,
    Float,
    String(StringFormatKind),
    /// An ordered list of homogenous types
    List(Box<SerializedFormatKind>),
    /// An unordered bag of homogenous types
    Set(Box<SerializedFormatKind>),
    /// An unordered bag of homogenous keys and values
    Map(StringFormatKind, Box<SerializedFormatKind>),
    Optional(Box<SerializedFormatKind>),
}

/// A precise description of a feattle string-type
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct StringFormat {
    /// An exact and machine-readable description of the format
    pub kind: StringFormatKind,
    /// A human-readable description of the format, shown by `Display`
    pub tag: String,
}

/// An exact and machine-readable description of a feattle string-type
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "tag", content = "content")]
pub enum StringFormatKind {
    /// Accepts any possible string.
    Any,
    /// The string must conform to the pattern, described using
    /// [JavaScript's RegExp syntax](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions/Cheatsheet)
    /// in `'u'` (Unicode) mode.
    /// The matching is done against the entire value, not just any subset, as if a `^(?:` was
    /// implied at the start of the pattern and a `)$` at the end.
    Pattern(&'static str),
    /// Only one of the listed values is accepted.
    Choices(&'static [&'static str]),
}

/// A data struct, describing a single feattle.
#[derive(Debug, Clone, Serialize)]
pub struct FeattleDefinition {
    /// The feattle's name
    pub key: &'static str,
    /// Its documentation
    pub description: String,
    /// The precise description of its format
    pub format: SerializedFormat,
    /// Its current in-memory value, as JSON
    pub value: Value,
    /// A short human description of its current in-memory value
    pub value_overview: String,
    /// Its default value, as JSON
    pub default: Value,
    /// The last time it was modified by an user
    pub modified_at: Option<DateTime<Utc>>,
    /// The user that last modified it
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
