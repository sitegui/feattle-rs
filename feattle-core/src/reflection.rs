pub enum SerializedFormat {
    Bool,
    Number,
    String(StringFormat),
    List(Box<SerializedFormat>),
    Set(Box<SerializedFormat>),
    Map(StringFormat, Box<SerializedFormat>),
}

pub enum StringFormat {
    Any,
    Pattern(&'static str),
    Choices(&'static [&'static str]),
}

pub struct FeatureDefinition<T: Clone> {
    pub key: String,
    pub description: String,
    pub default: T,
}
