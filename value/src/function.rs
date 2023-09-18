use std::fmt;

use rimu_ast::{SpannedBlock, SpannedExpression};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Function {
    pub args: Vec<String>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum FunctionBody {
    Block(SpannedBlock),
    Expression(SpannedExpression),
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function")
    }
}
