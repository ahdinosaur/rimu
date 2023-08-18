use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::BTreeMap, iter::empty};

use rimu_value::{value_get_in, Object, Value};

pub struct Environment<'a> {
    content: Object,
    parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            content: BTreeMap::new(),
            parent: None,
        }
    }

    pub fn child(&'a self) -> Environment<'a> {
        Environment {
            content: BTreeMap::new(),
            parent: Some(self),
        }
    }

    pub fn from_value(
        value: &'_ Value,
        parent: Option<&'a Environment>,
    ) -> Result<Environment<'a>, EnvironmentError> {
        let mut context = Environment {
            content: BTreeMap::new(),
            parent,
        };

        if let Value::Object(object) = value {
            for key in object.keys() {
                if !is_identifier(key) {
                    return Err(EnvironmentError::InvalidKey { key: key.clone() });
                }
            }
            for (key, value) in object.iter() {
                context.insert(key, value.clone());
            }
        } else {
            return Err(EnvironmentError::InvalidEnvironmentValue {
                value: value.clone(),
            });
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

#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    #[error("top level keys of context must follow /[a-zA-Z_][a-zA-Z0-9_]*: `{key}`")]
    InvalidKey { key: String },
    #[error("context value is not an object: {:?}", value)]
    InvalidEnvironmentValue { value: Value },
}

fn is_identifier(identifier: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }
    RE.is_match(identifier)
}