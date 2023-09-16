mod block;
mod environment;
mod error;
mod expression;

pub use block::evaluate as evaluate_block;
pub use environment::{Environment, EnvironmentError};
pub use error::EvalError;
pub use expression::evaluate as evaluate_expression;
