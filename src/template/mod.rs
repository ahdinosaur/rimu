use std::collections::BTreeMap;

use rhai::EvalAltResult;
use serde::{de::value::MapDeserializer, Deserialize};

use crate::{
    operations::{find_operator, parse_operation, unescape_non_operation, Operations},
    value::{Number, Value, ValueError},
};

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("unknown operator: {}", operator)]
    UnknownOperator { operator: String },
    #[error("too many operators")]
    TooManyOperators,
    #[error("invalid operation: {:?}", template)]
    InvalidOperation { template: Template },
    #[error("missing context: {var}")]
    MissingContext { var: String },
    #[error("value error: {0}")]
    Value(#[from] ValueError),
    #[error("rhai eval error: {0}")]
    RhaiEval(#[from] Box<EvalAltResult>),
}

pub(crate) type List = Vec<Template>;
pub(crate) type Object = BTreeMap<String, Template>;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "Value")]
pub enum Template {
    Null,
    Boolean(bool),
    String(String),
    Number(Number),
    List(List),
    Object(Object),
    Operation(Operations),
}

impl TryFrom<Value> for Template {
    type Error = TemplateError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(Template::Null),
            Value::Boolean(boolean) => Ok(Template::Boolean(boolean)),
            Value::Number(number) => Ok(Template::Number(number)),
            Value::String(string) => Ok(Template::String(string)),
            Value::List(list) => {
                let next_list: Vec<Template> = list
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<Vec<Template>, Self::Error>>()?;
                Ok(Template::List(next_list))
            }
            Value::Object(object) => {
                if let Some(operator) = find_operator(object.clone())? {
                    return Ok(Template::Operation(parse_operation(&operator, &object)?));
                }

                let object = unescape_non_operation(object.clone());

                let mut next_object = BTreeMap::new();
                for (key, value) in object.into_iter() {
                    next_object.insert(key, value.try_into()?);
                }
                Ok(Template::Object(next_object))
            }
        }
    }
}
