use std::fmt;

use rimu_expr::SpannedExpression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: SpannedExpression,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
