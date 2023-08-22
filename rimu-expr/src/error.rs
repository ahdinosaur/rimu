// with help from
// - https://github.com/Egggggg/plum/blob/e9153c6cf9586d033a777cdaa28ad2a8cd95bcf3/src/error.rs#L70

use ariadne::{Config, Label, Report, ReportKind, Source};
use rimu_report::{ReportError, SourceId};

use crate::{CompilerError, LexerError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lexer(LexerError),
    Compiler(CompilerError),
}

impl ReportError for Error {
    fn display<'a>(&self, source: &'a str, source_id: SourceId) {
        let (msg, spans, notes) = match self {
            Error::Lexer(error) => (
                "Lexer: Unexpected character",
                vec![(error.span(), format!("{}", error), ariadne::Color::Blue)],
                if let Some(e) = error.label() {
                    vec![format!("Label is `{}`", e)]
                } else {
                    vec![]
                },
            ),
            Error::Compiler(error) => (
                "Compiler: Unexpected token",
                vec![(error.span(), format!("{}", error), ariadne::Color::Green)],
                if let Some(e) = error.label() {
                    vec![format!("Label is `{}`", e)]
                } else {
                    vec![]
                },
            ),
        };

        let mut report = Report::build(
            ReportKind::Error,
            spans
                .first()
                .map(|s| s.0.source())
                .unwrap_or(source_id.clone()),
            spans.first().map(|s| s.0.start()).unwrap_or(0),
        )
        .with_message(msg);

        for (i, (span, msg, color)) in spans.into_iter().enumerate() {
            report = report.with_label(
                Label::new(span)
                    .with_message(msg)
                    .with_order(i as i32)
                    .with_color(color),
            );
        }

        for note in notes {
            report = report.with_note(note);
        }

        report
            .with_config(Config::default().with_compact(false))
            .finish()
            .eprint((source_id, Source::from(source)))
            .unwrap();
    }
}
