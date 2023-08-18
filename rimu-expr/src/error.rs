// with help from
// - https://github.com/Egggggg/plum/blob/e9153c6cf9586d033a777cdaa28ad2a8cd95bcf3/src/error.rs#L70

use crate::{CompilerError, LexerError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lexer(LexerError),
    Compiler(CompilerError),
}
