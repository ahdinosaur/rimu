use std::collections::BTreeMap;

use rimu_expr::Expression;
use rimu_report::Spanned;

use crate::operation::Operation;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Block {
    Object(Vec<(Spanned<String>, SpannedBlock)>),
    List(Vec<SpannedBlock>),
    Expression(Expression),
    Operation(Box<Operation>),
}

pub type SpannedBlock = Spanned<Block>;
