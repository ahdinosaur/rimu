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
pub type SerdeValueMeta = IndexMap<String, SerdeValue>;

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
    /// See [`crate::Value::Tagged`]. Serializes as a JSON object with the
    /// reserved key [`crate::TAGGED_KEY`] (`"__rimu_tag"`) so values round-trip
    /// through JSON interchange.
    Tagged {
        tag: String,
        inner: Box<SerdeValue>,
        meta: SerdeValueMeta,
    },
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
            SerdeValue::Tagged { tag, inner, meta } => {
                formatter.write_str("Tagged ")?;
                formatter
                    .debug_struct("")
                    .field("tag", tag)
                    .field("inner", inner)
                    .field("meta", meta)
                    .finish()
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
            SerdeValue::Tagged { tag, inner, meta } => {
                if meta.is_empty() {
                    write!(f, "<{} {}>", tag, inner)
                } else {
                    let m = meta
                        .iter()
                        .map(|(key, value)| format!("{}: {}", key, value))
                        .collect::<Vec<String>>()
                        .join(", ");
                    write!(f, "<{} {} {{{}}}>", tag, inner, m)
                }
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

#[cfg(test)]
mod test {
    use std::{borrow::Cow, ffi::OsString, path::PathBuf};

    use crate::{
        from_serde_value, to_serde_value, SerdeValue, SerdeValueMeta, SerdeValueObject, TAGGED_KEY,
        TAGGED_META_KEY, TAGGED_VALUE_KEY,
    };
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

    #[test]
    fn tagged_serializes_to_envelope_object() {
        let mut meta = SerdeValueMeta::new();
        meta.insert("origin_dir".into(), SerdeValue::String("/src".into()));
        let tagged = SerdeValue::Tagged {
            tag: "host_path".into(),
            inner: Box::new(SerdeValue::String("/abs/a".into())),
            meta,
        };

        let serialized = to_serde_value(&tagged).unwrap();

        let SerdeValue::Object(envelope) = serialized else {
            panic!("expected object, got {:?}", serialized);
        };
        assert_eq!(envelope.len(), 3);
        assert_eq!(
            envelope.get(TAGGED_KEY),
            Some(&SerdeValue::String("host_path".into())),
        );
        assert_eq!(
            envelope.get(TAGGED_VALUE_KEY),
            Some(&SerdeValue::String("/abs/a".into())),
        );
        let meta_entry = envelope.get(TAGGED_META_KEY).unwrap();
        let SerdeValue::Object(meta) = meta_entry else {
            panic!("expected meta to be object");
        };
        assert_eq!(
            meta.get("origin_dir"),
            Some(&SerdeValue::String("/src".into())),
        );
    }

    #[test]
    fn tagged_round_trips_through_envelope() {
        let mut meta = SerdeValueMeta::new();
        meta.insert("origin_dir".into(), SerdeValue::String("/src".into()));
        let tagged = SerdeValue::Tagged {
            tag: "host_path".into(),
            inner: Box::new(SerdeValue::String("/abs/a".into())),
            meta,
        };

        let serialized = to_serde_value(&tagged).unwrap();
        let round_tripped: SerdeValue = from_serde_value(serialized).unwrap();

        assert_eq!(round_tripped, tagged);
    }

    #[test]
    fn envelope_object_deserializes_to_tagged() {
        let mut envelope = SerdeValueObject::new();
        envelope.insert(TAGGED_KEY.into(), SerdeValue::String("host_path".into()));
        envelope.insert(TAGGED_VALUE_KEY.into(), SerdeValue::String("/abs/a".into()));
        envelope.insert(
            TAGGED_META_KEY.into(),
            SerdeValue::Object(SerdeValueObject::new()),
        );

        let value: SerdeValue = from_serde_value(SerdeValue::Object(envelope)).unwrap();

        assert!(matches!(value, SerdeValue::Tagged { .. }));
    }

    #[test]
    fn plain_object_missing_one_reserved_key_stays_object() {
        let mut object = SerdeValueObject::new();
        object.insert(TAGGED_KEY.into(), SerdeValue::String("host_path".into()));
        object.insert(TAGGED_VALUE_KEY.into(), SerdeValue::String("/abs/a".into()));

        let value: SerdeValue = from_serde_value(SerdeValue::Object(object)).unwrap();

        assert!(matches!(value, SerdeValue::Object(_)));
    }
}
