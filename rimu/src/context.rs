use lazy_static::lazy_static;
use regex::Regex;
use rhai::Scope;
use std::{collections::BTreeMap, iter::empty};

use rimu_value::{value_get_in, Object, Value};

pub struct Context<'a> {
    content: Object,
    parent: Option<&'a Context<'a>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            content: BTreeMap::new(),
            parent: None,
        }
    }

    pub fn child(&'a self) -> Context<'a> {
        Context {
            content: BTreeMap::new(),
            parent: Some(self),
        }
    }

    pub fn from_value(
        value: &'_ Value,
        parent: Option<&'a Context>,
    ) -> Result<Context<'a>, ContextError> {
        let mut context = Context {
            content: BTreeMap::new(),
            parent,
        };

        if let Value::Object(object) = value {
            for key in object.keys() {
                if !is_identifier(key) {
                    return Err(ContextError::InvalidKey { key: key.clone() });
                }
            }
            for (key, value) in object.iter() {
                context.insert(key, value.clone());
            }
        } else {
            return Err(ContextError::InvalidContextValue {
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

    pub fn to_rhai_scope(&'a self) -> Scope {
        let mut scope = Scope::new();

        self.iter().for_each(|(key, value)| {
            let dynamic = match rhai::serde::to_dynamic(value) {
                Ok(dynamic) => dynamic,
                Err(error) => {
                    panic!("Failed to convert context value to dynamic: {}", error);
                }
            };

            scope.push_constant(key.clone(), dynamic);
        });

        scope
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("top level keys of context must follow /[a-zA-Z_][a-zA-Z0-9_]*: `{key}`")]
    InvalidKey { key: String },
    #[error("context value is not an object: {:?}", value)]
    InvalidContextValue { value: Value },
}

fn is_identifier(identifier: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }
    RE.is_match(identifier)
}
