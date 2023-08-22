use ariadne::{Color, Config, Label, Report, ReportKind, Source};
use rimu_env::EnvironmentError;
use rimu_expr::Expression;
use rimu_report::{ReportError, Span};
use rimu_value::{Object, Value};

#[derive(Debug, thiserror::Error, Clone, PartialEq, PartialOrd)]
pub enum EvalError {
    #[error("{source}")]
    Environment {
        span: Span,
        #[source]
        source: EnvironmentError,
    },
    #[error("missing variable: {var}")]
    MissingVariable { span: Span, var: String },
    #[error("tried to call non-function: {expr}")]
    CallNonFunction { span: Span, expr: Expression },
    #[error("type error, expected: {expected}, got: {got}")]
    TypeError {
        span: Span,
        expected: String,
        got: Value,
    },
    #[error("index out of bounds, index: {index}, length: {length}")]
    IndexOutOfBounds {
        span: Span,
        index: isize,
        length: usize,
    },
    #[error("key error, key: {key}, object: {object:?}")]
    KeyNotFound {
        span: Span,
        key: String,
        object: Object,
    },
    #[error("range start >= end, start: {start}, end: {end}")]
    RangeStartGreaterThanOrEqualToEnd {
        span: Span,
        start: usize,
        end: usize,
    },
    #[error("error expression")]
    ErrorExpression { span: Span },
}

impl ReportError for EvalError {
    fn display<'a>(&self, source: &'a str, source_id: rimu_report::SourceId) {
        let (msg, spans, notes): (&str, Vec<(Span, String, Color)>, Vec<String>) = match self {
            EvalError::Environment { span, source } => (
                "EvalError",
                vec![(span.clone(), format!("{}", source), Color::Cyan)],
                vec![],
            ),
            EvalError::MissingVariable { span, var } => (
                "EvalError",
                vec![(
                    span.clone(),
                    format!("Missing variable: {}", var),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::CallNonFunction { span, expr } => (
                "EvalError",
                vec![(
                    span.clone(),
                    format!("Tried to call non-function: {}", expr),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::TypeError {
                span,
                expected,
                got,
            } => (
                "EvalError",
                vec![(
                    span.clone(),
                    format!("Type error, expected: {}, got: {}", expected, got),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::IndexOutOfBounds {
                span,
                index,
                length,
            } => (
                "EvalError",
                vec![(
                    span.clone(),
                    format!("Index out of bounds, index: {}, length: {}", index, length),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::KeyNotFound { span, key, object } => (
                "EvalError",
                vec![(
                    span.clone(),
                    format!(
                        "Key not found, key: {}, object: {}",
                        key,
                        Value::Object(object.clone())
                    ),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::RangeStartGreaterThanOrEqualToEnd { span, start, end } => (
                "EvalError",
                vec![(
                    span.clone(),
                    format!("Range start >= end, start: {}, end: {}", start, end),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::ErrorExpression { span } => (
                "EvalError",
                vec![(span.clone(), format!("Expression error"), Color::Cyan)],
                vec![],
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
