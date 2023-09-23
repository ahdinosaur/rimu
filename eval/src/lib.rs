mod block;
mod common;
mod error;
mod expression;

pub use block::evaluate as evaluate_block;
pub use error::EvalError;
pub use expression::evaluate as evaluate_expression;

pub type Result<Value> = std::result::Result<Value, EvalError>;
