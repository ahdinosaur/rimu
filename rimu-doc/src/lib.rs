// maybe this shouldn't use chumsky.
//
// state
// - current indentation
//
// parse each line
//   - get indentation (relative to current)
//   - get type
//     - just use regexes
//     - if has ":" (not inside string), then object entry
//     - if starts with "-", then list item
//
// tokens:
// - indent
// - dedent
// - key
// - value
// - list item
//
// doc:
// - object
// - list
// - expression: string

use compiler::{compile, CompilerError, SpannedDoc};
use lexer::{tokenize, LexerError};
use rimu_report::{SourceId, Span};

mod compiler;
mod lexer;

pub enum Error {
    Lexer(LexerError),
    Compiler(CompilerError),
}

pub fn parse(code: &str, source: SourceId) -> (Option<SpannedDoc>, Vec<Error>) {
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
