use std::collections::BTreeMap;

use rimu_report::{Span, Spanned};

use crate::{
    block::{Block, SpannedBlock},
    compiler::CompilerError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    Let {
        // #[serde(rename = "$let")]
        variables: SpannedBlock,
        // #[serde(rename = "in")]
        body: SpannedBlock,
    },
    If {
        // #[serde(rename = "$if")]
        condition: SpannedBlock,
        // #[serde(rename = "then")]
        consequent: Option<SpannedBlock>,
        // #[serde(rename = "else")]
        alternative: Option<SpannedBlock>,
    },
}

pub(crate) fn find_operator<Value>(object: &BTreeMap<Spanned<String>, Value>) -> Option<String> {
    for key in object.keys() {
        let key = key.inner();
        let mut chars = key.chars();
        let is_op = chars.next() == Some('$') && chars.next() != Some('$');
        if is_op {
            return Some(key.clone());
        }
    }
    None
}

pub(crate) fn parse_operation(
    operator: String,
    object: BTreeMap<Spanned<String>, Spanned<Block>>,
    span: Span,
) -> Result<Operation, CompilerError> {
    let object = BTreeMap::from_iter(
        object
            .into_iter()
            .map(|(key, value)| (key.into_inner(), value)),
    );
    let operation = match operator.as_str() {
        "$if" => {
            static KEYS: [&str; 3] = ["$if", "then", "else"];
            check_operation_keys(span, &KEYS, &object)?;

            let condition = object.get("$if").unwrap().to_owned();
            let consequent = object.get("then").cloned();
            let alternative = object.get("else").cloned();

            Operation::If {
                condition,
                consequent,
                alternative,
            }
        }
        "$let" => {
            static KEYS: [&str; 2] = ["$let", "in"];
            check_operation_keys(span.clone(), &KEYS, &object)?;

            let variables = object.get("$let").unwrap().to_owned();
            let body = object
                .get("in")
                .ok_or_else(|| CompilerError::custom(span, "$let: missing field \"in\""))?
                .to_owned();
            Operation::Let { variables, body }
        }
        &_ => todo!(),
    };
    Ok(operation)
}

fn check_operation_keys<Value>(
    span: Span,
    keys: &[&str],
    object: &BTreeMap<String, Value>,
) -> Result<(), CompilerError> {
    for key in object.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(CompilerError::custom(
                span,
                format!("$if: unexpected field \"{}\"", key),
            ));
        }
    }
    Ok(())
}

pub(crate) fn unescape_non_operation_key(key: &str) -> &str {
    if key.starts_with("$$") {
        &key[1..]
    } else {
        key
    }
}
