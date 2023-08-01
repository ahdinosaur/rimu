mod eval;

use serde::{de::value::MapDeserializer, Deserialize};

pub use self::eval::EvalOperation;
use crate::{Context, Engine, Object, ParseError, RenderError, Template};

pub trait Operation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Template, RenderError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operations {
    Eval(EvalOperation),
}

impl Operations {
    pub(crate) fn render(
        &self,
        engine: &Engine,
        context: &Context,
    ) -> Result<Template, RenderError> {
        match self {
            Operations::Eval(eval) => eval.render(engine, context),
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
