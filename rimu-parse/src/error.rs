use rimu_meta::ErrorReport;

use crate::compiler::CompilerError;
use crate::lexer::lines::LinesLexerError;
use crate::lexer::LexerError;

#[derive(Debug)]
pub enum Error {
    Lexer(LexerError),
    Compiler(CompilerError),
}

impl From<Error> for ErrorReport {
    fn from(value: Error) -> Self {
        match value {
            Error::Lexer(LexerError::Lines(error)) => match error {
                LinesLexerError::InconsistentLeadingWhitespace {
                    span,
                    found,
                    expected,
                } => ErrorReport {
                    message: "Lexer: Inconsistent leading whitespace".into(),
                    span: span.clone(),
                    labels: vec![(
                        span.clone(),
                        format!(
                            "Found {} spaces, expected one of {} spaces.",
                            found,
                            expected
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<String>>()
                                .join(",")
                        ),
                    )],
                    notes: vec![],
                },
            },
            Error::Lexer(LexerError::Line(error)) => ErrorReport {
                message: "Lexer: Unexpected character".into(),
                span: error.span(),
                labels: vec![(error.span(), format!("{}", error))],
                notes: if let Some(e) = error.label() {
                    vec![format!("Label is `{}`", e)]
                } else {
                    vec![]
                },
            },
            Error::Compiler(error) => ErrorReport {
                message: "Compiler: Unexpected token".into(),
                span: error.span(),
                labels: vec![(error.span(), format!("{}", error))],
                notes: if let Some(e) = error.label() {
                    vec![format!("Label is `{}`", e)]
                } else {
                    vec![]
                },
            },
        }
    }
}
