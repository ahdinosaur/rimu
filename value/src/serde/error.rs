use std::fmt::Display;

use serde::{de, ser};

#[derive(Debug, Clone, thiserror::Error)]
pub enum SerdeValueError {
    #[error("{0}")]
    Deserialize(String),

    #[error("{0}")]
    Serialize(String),

    /// EOF while parsing a list.
    #[error("")]
    EofWhileParsingList,

    /// EOF while parsing an object.
    #[error("")]
    EofWhileParsingObject,

    /// EOF while parsing a string.
    #[error("")]
    EofWhileParsingString,

    /// EOF while parsing a JSON value.
    #[error("")]
    EofWhileParsingValue,

    /// Expected this character to be a `':'`.
    #[error("")]
    ExpectedColon,

    /// Expected this character to be either a `','` or a `']'`.
    #[error("")]
    ExpectedListCommaOrEnd,

    /// Expected this character to be either a `','` or a `'}'`.
    #[error("")]
    ExpectedObjectCommaOrEnd,

    /// Expected to parse either a `true`, `false`, or a `null`.
    #[error("")]
    ExpectedSomeIdent,

    /// Expected this character to start a JSON value.
    #[error("")]
    ExpectedSomeValue,

    /// Expected this character to be a `"`.
    #[error("")]
    ExpectedDoubleQuote,

    /// Invalid hex escape code.
    #[error("")]
    InvalidEscape,

    /// Invalid number.
    #[error("")]
    InvalidNumber,

    /// Number is bigger than the maximum value of its type.
    #[error("")]
    NumberOutOfRange,

    /// Invalid unicode code point.
    #[error("")]
    InvalidUnicodeCodePoint,

    /// Control character found while parsing a string.
    #[error("")]
    ControlCharacterWhileParsingString,

    /// Object key is not a string.
    #[error("")]
    KeyMustBeAString,

    /// Contents of key were supposed to be a number.
    #[error("")]
    ExpectedNumericKey,

    /// Object key is a non-finite float value.
    #[error("")]
    FloatKeyMustBeFinite,

    /// Lone leading surrogate in hex escape.
    #[error("")]
    LoneLeadingSurrogateInHexEscape,

    /// JSON has a comma after the last value in an array or map.
    #[error("")]
    TrailingComma,

    /// JSON has non-whitespace trailing characters after the value.
    #[error("")]
    TrailingCharacters,

    /// Unexpected end of hex escape.
    #[error("")]
    UnexpectedEndOfHexEscape,

    /// Encountered nesting of JSON maps and arrays more than 128 layers deep.
    #[error("")]
    RecursionLimitExceeded,
}

impl de::Error for SerdeValueError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeValueError::Deserialize(msg.to_string())
    }
}

impl ser::Error for SerdeValueError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeValueError::Serialize(msg.to_string())
    }
}
