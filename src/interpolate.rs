use lazy_static::lazy_static;
use regex::Regex;

use crate::{context::Context, template::TemplateError};

// With help from: https://github.com/hasezoey/new_string_template/blob/master/src/template.rs
pub(crate) fn interpolate(source: &str, context: &Context) -> Result<String, TemplateError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{\{\s*(\w+(?:\.\w+)*)\s*\}\}").unwrap();
    }

    let matches: Vec<regex::Captures<'_>> = RE.captures_iter(source).collect();

    if matches.is_empty() {
        return Ok(source.to_string());
    }

    let mut parts: Vec<String> = Vec::with_capacity(matches.len());
    // Save last index of an match, starting with "0"
    let mut last_index: usize = 0;

    for entry in &matches {
        let full_match = entry.get(0).expect("Match Index 0 was None (Full Match)");
        let var_match = entry.get(1).expect("Match Index 1 was None (Value Match)");

        let full_match_start = full_match.start();
        let full_match_end = full_match.end();

        let var_match_start = var_match.start();
        let var_match_end = var_match.end();

        parts.push(source[last_index..full_match_start].to_owned()); // non-inclusive to only copy up-to just before the starting character of the beginning of the match

        let var_str = &source[var_match_start..var_match_end]; // non-inclusive because regex's "end" referes to the character after the match
        let var_path: Vec<&str> = var_str.split(".").collect();

        // not using "unwrap_or_else" because of the need to return "Err"
        if let Some(v) = context.get_in(var_path) {
            parts.push(v.to_string());
        } else {
            return Err(TemplateError::MissingContext {
                var: var_str.to_string(),
            });
        }

        last_index = full_match_end;
    }

    // if string is not already fully copied, copy the rest of it
    if last_index < source.len() {
        parts.push(source[last_index..source.len()].to_owned()); // non-inclusive because "len" is last index + 1
    }

    return Ok(parts.join(""));
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::value::{Object, Value};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_interpolate() -> Result<(), TemplateError> {
        let content = "one {{ two }} three {{ four.five }}";

        let mut context = Context::new();
        context.insert("two", Value::String("2".into()));
        context.insert(
            "four",
            Value::Object({
                let mut object = Object::new();
                object.insert("five".into(), Value::String("9".into()));
                object
            }),
        );

        let result = interpolate(content, &context)?;

        assert_eq!(result, "one 2 three 9".to_string());

        Ok(())
    }
}
