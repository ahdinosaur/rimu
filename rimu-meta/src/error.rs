use ariadne::{Config, Label, Report, ReportKind, Source};
use serde::{Deserialize, Serialize};

use crate::{SourceId, Span};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport<'a> {
    source: &'a str,
    source_id: SourceId,
    message: String,
    spans: Vec<(Span, String)>,
    notes: Vec<String>,
}

impl ErrorReport<'_> {
    pub fn display(&self) {
        let mut report = Report::build(
            ReportKind::Error,
            self.spans
                .first()
                .map(|s| s.0.source())
                .unwrap_or(self.source_id.clone()),
            self.spans.first().map(|s| s.0.end()).unwrap_or(0),
        )
        .with_message(self.message.clone());

        for (i, (span, msg)) in self.spans.clone().into_iter().enumerate() {
            report = report.with_label(Label::new(span).with_message(msg).with_order(i as i32));
        }

        for note in &self.notes {
            report = report.with_note(note);
        }

        report
            .with_config(Config::default().with_compact(false))
            .finish()
            .eprint((self.source_id.clone(), Source::from(self.source)))
            .unwrap();
    }
}
