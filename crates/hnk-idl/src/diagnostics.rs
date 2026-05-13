use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: Option<PathBuf>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticSet {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticSet {
    pub fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn singleton(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

impl IntoIterator for DiagnosticSet {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl Diagnostic {
    pub fn new(
        severity: Severity,
        code: &'static str,
        message: impl Into<String>,
        location: Option<SourceLocation>,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            location,
            suggestion,
        }
    }

    pub fn parse_error(
        code: &'static str,
        path: Option<&Path>,
        message: impl Into<String>,
        line: Option<usize>,
        column: Option<usize>,
        suggestion: Option<String>,
    ) -> Self {
        let location = match (line, column) {
            (Some(line), Some(column)) => Some(SourceLocation {
                path: path.map(Path::to_path_buf),
                line,
                column,
            }),
            _ => None,
        };

        Self::new(Severity::Error, code, message, location, suggestion)
    }
}
