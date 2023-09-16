use indexmap::IndexMap;
use std::iter::empty;

use rimu_value::{value_get_in, Object, Value};

#[derive(Debug, Clone)]
pub struct Environment<'a> {
    content: Object,
    parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            content: IndexMap::new(),
            parent: None,
        }
    }

    pub fn child(&'a self) -> Environment<'a> {
        Environment {
            content: IndexMap::new(),
            parent: Some(self),
        }
    }

    pub fn from_value(
        value: &'_ Value,
        parent: Option<&'a Environment>,
    ) -> Result<Environment<'a>, EnvironmentError> {
        if let Value::Object(object) = value {
            Self::from_object(object, parent)
        } else {
            Err(EnvironmentError::InvalidEnvironmentValue {
                value: value.clone(),
            })
        }
    }

    pub fn from_object(
        object: &'_ Object,
        parent: Option<&'a Environment>,
    ) -> Result<Environment<'a>, EnvironmentError> {
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
        V: Into<Value>,
    {
        // TODO check is_identifier
        // ... or key should be a separate struct

        self.content.insert(k.into(), v.into());
    }

    pub fn get<'b>(&'b self, key: &str) -> Option<&'b Value> {
        // TODO check is_identifier
        // ... or key should be a separate struct

        match self.content.get(key) {
            Some(value) => Some(value),
            None => match self.parent {
                Some(parent) => parent.get(key),
                None => None,
            },
        }
    }

    pub fn get_in<'b>(&'b self, keys: Vec<&str>) -> Option<&'b Value> {
        // TODO check is_identifier
        // ... or key should be a separate struct

        let Some((first, rest)) = keys.split_first() else {
            return None;
        };
        match self.get(first) {
            Some(value) => value_get_in(value, rest),
            None => None,
        }
    }

    pub fn iter(&'a self) -> Box<dyn 'a + Iterator<Item = (&String, &Value)>> {
        let parent_iter = match self.parent {
            Some(parent) => parent.iter(),
            None => Box::new(empty()),
        };
        let self_iter = self.content.iter();
        Box::new(parent_iter.chain(self_iter))
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum EnvironmentError {
    #[error("context value is not an object: {:?}", value)]
    InvalidEnvironmentValue { value: Value },
}
