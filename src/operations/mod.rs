use serde::{de::value::MapDeserializer, Deserialize};

use crate::{Context, Engine, Object, Template, TemplateError, Value};

pub(crate) trait Operation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Template, TemplateError>;
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
    ) -> Result<Template, TemplateError> {
        match self {
            Operations::Eval(eval) => eval.render(engine, context),
        }
    }
}

pub(crate) fn find_operator(object: &Object) -> Result<Option<String>, TemplateError> {
    let operators: Vec<&String> = object
        .keys()
        .filter(|key| {
            let mut chars = key.chars();
            chars.next() == Some('$') && chars.next() != Some('$')
        })
        .collect();
    if operators.len() > 1 {
        Err(TemplateError::TooManyOperators)
    } else if operators.len() == 1 {
        Ok(Some(operators[0].to_owned()))
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_operation(
    operator: &str,
    object: &Object,
) -> Result<Operations, TemplateError> {
    let map_de = MapDeserializer::new(object.clone().into_iter());
    match operator {
        "$eval" => Ok(Operations::Eval(EvalOperation::deserialize(map_de)?)),
        _ => Err(TemplateError::UnknownOperator {
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

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct EvalOperation {
    #[serde(alias = "$eval")]
    pub expr: String,
}

impl Operation for EvalOperation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Template, TemplateError> {
        let value: Template = engine.evaluate(&self.expr, context)?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;

    use crate::{
        context::Context,
        template::Template,
        value::{Number, Value},
        Engine,
    };

    #[test]
    fn eval() -> Result<(), Box<dyn Error>> {
        let content = r#"
zero:
  $eval: one + 2
three:
  four: five
"#;
        let template: Template = serde_yaml::from_str(content)?;

        let engine = Engine::default();
        let mut context = Context::new();
        context.insert("one", Value::Number(Number::Signed(98)));

        let actual: Value = engine.render(&template, &context)?;

        let expected: Value = Value::Object(btree_map! {
            "zero".into() => Value::Number(100.into()),
            "three".into() => Value::Object(btree_map! {
                "four".into() => Value::String("five".into())
            })
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
