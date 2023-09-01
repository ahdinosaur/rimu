use std::collections::BTreeMap;

use rimu_report::Spanned;

use crate::operation::Operation;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Block {
    Object(BTreeMap<Spanned<String>, SpannedBlock>),
    List(Vec<SpannedBlock>),
    Expression(String),
    Operation(Box<Operation>),
}

pub type SpannedBlock = Spanned<Block>;
