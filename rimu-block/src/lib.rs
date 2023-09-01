use compiler::{compile, CompilerError, SpannedBlock};
use lexer::{tokenize, LexerError};
use rimu_report::{SourceId, Span};

mod compiler;
mod lexer;

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
