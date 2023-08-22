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
        container_span: Span,
        index_span: Span,
        index: isize,
        length: usize,
    },
    #[error("key error, key: {key}, object: {object:?}")]
    KeyNotFound {
        key_span: Span,
        key: String,
        object_span: Span,
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
                "Eval: Environment error",
                vec![(span.clone(), format!("{}", source), Color::Cyan)],
                vec![],
            ),
            EvalError::MissingVariable { span, var } => (
                "Eval: Missing variable",
                vec![(
                    span.clone(),
                    format!("Not in environment: {}", var),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::CallNonFunction { span, expr } => (
                "Eval: Tried to call non-function",
                vec![(
                    span.clone(),
                    format!("Not a function: {}", expr),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::TypeError {
                span,
                expected,
                got,
            } => (
                "Eval: Unexpected type",
                vec![(
                    span.clone(),
                    format!("Expected: {}, got: {}", expected, got),
                    Color::Cyan,
                )],
                vec![],
            ),
            EvalError::IndexOutOfBounds {
                container_span,
                index,
                index_span,
                length,
            } => (
                "Eval: Index out of bounds",
                vec![
                    (
                        container_span.clone(),
                        format!("Length: {}", length),
                        Color::Cyan,
                    ),
                    (index_span.clone(), format!("Index: {}", index), Color::Cyan),
                ],
                vec![],
            ),
            EvalError::KeyNotFound {
                key_span,
                key,
                object_span,
                object,
            } => (
                "Eval: Key not found",
                vec![
                    (
                        object_span.clone(),
                        format!("Object: {}", Value::Object(object.clone())),
                        Color::Cyan,
                    ),
                    (key_span.clone(), format!("Key: {}", key), Color::Cyan),
                ],
                vec![],
            ),
            EvalError::RangeStartGreaterThanOrEqualToEnd { span, start, end } => (
                "Eval: Range start >= end",
                vec![(span.clone(), format!("{} >= {}", start, end), Color::Cyan)],
                vec![],
            ),
            EvalError::ErrorExpression { span } => (
                "Eval: Expression error",
                vec![(span.clone(), format!("Error"), Color::Cyan)],
                vec![],
            ),
        };

        let mut report = Report::build(
            ReportKind::Error,
            spans
                .first()
                .map(|s| s.0.source())
                .unwrap_or(source_id.clone()),
            spans.first().map(|s| s.0.end()).unwrap_or(0),
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
