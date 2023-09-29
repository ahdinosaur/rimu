use indexmap::IndexMap;
use std::{cell::RefCell, iter::empty, rc::Rc};

use crate::serde::{value_get_in, SerdeValue, SerdeValueObject};

#[derive(Debug, Clone)]
pub struct Environment {
    content: SerdeValueObject,
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
                value: value.clone(),
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
        V: Into<SerdeValue>,
    {
        self.content.insert(k.into(), v.into());
    }

    pub fn get(&self, key: &str) -> Option<SerdeValue> {
        match self.content.get(key) {
            Some(value) => Some(value.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().get(key),
                None => None,
            },
        }
    }

    pub fn get_in(&self, keys: Vec<&str>) -> Option<SerdeValue> {
        let Some((first, rest)) = keys.split_first() else {
            return None;
        };
        match self.get(first) {
            Some(value) => value_get_in(&value, rest).cloned(),
            None => None,
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (String, SerdeValue)>> {
        let parent_iter = match &self.parent {
            Some(parent) => parent.borrow().iter(),
            None => Box::new(empty()),
        };
        let self_iter = self.content.clone().into_iter();
        Box::new(parent_iter.chain(self_iter))
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum EnvironmentError {
    #[error("context value is not an object: {:?}", value)]
    InvalidEnvironmentValue { value: SerdeValue },
}
