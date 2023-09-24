use indexmap::IndexMap;
use rimu_meta::Spanned;

use crate::{Function, Number, Object, Value};

pub type SpannedList = Vec<SpannedValue>;
pub type SpannedObject = IndexMap<String, SpannedValue>;

#[derive(Default, Clone, PartialEq)]
pub enum SpannedValueInner {
    #[default]
    Null,
    Boolean(bool),
    String(String),
    Number(Number),
    Function(Function),
    List(SpannedList),
    Object(SpannedObject),
}

pub type SpannedValue = Spanned<SpannedValueInner>;

impl From<SpannedValue> for Value {
    fn from(value: SpannedValue) -> Self {
        value.into_inner().into()
    }
}

impl From<SpannedValueInner> for Value {
    fn from(value: SpannedValueInner) -> Self {
        match value {
            SpannedValueInner::Null => Value::Null,
            SpannedValueInner::Boolean(boolean) => Value::Boolean(boolean),
            SpannedValueInner::String(string) => Value::String(string),
            SpannedValueInner::Number(number) => Value::Number(number),
            SpannedValueInner::Function(function) => Value::Function(function),
            SpannedValueInner::List(list) => {
                Value::List(list.iter().map(|item| item.clone().into()).collect())
            }
            SpannedValueInner::Object(object) => Value::Object(Object::from_iter(
                object
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone().into()))
                    .collect::<Vec<_>>(),
            )),
        }
    }
}
