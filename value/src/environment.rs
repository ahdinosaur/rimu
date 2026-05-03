use indexmap::IndexMap;
use std::{cell::RefCell, iter::empty, rc::Rc};

use crate::{SerdeValue, SerdeValueObject, SpannedValue, Value, ValueObject};

/// Variable scope used by the evaluator.
///
/// Stores fully typed [`SpannedValue`]s, so all `Value` variants — including
/// `HostPath` / `TargetPath` — survive variable bindings, function-arg
/// passing, and closure capture without going through serde flattening.
///
/// The serde-shaped constructors ([`from_value`](Self::from_value),
/// [`from_object`](Self::from_object)) and the [`From<SerdeValue> for
/// SpannedValue`] insert path remain for callers that drive the env from
/// YAML/JSON or test fixtures; those paths do not have typed variants to
/// preserve in the first place, so the lossy bridge there is by construction.
#[derive(Debug, Clone)]
pub struct Environment {
    content: ValueObject,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            content: IndexMap::new(),
            parent: None,
        }
    }

    pub fn new_with_parent(parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            content: IndexMap::new(),
            parent: Some(parent),
        }
    }

    pub fn from_value(
        value: &SerdeValue,
        parent: Option<Rc<RefCell<Environment>>>,
    ) -> Result<Environment, EnvironmentError> {
        if let SerdeValue::Object(object) = value {
            Self::from_object(object, parent)
        } else {
            Err(EnvironmentError::InvalidEnvironmentValue {
                value: Box::new(value.clone()),
            })
        }
    }

    pub fn from_object(
        object: &SerdeValueObject,
        parent: Option<Rc<RefCell<Environment>>>,
    ) -> Result<Environment, EnvironmentError> {
        let mut context = Environment {
            content: IndexMap::new(),
            parent,
        };

        for (key, value) in object.iter() {
            context.insert(key, value.clone());
        }

        Ok(context)
    }

    pub fn insert<K, V>(&mut self, k: K, v: V)
    where
        K: Into<String>,
        V: Into<SpannedValue>,
    {
        self.content.insert(k.into(), v.into());
    }

    pub fn get(&self, key: &str) -> Option<SpannedValue> {
        match self.content.get(key) {
            Some(value) => Some(value.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().get(key),
                None => None,
            },
        }
    }

    // Note(cc): `Vec<&str>` is awkward at call sites — `&[&str]` would be more
    // idiomatic and zero-cost. Held for a separate cleanup pass to keep the
    // typed-env diff focused.
    pub fn get_in(&self, keys: Vec<&str>) -> Option<SpannedValue> {
        let (first, rest) = keys.split_first()?;
        match self.get(first) {
            Some(value) => value_get_in(&value, rest).cloned(),
            None => None,
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (String, SpannedValue)>> {
        let parent_iter = match &self.parent {
            Some(parent) => parent.borrow().iter(),
            None => Box::new(empty()),
        };
        let self_iter = self.content.clone().into_iter();
        Box::new(parent_iter.chain(self_iter))
    }
}

/// Walk an object value down a key path. Returns the inner [`SpannedValue`]
/// reference so callers can decide whether to clone or take a slice further.
fn value_get_in<'a>(value: &'a SpannedValue, keys: &[&str]) -> Option<&'a SpannedValue> {
    let Some((first, rest)) = keys.split_first() else {
        return Some(value);
    };
    match value.inner() {
        Value::Object(object) => match object.get(*first) {
            Some(value) => value_get_in(value, rest),
            None => None,
        },
        _ => None,
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum EnvironmentError {
    #[error("context value is not an object: {:?}", value)]
    InvalidEnvironmentValue { value: Box<SerdeValue> },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use indexmap::indexmap;
    use pretty_assertions::assert_eq;
    use rimu_meta::{Span, Spanned};

    use super::*;

    fn empty_span() -> Span {
        Span::default()
    }

    #[test]
    fn insert_get_preserves_host_path() {
        // The bug we're fixing: `Value::HostPath` must round-trip through
        // `Environment` without flattening to `Value::String`.
        let mut env = Environment::new();
        let path = PathBuf::from("/abs/host/path");
        env.insert(
            "p",
            Spanned::new(Value::HostPath(path.clone()), empty_span()),
        );
        match env.get("p").expect("present").into_inner() {
            Value::HostPath(got) => assert_eq!(got, path),
            other => panic!("expected HostPath, got {other:?}"),
        }
    }

    #[test]
    fn insert_get_preserves_target_path() {
        // Sibling: same invariant for `Value::TargetPath`.
        use typed_path::Utf8TypedPathBuf;
        let mut env = Environment::new();
        let path = Utf8TypedPathBuf::from_unix("/abs/target/path");
        env.insert(
            "p",
            Spanned::new(Value::TargetPath(path.clone()), empty_span()),
        );
        match env.get("p").expect("present").into_inner() {
            Value::TargetPath(got) => assert_eq!(got, path),
            other => panic!("expected TargetPath, got {other:?}"),
        }
    }

    #[test]
    fn from_object_does_not_promote_strings_to_paths() {
        // Pin down the deliberate non-promotion: feeding a `SerdeValue::String("/abs")`
        // via the serde-shaped constructor must yield `Value::String`, never
        // `Value::HostPath`. Typed-promotion is the caller's responsibility
        // (in lusid, that's `params::validate`).
        let object: SerdeValueObject = indexmap! {
            "p".to_string() => SerdeValue::String("/abs".into()),
        };
        let env = Environment::from_object(&object, None).expect("ok");
        match env.get("p").expect("present").into_inner() {
            Value::String(s) => assert_eq!(s, "/abs"),
            other => panic!("expected String, got {other:?}"),
        }
    }
}
