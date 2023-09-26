mod convert;
mod de;
mod error;
mod from;
mod ser;

use std::fmt::{Debug, Display};

use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::{Function, Number};

pub use self::convert::convert;
pub use self::error::SerdeValueError;

use self::ser::Serializer;

pub type SerdeValueList = Vec<SerdeValue>;
pub type SerdeValueObject = IndexMap<String, SerdeValue>;

#[derive(Default, Clone, PartialEq)]
pub enum SerdeValue {
    #[default]
    Null,
    Boolean(bool),
    String(String),
    Number(Number),
    Function(Function),
    List(SerdeValueList),
    Object(SerdeValueObject),
}

pub fn to_serde_value<T>(value: T) -> Result<SerdeValue, SerdeValueError>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

pub fn from_serde_value<T>(value: SerdeValue) -> Result<T, SerdeValueError>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

impl Debug for SerdeValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerdeValue::Null => formatter.write_str("Null"),
            SerdeValue::Boolean(boolean) => match boolean {
                true => formatter.write_str("true"),
                false => formatter.write_str("false"),
            },
            SerdeValue::String(string) => write!(formatter, "String({:?})", string),
            SerdeValue::Number(number) => write!(formatter, "Number({})", number),
            SerdeValue::Function(function) => {
                write!(formatter, "Function({:?})", function)
            }
            SerdeValue::List(list) => {
                formatter.write_str("List ")?;
                formatter.debug_list().entries(list).finish()
            }
            SerdeValue::Object(object) => {
                formatter.write_str("Object ")?;
                formatter.debug_map().entries(object).finish()
            }
        }
    }
}

impl Display for SerdeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerdeValue::Null => write!(f, "null"),
            SerdeValue::Boolean(boolean) => write!(f, "{}", boolean),
            SerdeValue::String(string) => write!(f, "{}", string),
            SerdeValue::Number(number) => write!(f, "{}", number),
            SerdeValue::Function(function) => write!(f, "{}", function),
            SerdeValue::List(list) => {
                let items = list
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{}]", items)
            }
            SerdeValue::Object(object) => {
                let entries = object
                    .iter()
                    .map(|(key, value)| format!("\"{}\": {}", key, value))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{}}}", entries)
            }
        }
    }
}

pub fn value_get_in<'a>(value: &'a SerdeValue, keys: &[&str]) -> Option<&'a SerdeValue> {
    let Some((first, rest)) = keys.split_first() else {
        return Some(value);
    };
    match value {
        SerdeValue::Object(object) => match object.get(*first) {
            Some(value) => value_get_in(value, rest),
            None => None,
        },
        _ => None,
    }
}

/// Everything except `false` and `null' is truthy.
impl From<SerdeValue> for bool {
    fn from(value: SerdeValue) -> Self {
        #[allow(clippy::match_like_matches_macro)]
        match value {
            SerdeValue::Null | SerdeValue::Boolean(false) => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, ffi::OsString, path::PathBuf};

    use crate::SerdeValue;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn from_string_tests() {
        assert_eq!(
            SerdeValue::from("John Sheppard"),
            SerdeValue::String("John Sheppard".to_string())
        );

        assert_eq!(
            SerdeValue::from("Elizabeth Weir".to_string()),
            SerdeValue::String("Elizabeth Weir".to_string())
        );

        assert_eq!(
            SerdeValue::from(PathBuf::new()),
            SerdeValue::String("".to_string())
        );

        assert_eq!(
            SerdeValue::from(Cow::from("Samantha Carter")),
            SerdeValue::String("Samantha Carter".to_string())
        );

        assert_eq!(
            SerdeValue::from(OsString::from("Jennifer Keller")),
            SerdeValue::String("Jennifer Keller".to_string())
        );
    }

    #[test]
    fn from_vec_test() {
        assert_eq!(
            SerdeValue::from(vec!["Aiden Ford", "Rodney McKay", "Ronon Dex"]),
            SerdeValue::List(vec![
                SerdeValue::String("Aiden Ford".to_string()),
                SerdeValue::String("Rodney McKay".to_string()),
                SerdeValue::String("Ronon Dex".to_string())
            ])
        );
    }

    #[test]
    fn debug_tests() {
        assert_eq!(format!("{:?}", SerdeValue::Null), "Null".to_string());

        assert_eq!(
            format!("{:?}", SerdeValue::String("Richard Woolsey".to_string())),
            "String(\"Richard Woolsey\")".to_string()
        );

        assert_eq!(
            format!(
                "{:?}",
                SerdeValue::List(vec![
                    SerdeValue::String("Aiden Ford".to_string()),
                    SerdeValue::String("Rodney McKay".to_string()),
                    SerdeValue::String("Ronon Dex".to_string())
                ])
            ),
            "List [String(\"Aiden Ford\"), String(\"Rodney McKay\"), String(\"Ronon Dex\")]"
                .to_string()
        );

        assert_eq!(
            format!("{:?}", SerdeValue::Number(dec!(2).into())),
            "Number(2)".to_string()
        );
    }
}
