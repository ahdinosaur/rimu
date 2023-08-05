pub(crate) mod convert;
pub(crate) mod de;
pub(crate) mod error;
pub(crate) mod from;
pub(crate) mod number;
pub(crate) mod ser;

use std::collections::BTreeMap;

pub type List = Vec<Value>;
pub type Object = BTreeMap<String, Value>;

use std::fmt::{Debug, Display};

use serde::de::DeserializeOwned;
use serde::Serialize;

pub use self::convert::convert;
pub use self::error::ValueError;
pub use self::number::Number;
use self::ser::Serializer;

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Boolean(bool),
    String(String),
    Number(Number),
    List(List),
    Object(Object),
}

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

pub fn to_value<T>(value: T) -> Result<Value, ValueError>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

pub fn from_value<T>(value: Value) -> Result<T, ValueError>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => formatter.write_str("Null"),
            Value::Boolean(boolean) => match boolean {
                true => formatter.write_str("true"),
                false => formatter.write_str("false"),
            },
            Value::String(string) => write!(formatter, "String({:?})", string),
            Value::Number(number) => write!(formatter, "Number({})", number),
            Value::List(list) => {
                formatter.write_str("List ")?;
                formatter.debug_list().entries(list).finish()
            }
            Value::Object(object) => {
                formatter.write_str("Object ")?;
                formatter.debug_map().entries(object).finish()
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::String(string) => write!(f, "{}", string),
            Value::Number(number) => write!(f, "{}", number),
            Value::List(list) => {
                let keys = list
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{}]", keys)
            }
            Value::Object(object) => {
                let entries = object
                    .iter()
                    .map(|(key, value)| format!("\"{}\": {}", key, value.to_string()))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{}}}", entries)
            }
        }
    }
}

pub fn value_get_in<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    let Some((first, rest)) = keys.split_first() else {
        return Some(value);
    };
    match value {
        Value::Object(object) => match object.get(*first) {
            Some(value) => value_get_in(value, rest),
            None => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, ffi::OsString, path::PathBuf};

    use crate::{Number, Value};
    use pretty_assertions::assert_eq;

    #[test]
    fn from_string_tests() {
        assert_eq!(
            Value::from("John Sheppard"),
            Value::String("John Sheppard".to_string())
        );

        assert_eq!(
            Value::from("Elizabeth Weir".to_string()),
            Value::String("Elizabeth Weir".to_string())
        );

        assert_eq!(Value::from(PathBuf::new()), Value::String("".to_string()));

        assert_eq!(
            Value::from(Cow::from("Samantha Carter")),
            Value::String("Samantha Carter".to_string())
        );

        assert_eq!(
            Value::from(OsString::from("Jennifer Keller")),
            Value::String("Jennifer Keller".to_string())
        );
    }

    #[test]
    fn from_vec_test() {
        assert_eq!(
            Value::from(vec!["Aiden Ford", "Rodney McKay", "Ronon Dex"]),
            Value::List(vec![
                Value::String("Aiden Ford".to_string()),
                Value::String("Rodney McKay".to_string()),
                Value::String("Ronon Dex".to_string())
            ])
        );
    }

    #[test]
    fn debug_tests() {
        assert_eq!(format!("{:?}", Value::Null), "Null".to_string());

        assert_eq!(
            format!("{:?}", Value::String("Richard Woolsey".to_string())),
            "String(\"Richard Woolsey\")".to_string()
        );

        assert_eq!(
            format!(
                "{:?}",
                Value::List(vec![
                    Value::String("Aiden Ford".to_string()),
                    Value::String("Rodney McKay".to_string()),
                    Value::String("Ronon Dex".to_string())
                ])
            ),
            "List [String(\"Aiden Ford\"), String(\"Rodney McKay\"), String(\"Ronon Dex\")]"
                .to_string()
        );

        assert_eq!(
            format!("{:?}", Value::Number(Number::Unsigned(2))),
            "Number(2)".to_string()
        );

        assert_eq!(
            format!("{:?}", Value::Number(Number::Signed(2))),
            "Number(2)".to_string()
        );

        assert_eq!(
            format!("{:?}", Value::Number(Number::Float(2.0))),
            "Number(2)".to_string()
        );
    }
}