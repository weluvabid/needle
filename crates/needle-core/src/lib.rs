//! The browser-independent language implementation for Needle.

mod ast;
mod compiler;
mod diagnostic;
mod interpreter;
mod lexer;
mod parser;
mod span;

pub use ast::{Instruction, Operation, Program};
pub use compiler::{Compilation, SourceMapping, compile};
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use interpreter::{
    ExecutionLimits, ExecutionSession, ExecutionSnapshot, ExecutionStatus, RuntimeError,
    RuntimeErrorCode, MAX_BATCH_STEPS, MAX_INPUT_BYTES, MAX_TRACE_ENTRIES, TAPE_WINDOW_RADIUS,
};
pub use parser::{MAX_COUNT, MAX_SOURCE_BYTES, parse};
pub use span::SourceSpan;

#[cfg(test)]
mod tests;
