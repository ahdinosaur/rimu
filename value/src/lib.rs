mod environment;
mod eval;
mod from;
mod function;
mod native;
mod number;
mod serde;

use indexmap::IndexMap;
use rimu_meta::{Span, Spanned};
use typed_path::Utf8TypedPathBuf;

use std::fmt::{Debug, Display};
use std::path::PathBuf;

pub use self::environment::{Environment, EnvironmentError};
pub use self::eval::EvalError;
pub use self::function::{Function, FunctionBody};
pub use self::native::NativeFunction;
pub use self::number::Number;
pub use self::serde::{
    convert, from_serde_value, to_serde_value, SerdeValue, SerdeValueError, SerdeValueList,
    SerdeValueObject,
};

pub type ValueList = Vec<SpannedValue>;
pub type ValueObject = IndexMap<String, SpannedValue>;

#[derive(Default, Clone, PartialEq)]
pub enum Value {
    #[default]
    Null,
    Boolean(bool),
    String(String),
    Number(Number),
    Function(Function),
    List(ValueList),
    Object(ValueObject),
    /// A path on the local machine. Constructed via `host_path("./rel")`,
    /// which resolves the input against the source file's directory at call
    /// time. Absolute when the source id is itself an absolute path; otherwise
    /// relative to the current working directory.
    HostPath(PathBuf),
    /// An absolute path on a remote machine. Stored as a `Utf8TypedPathBuf`
    /// from the `typed-path` crate so we can carry Unix or Windows path
    /// semantics independently of the host platform — useful when the host
    /// runs Linux but targets a Windows box, or vice versa.
    TargetPath(Utf8TypedPathBuf),
}

pub type SpannedValue = Spanned<Value>;

impl From<SpannedValue> for SerdeValue {
    fn from(value: SpannedValue) -> Self {
        value.into_inner().into()
    }
}

impl From<Value> for SerdeValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => SerdeValue::Null,
            Value::Boolean(boolean) => SerdeValue::Boolean(boolean),
            Value::String(string) => SerdeValue::String(string),
            Value::Number(number) => SerdeValue::Number(number),
            Value::Function(function) => SerdeValue::Function(function),
            Value::List(list) => SerdeValue::List(convert_value_list_to_serde_value_list(list)),
            Value::Object(object) => {
                SerdeValue::Object(convert_value_object_to_serde_value_object(object))
            }
            Value::HostPath(path) => SerdeValue::String(path.display().to_string()),
            Value::TargetPath(path) => SerdeValue::String(path.into_string()),
        }
    }
}

/// Lossy: `SerdeValue` has no `HostPath`/`TargetPath` variants, so a string
/// stays a string here. Typed-promotion (e.g. `Value::String("/abs")` →
/// `Value::HostPath(...)`) is the caller's responsibility — in lusid that's
/// `params::validate`. Functions, lists, objects round-trip recursively.
impl From<SerdeValue> for Value {
    fn from(value: SerdeValue) -> Self {
        match value {
            SerdeValue::Null => Value::Null,
            SerdeValue::Boolean(boolean) => Value::Boolean(boolean),
            SerdeValue::String(string) => Value::String(string),
            SerdeValue::Number(number) => Value::Number(number),
            SerdeValue::Function(function) => Value::Function(function),
            SerdeValue::List(list) => Value::List(
                list.into_iter()
                    .map(|item| Spanned::new(item.into(), Span::default()))
                    .collect(),
            ),
            SerdeValue::Object(object) => Value::Object(
                object
                    .into_iter()
                    .map(|(k, v)| (k, Spanned::new(v.into(), Span::default())))
                    .collect(),
            ),
        }
    }
}

/// Lossy bridge for serde-shaped sources (CLI YAML/JSON, wasm playground
/// stdlib, test env-objects). Callers with a real source span should
/// construct `SpannedValue` directly — using this `From` silently sets
/// `Span::default()`, which renders as `:0..0` against an empty source id in
/// any later diagnostic.
impl From<SerdeValue> for SpannedValue {
    fn from(value: SerdeValue) -> Self {
        Spanned::new(value.into(), Span::default())
    }
}

pub fn convert_value_list_to_serde_value_list(list: ValueList) -> SerdeValueList {
    list.iter().map(|item| item.clone().into()).collect()
}

pub fn convert_value_object_to_serde_value_object(object: ValueObject) -> SerdeValueObject {
    SerdeValueObject::from_iter(
        object
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().into()))
            .collect::<Vec<_>>(),
    )
}

impl SerdeValue {
    pub fn with_span(&self, span: Span) -> SpannedValue {
        let value = match self {
            SerdeValue::Null => Value::Null,
            SerdeValue::Boolean(boolean) => Value::Boolean(boolean.to_owned()),
            SerdeValue::String(string) => Value::String(string.to_owned()),
            SerdeValue::Number(number) => Value::Number(number.to_owned()),
            SerdeValue::Function(function) => Value::Function(function.to_owned()),
            SerdeValue::List(list) => Value::List(
                list.iter()
                    .map(|item| item.clone().with_span(span.clone()))
                    .collect(),
            ),
            SerdeValue::Object(object) => Value::Object(ValueObject::from_iter(
                object
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone().with_span(span.clone())))
                    .collect::<Vec<_>>(),
            )),
        };
        Spanned::new(value, span)
    }
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
            Value::Function(function) => {
                write!(formatter, "Function({:?})", function)
            }
            Value::List(list) => {
                formatter.write_str("List ")?;
                formatter.debug_list().entries(list).finish()
            }
            Value::Object(object) => {
                formatter.write_str("Object ")?;
                formatter.debug_map().entries(object).finish()
            }
            Value::HostPath(path) => write!(formatter, "HostPath({})", path.display()),
            Value::TargetPath(path) => write!(formatter, "TargetPath({:?})", path.as_str()),
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
            Value::Function(function) => write!(f, "{}", function),
            Value::List(list) => {
                let items = list
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{}]", items)
            }
            Value::Object(object) => {
                let entries = object
                    .iter()
                    .map(|(key, value)| format!("\"{}\": {}", key, value))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{}}}", entries)
            }
            Value::HostPath(path) => write!(f, "{}", path.display()),
            Value::TargetPath(path) => write!(f, "{}", path.as_str()),
        }
    }
}

/// Everything except `false` and `null' is truthy.
impl From<Value> for bool {
    fn from(value: Value) -> Self {
        #[allow(clippy::match_like_matches_macro)]
        match value {
            Value::Null | Value::Boolean(false) => false,
            _ => true,
        }
    }
}
