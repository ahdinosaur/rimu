mod environment;
mod eval;
mod from;
mod function;
mod native;
mod number;
mod serde;
mod tag;

use indexmap::IndexMap;
use rimu_meta::{Span, Spanned};

use std::fmt::{Debug, Display};

pub use self::environment::{Environment, EnvironmentError};
pub use self::eval::{BothTagged, EvalError};
pub use self::function::{Function, FunctionBody};
pub use self::native::NativeFunction;
pub use self::number::Number;
pub use self::serde::{
    convert, from_serde_value, to_serde_value, SerdeValue, SerdeValueError, SerdeValueList,
    SerdeValueMeta, SerdeValueObject,
};
pub use self::tag::{merge_tag_metas, peel_tag, rewrap_tag, TagMeta};

pub type ValueList = Vec<SpannedValue>;
pub type ValueObject = IndexMap<String, SpannedValue>;
pub type ValueMeta = IndexMap<String, SpannedValue>;

/// Reserved keys used to (de)serialize tagged values through a JSON-safe
/// envelope object. A deserialized object with all three keys in the expected
/// shapes is promoted back to [`Value::Tagged`]. All three keys carry the
/// `__rimu_` prefix so a plain user object is extremely unlikely to collide.
pub const TAGGED_KEY: &str = "__rimu_tag";
pub const TAGGED_VALUE_KEY: &str = "__rimu_value";
pub const TAGGED_META_KEY: &str = "__rimu_meta";

#[derive(Default, Clone)]
pub enum Value {
    #[default]
    Null,
    Boolean(bool),
    String(String),
    Number(Number),
    Function(Function),
    List(ValueList),
    Object(ValueObject),
    /// A value annotated with a consumer-defined `tag` and arbitrary `meta`.
    /// Tags are opaque to the evaluator. Every op that reads a value first
    /// peels the outer tag (if any), operates on the inner, and re-wraps:
    ///
    /// - unary, arithmetic/concat, boolean short-circuit, index/key/slice,
    ///   and function calls propagate the tag + meta to the result,
    /// - same-tag operands combine: tag is kept, metas are merged
    ///   (right-wins on key collision),
    /// - different-tag operands error ([`BothTagged`]), except when a
    ///   short-circuiting `&&` / `||` never evaluates the right-hand side,
    /// - ordering comparisons (`<`, `>`, `<=`, `>=`) peel and return a plain
    ///   boolean (tag is dropped — the result is a truth value, not a member
    ///   of the tagged domain),
    /// - equality is structural over tag, inner, and meta.
    ///
    /// Nested tagging (`Tagged { inner: Tagged { ... } }`) is not supported —
    /// operations unwrap at most one level and will `TypeError` on the inner
    /// tagged value. Producers are expected to keep tagging flat.
    Tagged {
        tag: String,
        inner: Box<SpannedValue>,
        meta: ValueMeta,
    },
}

pub type SpannedValue = Spanned<Value>;

/// Spans on nested [`SpannedValue`]s are ignored, so values built at
/// different source locations still compare equal.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.inner() == y.inner())
            }
            (Value::Object(a), Value::Object(b)) => object_eq(a, b),
            (
                Value::Tagged {
                    tag: tag_a,
                    inner: inner_a,
                    meta: meta_a,
                },
                Value::Tagged {
                    tag: tag_b,
                    inner: inner_b,
                    meta: meta_b,
                },
            ) => tag_a == tag_b && inner_a.inner() == inner_b.inner() && object_eq(meta_a, meta_b),
            _ => false,
        }
    }
}

fn object_eq(a: &ValueObject, b: &ValueObject) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().all(|(key, value)| match b.get(key) {
        Some(other) => value.inner() == other.inner(),
        None => false,
    })
}

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
            Value::Tagged { tag, inner, meta } => SerdeValue::Tagged {
                tag,
                inner: Box::new((*inner).into()),
                meta: convert_value_object_to_serde_value_object(meta),
            },
        }
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
            SerdeValue::Tagged { tag, inner, meta } => Value::Tagged {
                tag: tag.clone(),
                inner: Box::new(inner.with_span(span.clone())),
                meta: ValueMeta::from_iter(
                    meta.iter()
                        .map(|(key, value)| (key.clone(), value.clone().with_span(span.clone())))
                        .collect::<Vec<_>>(),
                ),
            },
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
            Value::Tagged { tag, inner, meta } => {
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
            Value::Tagged { tag, inner, meta } => {
                if meta.is_empty() {
                    write!(f, "<{} {}>", tag, inner.inner())
                } else {
                    let m = meta
                        .iter()
                        .map(|(key, value)| format!("{}: {}", key, value.inner()))
                        .collect::<Vec<String>>()
                        .join(", ");
                    write!(f, "<{} {} {{{}}}>", tag, inner.inner(), m)
                }
            }
        }
    }
}

/// Everything except `false` and `null' is truthy. A tagged value delegates
/// truthiness to its inner value.
impl From<Value> for bool {
    fn from(value: Value) -> Self {
        #[allow(clippy::match_like_matches_macro)]
        match value {
            Value::Null | Value::Boolean(false) => false,
            Value::Tagged { inner, .. } => inner.into_inner().into(),
            _ => true,
        }
    }
}
