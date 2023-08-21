// with help from
// - https://github.com/Egggggg/plum/blob/e9153c6cf9586d033a777cdaa28ad2a8cd95bcf3/src/error.rs#L70

use ariadne::{Label, Report, Source};
use rimu_report::ReportError;

use crate::{CompilerError, LexerError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lexer(LexerError),
    Compiler(CompilerError),
}

impl ReportError for Error {
    fn display<'a>(&self, code: &'a str) {
        match self {
            Error::Lexer(error) => {
                let span = error.span();
                Report::build(ariadne::ReportKind::Error, span.source(), span.start())
                    .with_code(1)
                    .with_message("SyntaxError: Unexpected token")
                    .with_label(
                        Label::new((span.source(), span.range()))
                            .with_message(format!("{}", error))
                            .with_color(ariadne::Color::Green),
                    )
                    .with_note(if let Some(e) = error.label() {
                        format!("Label is `{}`", e)
                    } else {
                        "No label".to_owned()
                    })
                    .finish()
                    .eprint((span.source(), Source::from(code)))
                    .unwrap();
            }
            Error::Compiler(error) => {
                let span = error.span();
                Report::build(ariadne::ReportKind::Error, span.source(), span.start())
                    .with_code(1)
                    .with_message("SyntaxError: Unexpected token")
                    .with_label(
                        Label::new((span.source(), span.range()))
                            .with_message(format!("{}", error))
                            .with_color(ariadne::Color::Green),
                    )
                    .with_note(if let Some(e) = error.label() {
                        format!("Label is `{}`", e)
                    } else {
                        "No label".to_owned()
                    })
                    .finish()
                    .eprint((span.source(), Source::from(code)))
                    .unwrap();
            }
        }
    }
}
