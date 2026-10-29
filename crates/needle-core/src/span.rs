use serde::Serialize;

/// A byte range and one-based display location in the original source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    /// Inclusive UTF-8 byte offset.
    pub start: usize,
    /// Exclusive UTF-8 byte offset.
    pub end: usize,
    /// One-based start line.
    pub line: usize,
    /// One-based start column, counted in Unicode scalar values.
    pub column: usize,
    /// One-based exclusive end line.
    pub end_line: usize,
    /// One-based exclusive end column.
    pub end_column: usize,
}

impl SourceSpan {
    pub(crate) fn new(
        start: usize,
        end: usize,
        line: usize,
        column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Self {
        Self {
            start,
            end,
            line,
            column,
            end_line,
            end_column,
        }
    }

    pub(crate) fn point() -> Self {
        Self::new(0, 0, 1, 1, 1, 1)
    }
}
