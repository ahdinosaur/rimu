use rimu_expr::{parse, SourceId};

use crate::{Environment, Evaluator, Object, RenderError, Template, Value};

pub struct Engine {}

impl Default for Engine {
    fn default() -> Self {
        Self {}
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, template: &Template, context: &Environment) -> Result<Value, RenderError> {
        match template {
            Template::Null => Ok(Value::Null),
            Template::Boolean(boolean) => Ok(Value::Boolean(boolean.clone())),
            Template::Number(number) => Ok(Value::Number(number.clone())),
            Template::String(string) => {
                let next_string = self.interpolate(string, context)?;
                Ok(Value::String(next_string))
            }
            Template::List(list) => {
                let next_list = list
                    .into_iter()
                    .filter_map(|item| match self.render(item, context) {
                        Ok(Value::Null) => None,
                        Ok(template) => Some(Ok(template)),
                        Err(error) => Some(Err(error)),
                    })
                    .collect::<Result<Vec<Value>, RenderError>>()?;
                Ok(Value::List(next_list))
            }
            Template::Object(object) => {
                let mut next_object = Object::new();
                for (key, template) in object.iter() {
                    match self.render(template, context)? {
                        Value::Null => {}
                        template => {
                            next_object.insert(self.interpolate(key, context)?, template);
                        }
                    };
                }
                Ok(Value::Object(next_object))
            }
            Template::Block(block) => block.render(&self, context),
        }
    }

    pub(crate) fn evaluate(&self, expr: &str, env: &Environment) -> Result<Value, RenderError> {
        let (expr, _errors) = parse(expr, SourceId::empty());
        let Some(expr) = expr else {
            todo!()
        };
        Ok(Evaluator::evaluate(&expr, env)?)
    }

    pub(crate) fn interpolate(
        &self,
        mut source: &str,
        context: &Environment,
    ) -> Result<String, RenderError> {
        // shortcut the common no-interpolation case
        if source.find('$') == None {
            return Ok(source.into());
        }

        let mut result = String::new();

        while source.len() > 0 {
            let Some(start) = source.find('$') else {
                // remainder of the string contains no interpolations..
                result.push_str(source);
                break;
            };

            // If this is an un-escaped `${`, interpolate..
            if let Some("${") = source.get(start..start + 2) {
                result.push_str(&source[..start]);

                let Some(end) = source.find("}") else {
                    return Err(RenderError::UnterminatedInterpolation { src: source.to_string() });
                };

                let var_str = &source[start + 2..end].trim();
                let var_path: Vec<&str> = var_str.split(".").collect();

                let Some(value) = context.get_in(var_path) else {
                    return Err(RenderError::MissingEnvironment {
                        var: var_str.to_string(),
                    });
                };

                match value {
                    // null interpolates to an empty string
                    Value::Null => {}
                    Value::Number(n) => result.push_str(&n.to_string()),
                    Value::Boolean(true) => result.push_str("true"),
                    Value::Boolean(false) => result.push_str("false"),
                    Value::String(s) => result.push_str(&s),
                    Value::List(_) | Value::Object(_) | Value::Function(_) => {
                        return Err(RenderError::InvalidValueInterpolation {
                            var: var_str.to_string(),
                            value: value.clone(),
                        });
                    }
                }

                source = &source[end + 1..];
                continue;
            }

            // If this is an escape (`$${`), un-escape it
            if let Some("$${") = source.get(start..start + 3) {
                result.push_str(&source[..start + 1]);
                source = &source[start + 2..];
                continue;
            }

            // otherwise, carry on..
            result.push_str(&source[..start + 1]);
            source = &source[start + 1..];
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Object, Value};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_interpolate() -> Result<(), RenderError> {
        let content = "one ${ two } three ${ four.five }";

        let mut context = Environment::new();
        context.insert("two", Value::String("2".into()));
        context.insert(
            "four",
            Value::Object({
                let mut object = Object::new();
                object.insert("five".into(), Value::String("9".into()));
                object
            }),
        );

        let engine = Engine::default();
        let result = engine.interpolate(content, &context)?;

        assert_eq!(result, "one 2 three 9".to_string());

        Ok(())
    }
}
