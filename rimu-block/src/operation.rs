use std::collections::BTreeMap;

use rimu_report::Spanned;

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

pub(crate) fn find_operator<Value>(object: BTreeMap<Spanned<String>, Value>) -> Option<String> {
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

/*
pub(crate) fn parse_operation(
    operator: String,
    object: Spanned<BTreeMap<Spanned<String>, Spanned<Block>>>,
) -> Result<Operation, CompilerError> {
    let (object, span) = object.take();
    match operator.as_str() {
        "$let" => {
            let variables = object.get("$let".into()).unwrap();
            let body = object
                .get("in".into())
                .ok_or_else(|| CompilerError::custom(span, "Expected value for key \"in\""));
            Spanned::new(span, Operation::Let { variables, body })
        }
    }
}
*/

pub(crate) fn unescape_non_operation_key(key: &str) -> &str {
    if key.starts_with("$$") {
        &key[1..]
    } else {
        &key[..]
    }
}
