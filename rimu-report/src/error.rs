use crate::SourceId;

pub trait ReportError {
    fn display<'a>(&self, source: &'a str, source_id: SourceId);
}
