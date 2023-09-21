use std::{cell::RefCell, fmt, rc::Rc};

use rimu_ast::{SpannedBlock, SpannedExpression};

use crate::Environment;

#[derive(Debug, Clone)]
pub struct Function {
    pub args: Vec<String>,
    pub body: FunctionBody,
    pub env: Rc<RefCell<Environment>>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.args.eq(&other.args) && self.body.eq(&other.body) && Rc::ptr_eq(&self.env, &other.env)
    }
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
