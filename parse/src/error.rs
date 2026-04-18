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
            Error::Lexer(LexerError::Line(error)) => {
                let span = error.span().clone();
                let mut labels = vec![(span.clone(), error.reason().to_string())];
                for (context, context_span) in error.contexts() {
                    labels.push((context_span.clone(), format!("while parsing {}", context)));
                }
                ErrorReport {
                    message: "Lexer: Unexpected character".into(),
                    span,
                    labels,
                    notes: vec![],
                }
            }
            Error::Compiler(error) => {
                let span = error.span().clone();
                let mut labels = vec![(span.clone(), error.reason().to_string())];
                for (context, context_span) in error.contexts() {
                    labels.push((context_span.clone(), format!("while parsing {}", context)));
                }
                ErrorReport {
                    message: "Compiler: Unexpected token".into(),
                    span,
                    labels,
                    notes: vec![],
                }
            }
        }
    }
}
