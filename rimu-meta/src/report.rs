use ariadne::{Config, Label, Report, ReportKind, Source};
use serde::{Deserialize, Serialize};

use crate::{SourceId, Span};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub span: Span,
    pub message: String,
    pub labels: Vec<(Span, String)>,
    pub notes: Vec<String>,
}

impl ErrorReport {
    pub fn display(&self, source: &str, source_id: SourceId) {
        let mut report = Report::build(ReportKind::Error, self.span.source(), self.span.end())
            .with_message(self.message.clone());

        for (i, (span, msg)) in self.labels.clone().into_iter().enumerate() {
            report = report.with_label(Label::new(span).with_message(msg).with_order(i as i32));
        }

        for note in &self.notes {
            report = report.with_note(note);
        }

        report
            .with_config(Config::default().with_compact(false))
            .finish()
            .eprint((source_id.clone(), Source::from(source)))
            .unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReports {
    pub reports: Vec<ErrorReport>,
}

impl From<Vec<ErrorReport>> for ErrorReports {
    fn from(reports: Vec<ErrorReport>) -> Self {
        ErrorReports { reports }
    }
}
