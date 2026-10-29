use serde::Serialize;

use crate::ast::{Operation, Program};
use crate::{Diagnostic, DiagnosticCode, SourceSpan, parse};

/// Maximum size of the generated Brainfuck source, measured in instructions.
pub const MAX_BRAINFUCK_BYTES: usize = 1_000_000;

/// A generated-instruction range mapped to its originating Needle operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMapping {
    /// Inclusive byte/instruction offset in generated Brainfuck.
    pub generated_start: usize,
    /// Exclusive byte/instruction offset in generated Brainfuck.
    pub generated_end: usize,
    /// Original Needle source span.
    pub source: SourceSpan,
}

/// Result of a successful compilation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Compilation {
    /// Generated Brainfuck instructions.
    pub brainfuck: String,
    /// One mapping entry for every Needle operation.
    pub source_map: Vec<SourceMapping>,
    /// Validated program used by the reference interpreter.
    pub program: Program,
}

/// Compile a Needle source string directly to Brainfuck.
pub fn compile(source: &str) -> Result<Compilation, Vec<Diagnostic>> {
    let program = parse(source)?;
    let mut generated_len = 0_usize;

    for instruction in &program.instructions {
        let emitted = match instruction.operation {
            Operation::Poke { count }
            | Operation::Nick { count }
            | Operation::Next { count }
            | Operation::Back { count } => count as usize,
            _ => 1,
        };
        let Some(next_len) = generated_len.checked_add(emitted) else {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::GeneratedProgramTooLarge,
                "Generated Brainfuck size overflowed the supported range.",
                instruction.span,
            )]);
        };
        if next_len > MAX_BRAINFUCK_BYTES {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::GeneratedProgramTooLarge,
                format!("Generated Brainfuck is limited to {MAX_BRAINFUCK_BYTES} instructions."),
                instruction.span,
            )]);
        }
        generated_len = next_len;
    }

    let mut brainfuck = String::new();
    brainfuck.try_reserve_exact(generated_len).map_err(|_| {
        vec![Diagnostic::error(
            DiagnosticCode::AllocationFailed,
            "Needle couldn't reserve memory for generated Brainfuck.",
            SourceSpan::point(),
        )]
    })?;

    let mut source_map = Vec::new();
    source_map
        .try_reserve_exact(program.instructions.len())
        .map_err(|_| {
            vec![Diagnostic::error(
                DiagnosticCode::AllocationFailed,
                "Needle couldn't reserve memory for the source map.",
                SourceSpan::point(),
            )]
        })?;

    for instruction in &program.instructions {
        let generated_start = brainfuck.len();
        match &instruction.operation {
            Operation::Poke { count } => {
                emit_repeated(&mut brainfuck, '+', *count as usize, instruction.span)?;
            }
            Operation::Nick { count } => {
                emit_repeated(&mut brainfuck, '-', *count as usize, instruction.span)?;
            }
            Operation::Next { count } => {
                emit_repeated(&mut brainfuck, '>', *count as usize, instruction.span)?;
            }
            Operation::Back { count } => {
                emit_repeated(&mut brainfuck, '<', *count as usize, instruction.span)?;
            }
            Operation::LoopStart { .. } => brainfuck.push('['),
            Operation::LoopEnd { .. } => brainfuck.push(']'),
            Operation::Spill => brainfuck.push('.'),
            Operation::Snag => brainfuck.push(','),
        }
        source_map.push(SourceMapping {
            generated_start,
            generated_end: brainfuck.len(),
            source: instruction.span,
        });
    }

    debug_assert_eq!(brainfuck.len(), generated_len);
    Ok(Compilation {
        brainfuck,
        source_map,
        program,
    })
}

fn emit_repeated(
    output: &mut String,
    instruction: char,
    count: usize,
    span: SourceSpan,
) -> Result<(), Vec<Diagnostic>> {
    let Some(new_len) = output.len().checked_add(count) else {
        return Err(vec![Diagnostic::error(
            DiagnosticCode::GeneratedProgramTooLarge,
            "Generated Brainfuck size overflowed the supported range.",
            span,
        )]);
    };
    if new_len > MAX_BRAINFUCK_BYTES {
        return Err(vec![Diagnostic::error(
            DiagnosticCode::GeneratedProgramTooLarge,
            format!("Generated Brainfuck is limited to {MAX_BRAINFUCK_BYTES} instructions."),
            span,
        )]);
    }
    output.try_reserve(count).map_err(|_| {
        vec![Diagnostic::error(
            DiagnosticCode::AllocationFailed,
            "Needle couldn't reserve memory for generated Brainfuck.",
            span,
        )]
    })?;
    output.extend(std::iter::repeat_n(instruction, count));
    Ok(())
}
