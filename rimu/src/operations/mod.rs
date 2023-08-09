mod eval;
mod let_;

use serde::{de::value::MapDeserializer, Deserialize};

pub use self::eval::EvalOperation;
pub use self::let_::LetOperation;
use crate::{Context, Engine, Object, ParseError, RenderError, Value};

pub trait Operation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Value, RenderError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operations {
    Eval(EvalOperation),
    Let(LetOperation),
}

impl Operations {
    pub(crate) fn render(&self, engine: &Engine, context: &Context) -> Result<Value, RenderError> {
        match self {
            Operations::Eval(op) => op.render(engine, context),
            Operations::Let(op) => op.render(engine, context),
        }
    }
}

pub(crate) fn find_operator(object: &Object) -> Result<Option<String>, ParseError> {
    let operators: Vec<&String> = object
        .keys()
        .filter(|key| {
            let mut chars = key.chars();
            chars.next() == Some('$') && chars.next() != Some('$')
        })
        .collect();
    if operators.len() > 1 {
        Err(ParseError::TooManyOperators)
    } else if operators.len() == 1 {
        Ok(Some(operators[0].to_owned()))
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_operation(operator: &str, object: &Object) -> Result<Operations, ParseError> {
    let map_de = MapDeserializer::new(object.clone().into_iter());
    match operator {
        "$eval" => Ok(Operations::Eval(EvalOperation::deserialize(map_de)?)),
        "$let" => Ok(Operations::Let(LetOperation::deserialize(map_de)?)),
        _ => Err(ParseError::UnknownOperator {
            operator: operator.to_owned(),
        }),
    }
}

pub(crate) fn unescape_non_operation_key(key: &str) -> &str {
    if key.starts_with("$$") {
        &key[1..]
    } else {
        &key[..]
    }
}
