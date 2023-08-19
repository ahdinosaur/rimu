use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    blocks::{find_block_key, parse_block, unescape_non_block_key, Blocks},
    Number, ParseError, Value,
};

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
    Block(Blocks),
}

impl TryFrom<Value> for Template {
    type Error = ParseError;

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
                if let Some(block_key) = find_block_key(&object)? {
                    return Ok(Template::Block(parse_block(&block_key, &object)?));
                }

                let mut next_object = BTreeMap::new();
                for (key, value) in object.into_iter() {
                    let key = unescape_non_block_key(&key).to_owned();
                    next_object.insert(key, value.try_into()?);
                }
                Ok(Template::Object(next_object))
            }
            Value::Function(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use std::error::Error;

    use crate::{
        blocks::{Blocks, EvalBlock},
        Template, Value,
    };

    #[test]
    fn parse_template() -> Result<(), Box<dyn Error>> {
        let content = r#"
zero:
  $eval: one + 2
three:
  four:
    - five
    - six
"#;

        let expected = Template::Object(btree_map! {
            "zero".into() => Template::Block(
                Blocks::Eval(EvalBlock {
                    expr: "one + 2".into()
                })
            ),
            "three".into() => Template::Object(btree_map! {
                "four".into() => Template::List(vec![
                    Template::String("five".into()),
                    Template::String("six".into())
                ])
            })
        });

        let actual: Template = serde_yaml::from_str(content)?;
        assert_eq!(expected, actual);

        let value: Value = serde_yaml::from_str(content)?;
        let actual: Template = value.try_into()?;
        assert_eq!(expected, actual);

        Ok(())
    }
}
