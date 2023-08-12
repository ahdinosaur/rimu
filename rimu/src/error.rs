use rhai::EvalAltResult;

use crate::{context::ContextError, Value, ValueError};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unknown block key: {block_key}")]
    UnknownBlockKey { block_key: String },
    #[error("too many block keys")]
    TooManyBlockKeys,
    #[error("value error: {0}")]
    Value(#[from] ValueError),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("rhai eval error: {0}")]
    RhaiEval(#[from] Box<EvalAltResult>),
    #[error("missing context: {var}")]
    MissingContext { var: String },
    #[error("unterminated interpolation: {src}")]
    UnterminatedInterpolation { src: String },
    #[error("interpolation of '{var}' produced an array or object: {value}")]
    ListOrObjectInterpolation { var: String, value: Value },
    #[error("context error: {0}")]
    ContextError(#[from] ContextError),
}
