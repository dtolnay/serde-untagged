pub enum Error {
    Custom(String),
    InvalidType {
        unexpected: Unexpected,
        expected: String,
    },
    InvalidValue {
        unexpected: Unexpected,
        expected: String,
    },
    InvalidLength {
        len: usize,
        expected: String,
    },
    UnknownVariant {
        variant: String,
        expected: &'static [&'static str],
    },
    UnknownField {
        field: String,
        expected: &'static [&'static str],
    },
    MissingField {
        field: &'static str,
    },
    DuplicateField {
        field: &'static str,
    },
}

pub enum Unexpected {
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Char(char),
    Str(String),
    Bytes(Vec<u8>),
    Unit,
    Option,
    NewtypeStruct,
    Seq,
    Map,
    Enum,
    UnitVariant,
    NewtypeVariant,
    TupleVariant,
    StructVariant,
    Other(String),
}

impl Error {
    fn as_serde<E: serde::de::Error>(&self) -> E {
        match self {
            Error::Custom(msg) => E::custom(msg),
            Error::InvalidType {
                unexpected,
                expected,
            } => E::invalid_type(unexpected.as_serde(), &expected.as_str()),
            Error::InvalidValue {
                unexpected,
                expected,
            } => E::invalid_value(unexpected.as_serde(), &expected.as_str()),
            Error::InvalidLength { len, expected } => E::invalid_length(*len, &expected.as_str()),
            Error::UnknownVariant { variant, expected } => E::unknown_variant(variant, expected),
            Error::UnknownField { field, expected } => E::unknown_field(field, expected),
            Error::MissingField { field } => E::missing_field(field),
            Error::DuplicateField { field } => E::duplicate_field(field),
        }
    }
}

impl Unexpected {
    fn as_serde(&self) -> serde::de::Unexpected {
        match self {
            Unexpected::Bool(value) => serde::de::Unexpected::Bool(*value),
            Unexpected::Unsigned(value) => serde::de::Unexpected::Unsigned(*value),
            Unexpected::Signed(value) => serde::de::Unexpected::Signed(*value),
            Unexpected::Float(value) => serde::de::Unexpected::Float(*value),
            Unexpected::Char(value) => serde::de::Unexpected::Char(*value),
            Unexpected::Str(value) => serde::de::Unexpected::Str(value),
            Unexpected::Bytes(value) => serde::de::Unexpected::Bytes(value),
            Unexpected::Unit => serde::de::Unexpected::Unit,
            Unexpected::Option => serde::de::Unexpected::Option,
            Unexpected::NewtypeStruct => serde::de::Unexpected::NewtypeStruct,
            Unexpected::Seq => serde::de::Unexpected::Seq,
            Unexpected::Map => serde::de::Unexpected::Map,
            Unexpected::Enum => serde::de::Unexpected::Enum,
            Unexpected::UnitVariant => serde::de::Unexpected::UnitVariant,
            Unexpected::NewtypeVariant => serde::de::Unexpected::NewtypeVariant,
            Unexpected::TupleVariant => serde::de::Unexpected::TupleVariant,
            Unexpected::StructVariant => serde::de::Unexpected::StructVariant,
            Unexpected::Other(msg) => serde::de::Unexpected::Other(msg),
        }
    }
}
