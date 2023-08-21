pub trait ReportError {
    fn display<'a>(&self, code: &'a str);
}
