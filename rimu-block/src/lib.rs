//
// TODO
//
// - Block replaces rimu Template
// - block Token is same as expr token, plus Indent, Dedent, and EndOfLine
// - block line lexer uses expr lexer
// - block parser uses expr parser
// - handle operations in chumsky with chumsky::try_map and rimu's existing operation parse code.
// - also be sure to keep TryFrom<Value> for Block, like Template had
//
// ----
//
// OR...
//
// maybe we don't need to embed lexers and parsers. just use in the outer tokenize code.
//
//
// ---
//
// then later
//
// - rimu-eval evaluates either block or expr
//

use rimu_report::{SourceId, Span};

use crate::block::{Block, SpannedBlock};
use crate::compiler::{compile, CompilerError};
use crate::lexer::{tokenize, LexerError};

mod block;
mod compiler;
mod lexer;
mod operation;

pub enum Error {
    Lexer(LexerError),
    Compiler(CompilerError),
}

pub fn parse(code: &str, source: SourceId) -> (Option<SpannedBlock>, Vec<Error>) {
    let mut errors = Vec::new();

    let len = code.chars().count();
    let eoi = Span::new(source.clone(), len, len);

    let (tokens, lex_errors) = tokenize(code, source.clone());
    errors.append(&mut lex_errors.into_iter().map(Error::Lexer).collect());

    let Some(tokens) = tokens else {
        return (None, errors);
    };

    let (output, compile_errors) = compile(tokens, eoi);
    errors.append(&mut compile_errors.into_iter().map(Error::Compiler).collect());

    (output, errors)
}
