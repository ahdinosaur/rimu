pub(crate) mod evaluate;
pub(crate) mod interpolate;
pub(crate) mod operations;

use std::collections::BTreeMap;

use rhai::EvalAltResult;
use serde::{de::value::MapDeserializer, Deserialize};

use self::{evaluate::evaluate, operations::Operations};
use crate::{
    value::{Number, Value, ValueError},
    Context,
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

impl Template {
    pub fn evaluate(&self, context: &Context) -> Result<Value, TemplateError> {
        evaluate(self, context)
    }
}

impl TryFrom<Value> for Template {
    type Error = ValueError;

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
                if let Some(Value::String(typ)) = object.get("type") {
                    if typ.starts_with("op.") {
                        // https://github.com/serde-rs/serde/issues/1739#issuecomment-585442986
                        let operation =
                            Operations::deserialize(MapDeserializer::new(object.into_iter()))?;
                        return Ok(Template::Operation(operation));
                    }
                }

                let mut next_object = BTreeMap::new();
                for (key, value) in object.into_iter() {
                    next_object.insert(key, value.try_into()?);
                }
                Ok(Template::Object(next_object))
            }
        }
    }
}
