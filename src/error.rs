use rhai::EvalAltResult;

use crate::ValueError;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unknown operator: {}", operator)]
    UnknownOperator { operator: String },
    #[error("too many operators")]
    TooManyOperators,
    #[error("value error: {0}")]
    Value(#[from] ValueError),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("rhai eval error: {0}")]
    RhaiEval(#[from] Box<EvalAltResult>),
    #[error("missing context: {var}")]
    MissingContext { var: String },
}
