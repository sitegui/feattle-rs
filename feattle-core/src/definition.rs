use serde_json::Value;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SerializedFormat {
    Bool,
    Number,
    String(StringFormat),
    List(Box<SerializedFormat>),
    Set(Box<SerializedFormat>),
    Map(StringFormat, Box<SerializedFormat>),
    Optional(Box<SerializedFormat>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StringFormat {
    Any,
    Pattern(&'static str),
    Choices(&'static [&'static str]),
}

#[derive(Debug, Clone)]
pub struct FeatureDefinition {
    pub key: &'static str,
    pub description: String,
    pub format: SerializedFormat,
    pub value: Value,
}
