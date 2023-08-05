use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::Path,
    string::{String, ToString},
};
use std::{ffi::OsString, path::PathBuf};

use crate::{Number, Value};

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::Number(n.into())
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
    f32 f64
}

impl From<bool> for Value {
    fn from(f: bool) -> Self {
        Value::Boolean(f)
    }
}

impl From<String> for Value {
    fn from(f: String) -> Self {
        Value::String(f)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(f: &str) -> Self {
        Value::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    fn from(f: Cow<'a, str>) -> Self {
        Value::String(f.into_owned())
    }
}

impl From<OsString> for Value {
    fn from(from: OsString) -> Self {
        Value::String(from.to_str().unwrap_or("unknown").to_string())
    }
}

impl From<PathBuf> for Value {
    fn from(from: PathBuf) -> Self {
        Value::String(from.display().to_string())
    }
}

impl From<&Path> for Value {
    fn from(from: &Path) -> Self {
        Value::String(from.display().to_string())
    }
}

impl From<Number> for Value {
    fn from(f: Number) -> Self {
        Value::Number(f)
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(f: BTreeMap<String, Value>) -> Self {
        Value::Object(f)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(f: Vec<T>) -> Self {
        Value::List(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Value>> From<&'a [T]> for Value {
    fn from(f: &'a [T]) -> Self {
        Value::List(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<Value>> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::List(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Value {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Value::Object(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl From<()> for Value {
    fn from((): ()) -> Self {
        Value::Null
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => Value::Null,
            Some(value) => Into::into(value),
        }
    }
}
