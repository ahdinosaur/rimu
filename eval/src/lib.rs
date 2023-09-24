mod block;
mod common;
mod expression;

pub use block::evaluate as evaluate_block;
pub use common::call;
pub use expression::evaluate as evaluate_expression;
pub use rimu_value::EvalError;

pub type Result<Value> = std::result::Result<Value, EvalError>;
