use crate::{
    context::Context,
    value::{Object, Value},
};

use super::{interpolate::interpolate, Template, TemplateError};

pub(crate) fn evaluate(tmpl: &Template, context: &Context) -> Result<Value, TemplateError> {
    match tmpl {
        Template::Null => Ok(Value::Null),
        Template::Boolean(boolean) => Ok(Value::Boolean(boolean.clone())),
        Template::Number(number) => Ok(Value::Number(number.clone())),
        Template::String(string) => {
            let next_string = interpolate(string, context)?;
            Ok(Value::String(next_string))
        }
        Template::List(list) => {
            let next_list = list
                .into_iter()
                .filter_map(|item| match evaluate(item, context) {
                    Ok(Value::Null) => None,
                    Ok(tmpl) => Some(Ok(tmpl)),
                    Err(error) => Some(Err(error)),
                })
                .collect::<Result<Vec<Value>, TemplateError>>()?;
            Ok(Value::List(next_list))
        }
        Template::Object(object) => {
            let mut next_object = Object::new();
            for (key, tmpl) in object.iter() {
                match evaluate(tmpl, context)? {
                    Value::Null => {}
                    tmpl => {
                        next_object.insert(interpolate(key, context)?, tmpl);
                    }
                };
            }
            Ok(Value::Object(next_object))
        }
        Template::Operation(operation) => {
            let next_tmpl = operation.evaluate(context)?;
            evaluate(&next_tmpl, context)
        }
    }
}
