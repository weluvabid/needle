use serde::Serialize;

use crate::SourceSpan;

/// Stable diagnostic categories emitted by the frontend and compiler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    /// A character is not part of Needle's syntax.
    UnknownCharacter,
    /// A command name is not recognized.
    UnknownCommand,
    /// A numeric count is malformed or too large.
    InvalidCount,
    /// A number appears where a command was expected.
    UnexpectedNumber,
    /// A loop has no closing `stitch`.
    UnclosedLoop,
    /// A `stitch` has no opening `loop`.
    UnmatchedStitch,
    /// Source exceeds the supported size.
    SourceTooLarge,
    /// Generated Brainfuck exceeds the supported size.
    GeneratedProgramTooLarge,
    /// A compiler allocation could not be reserved.
    AllocationFailed,
}

/// Diagnostic severity. Current frontend diagnostics are errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The program cannot be compiled.
    Error,
    /// Informational feedback that does not prevent compilation.
    Warning,
}

/// A source-located, machine-readable frontend or compiler diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Stable code for tests and integrations.
    pub code: DiagnosticCode,
    /// Severity.
    pub severity: Severity,
    /// Human-readable explanation.
    pub message: String,
    /// Original source location.
    pub span: SourceSpan,
    /// Optional corrective suggestion.
    pub hint: Option<String>,
}

impl Diagnostic {
    pub(crate) fn error(
        code: DiagnosticCode,
        message: impl Into<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            span,
            hint: None,
        }
    }

    pub(crate) fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}
