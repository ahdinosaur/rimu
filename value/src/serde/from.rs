use indexmap::IndexMap;
use std::{
    borrow::Cow,
    path::Path,
    string::{String, ToString},
};
use std::{ffi::OsString, path::PathBuf};

use crate::{Function, Number, SerdeValue};

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for SerdeValue {
                fn from(n: $ty) -> Self {
                    SerdeValue::Number(n.into())
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

impl From<bool> for SerdeValue {
    fn from(f: bool) -> Self {
        SerdeValue::Boolean(f)
    }
}

impl From<String> for SerdeValue {
    fn from(f: String) -> Self {
        SerdeValue::String(f)
    }
}

impl From<&str> for SerdeValue {
    fn from(f: &str) -> Self {
        SerdeValue::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for SerdeValue {
    fn from(f: Cow<'a, str>) -> Self {
        SerdeValue::String(f.into_owned())
    }
}

impl From<OsString> for SerdeValue {
    fn from(from: OsString) -> Self {
        SerdeValue::String(from.to_str().unwrap_or("unknown").to_string())
    }
}

impl From<PathBuf> for SerdeValue {
    fn from(from: PathBuf) -> Self {
        SerdeValue::String(from.display().to_string())
    }
}

impl From<&Path> for SerdeValue {
    fn from(from: &Path) -> Self {
        SerdeValue::String(from.display().to_string())
    }
}

impl From<Number> for SerdeValue {
    fn from(f: Number) -> Self {
        SerdeValue::Number(f)
    }
}

impl From<IndexMap<String, SerdeValue>> for SerdeValue {
    fn from(f: IndexMap<String, SerdeValue>) -> Self {
        SerdeValue::Object(f)
    }
}

impl<T: Into<SerdeValue>> From<Vec<T>> for SerdeValue {
    fn from(f: Vec<T>) -> Self {
        SerdeValue::List(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<SerdeValue>> From<&'a [T]> for SerdeValue {
    fn from(f: &'a [T]) -> Self {
        SerdeValue::List(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<SerdeValue>> FromIterator<T> for SerdeValue {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        SerdeValue::List(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<String>, V: Into<SerdeValue>> FromIterator<(K, V)> for SerdeValue {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        SerdeValue::Object(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl From<Function> for SerdeValue {
    fn from(function: Function) -> Self {
        SerdeValue::Function(function)
    }
}

impl From<()> for SerdeValue {
    fn from((): ()) -> Self {
        SerdeValue::Null
    }
}

impl<T> From<Option<T>> for SerdeValue
where
    T: Into<SerdeValue>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => SerdeValue::Null,
            Some(value) => Into::into(value),
        }
    }
}
