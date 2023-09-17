use std::fmt;

use rimu_ast::{SpannedBlock, SpannedExpression};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Function {
    Block {
        args: Vec<String>,
        body: SpannedBlock,
    },
    Expression {
        args: Vec<String>,
        body: SpannedExpression,
    },
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function")
    }
}
