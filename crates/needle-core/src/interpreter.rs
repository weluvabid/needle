use std::collections::VecDeque;

use serde::Serialize;

use crate::ast::{Operation, Program};
use crate::parser::{MAX_COUNT, MAX_SOURCE_BYTES};
use crate::span::SourceSpan;

/// Maximum source input accepted by a `snag` session.
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
/// Maximum trace entries that a session may retain.
pub const MAX_TRACE_ENTRIES: usize = 250_000;
/// Maximum operations executed in one `run_steps` call.
pub const MAX_BATCH_STEPS: usize = 100_000;
/// Number of tape cells shown on either side of the active cell.
pub const TAPE_WINDOW_RADIUS: usize = 8;
const DEFAULT_TAPE_CELLS: usize = 30_000;
const MAX_TAPE_CELLS: usize = 1_000_000;
const DEFAULT_MAX_INSTRUCTIONS: u64 = 1_000_000;
const MAX_INSTRUCTIONS: u64 = 100_000_000;
const DEFAULT_MAX_OUTPUT_BYTES: usize = 1024 * 1024;
const DEFAULT_MAX_TRACE_ENTRIES: usize = 100_000;

/// Resource budgets for a reference-interpreter session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLimits {
    /// Number of 8-bit tape cells.
    pub tape_cells: usize,
    /// Maximum successful source operations during the session; rewinding does not refund budget.
    pub max_instructions: u64,
    /// Maximum accumulated output bytes.
    pub max_output_bytes: usize,
    /// Maximum reversible history entries retained.
    pub max_trace_entries: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            tape_cells: DEFAULT_TAPE_CELLS,
            max_instructions: DEFAULT_MAX_INSTRUCTIONS,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            max_trace_entries: DEFAULT_MAX_TRACE_ENTRIES,
        }
    }
}

impl ExecutionLimits {
    fn validate(&self) -> Result<(), RuntimeError> {
        if !(1..=MAX_TAPE_CELLS).contains(&self.tape_cells)
            || self.max_instructions == 0
            || self.max_instructions > MAX_INSTRUCTIONS
            || self.max_output_bytes == 0
            || self.max_output_bytes > MAX_INPUT_BYTES
            || self.max_trace_entries == 0
            || self.max_trace_entries > MAX_TRACE_ENTRIES
        {
            return Err(RuntimeError::new(
                RuntimeErrorCode::InvalidLimits,
                "Execution limits are outside Needle's supported range.",
                None,
            ));
        }
        Ok(())
    }
}

/// Terminal or active status of an execution session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// No source operation has executed yet.
    Ready,
    /// The session has executed at least one operation and can continue.
    Running,
    /// The instruction pointer has reached the end of the program.
    Halted,
    /// A defined runtime limit or tape boundary was reached.
    Failed,
}

/// Stable runtime error categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeErrorCode {
    /// The tape pointer would leave the tape.
    PointerOutOfBounds,
    /// The total instruction budget has been consumed.
    InstructionBudgetExceeded,
    /// The output buffer is full.
    OutputLimitExceeded,
    /// Input exceeds the supported byte limit.
    InputLimitExceeded,
    /// Runtime limit configuration is invalid.
    InvalidLimits,
    /// A required allocation could not be reserved within the configured limits.
    AllocationFailed,
    /// An in-memory program violates AST invariants.
    InvalidProgram,
    /// Source exceeds the supported size.
    SourceTooLarge,
}

/// Structured, source-located runtime failure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeError {
    /// Stable error code.
    pub code: RuntimeErrorCode,
    /// Human-readable detail.
    pub message: String,
    /// Source operation that triggered the failure, when applicable.
    pub span: Option<SourceSpan>,
}

impl RuntimeError {
    fn new(code: RuntimeErrorCode, message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self { code, message: message.into(), span }
    }
}

/// Read-only view of an execution session, designed for the playground.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSnapshot {
    /// Current source instruction index.
    pub instruction_pointer: usize,
    /// Current tape index.
    pub data_pointer: usize,
    /// Cell value under the pointer.
    pub current_cell: u8,
    /// First index represented by `tape_window`.
    pub tape_window_start: usize,
    /// A small window around the active cell.
    pub tape_window: Vec<u8>,
    /// Offset of the active cell inside `tape_window`.
    pub tape_window_pointer_offset: usize,
    /// Accumulated output length; retrieve bytes separately to avoid copying it on every step.
    pub output_length: usize,
    /// Input cursor.
    pub input_cursor: usize,
    /// Monotonic count of successful source operations, including operations later undone.
    pub total_steps_executed: u64,
    /// Session status.
    pub status: ExecutionStatus,
    /// Active source span, if the instruction pointer addresses an operation.
    pub current_span: Option<SourceSpan>,
    /// Most recent runtime error.
    pub last_error: Option<RuntimeError>,
    /// Whether there is history available to reverse.
    pub can_step_backward: bool,
    /// Number of retained history entries.
    pub history_depth: usize,
}

#[derive(Clone, Debug)]
struct Delta {
    instruction_pointer: usize,
    data_pointer: usize,
    input_cursor: usize,
    output_length: usize,
    previous_cell: Option<(usize, u8)>,
    status: ExecutionStatus,
}

/// A bounded reference-interpreter session with reversible state transitions.
pub struct ExecutionSession {
    program: Program,
    tape: Vec<u8>,
    instruction_pointer: usize,
    data_pointer: usize,
    input: Vec<u8>,
    input_cursor: usize,
    output: Vec<u8>,
    status: ExecutionStatus,
    total_steps_executed: u64,
    limits: ExecutionLimits,
    history: VecDeque<Delta>,
    last_error: Option<RuntimeError>,
}

impl ExecutionSession {
    /// Create a session from an already validated program and byte input.
    pub fn new(
        program: Program,
        input: Vec<u8>,
        limits: ExecutionLimits,
    ) -> Result<Self, RuntimeError> {
        limits.validate()?;
        validate_program(&program)?;
        if input.len() > MAX_INPUT_BYTES {
            return Err(RuntimeError::new(
                RuntimeErrorCode::InputLimitExceeded,
                format!("Input is limited to {MAX_INPUT_BYTES} bytes."),
                None,
            ));
        }
        let status = if program.instructions.is_empty() {
            ExecutionStatus::Halted
        } else {
            ExecutionStatus::Ready
        };
        let mut output = Vec::new();
        output
            .try_reserve(limits.max_output_bytes.min(4096))
            .map_err(|_| {
                RuntimeError::new(
                    RuntimeErrorCode::AllocationFailed,
                    "Unable to reserve the output buffer.",
                    None,
                )
            })?;
        let mut tape = Vec::new();
        tape.try_reserve_exact(limits.tape_cells).map_err(|_| {
            RuntimeError::new(
                RuntimeErrorCode::AllocationFailed,
                "Unable to reserve the memory tape.",
                None,
            )
        })?;
        tape.resize(limits.tape_cells, 0);

        let history = VecDeque::new();

        Ok(Self {
            program,
            tape,
            instruction_pointer: 0,
            data_pointer: 0,
            input,
            input_cursor: 0,
            output,
            status,
            total_steps_executed: 0,
            limits,
            history,
            last_error: None,
        })
    }

    /// Execute one Needle source operation. Returns `false` if already halted.
    pub fn step_forward(&mut self) -> Result<bool, RuntimeError> {
        if self.status == ExecutionStatus::Failed {
            return Err(self.last_error.clone().unwrap_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorCode::InvalidLimits,
                    "The session is in a failed state.",
                    None,
                )
            }));
        }
        if self.status == ExecutionStatus::Halted {
            return Ok(false);
        }

        let Some(instruction) = self
            .program
            .instructions
            .get(self.instruction_pointer)
            .copied()
        else {
            self.status = ExecutionStatus::Halted;
            return Ok(false);
        };

        if self.total_steps_executed >= self.limits.max_instructions {
            let error = RuntimeError::new(
                RuntimeErrorCode::InstructionBudgetExceeded,
                "Needle ran out of thread: the instruction budget was exhausted.",
                Some(instruction.span),
            );
            self.fail(error.clone());
            return Err(error);
        }
        if self.history.len() < self.limits.max_trace_entries
            && self.history.len() == self.history.capacity()
            && self.history.try_reserve(1).is_err()
        {
            let error = RuntimeError::new(
                RuntimeErrorCode::AllocationFailed,
                "Needle couldn't reserve another reversible execution record.",
                Some(instruction.span),
            );
            self.fail(error.clone());
            return Err(error);
        }

        let old_ip = self.instruction_pointer;
        let old_dp = self.data_pointer;
        let old_input_cursor = self.input_cursor;
        let old_output_len = self.output.len();
        let old_status = self.status;
        let mut previous_cell = None;

        match instruction.operation {
            Operation::Poke { count } => {
                previous_cell = Some((self.data_pointer, self.tape[self.data_pointer]));
                self.tape[self.data_pointer] =
                    self.tape[self.data_pointer].wrapping_add(count as u8);
                self.instruction_pointer += 1;
            }
            Operation::Nick { count } => {
                previous_cell = Some((self.data_pointer, self.tape[self.data_pointer]));
                self.tape[self.data_pointer] =
                    self.tape[self.data_pointer].wrapping_sub(count as u8);
                self.instruction_pointer += 1;
            }
            Operation::Next { count } => {
                let next = self.data_pointer.checked_add(count as usize);
                if next.is_none_or(|value| value >= self.tape.len()) {
                    let error = RuntimeError::new(
                        RuntimeErrorCode::PointerOutOfBounds,
                        "Off the fabric: `next` moved the data pointer past the tape.",
                        Some(instruction.span),
                    );
                    self.fail(error.clone());
                    return Err(error);
                }
                self.data_pointer = next.unwrap_or(self.data_pointer);
                self.instruction_pointer += 1;
            }
            Operation::Back { count } => {
                if count as usize > self.data_pointer {
                    let error = RuntimeError::new(
                        RuntimeErrorCode::PointerOutOfBounds,
                        "Off the fabric: `back` moved the data pointer before the tape.",
                        Some(instruction.span),
                    );
                    self.fail(error.clone());
                    return Err(error);
                }
                self.data_pointer -= count as usize;
                self.instruction_pointer += 1;
            }
            Operation::LoopStart { partner } => {
                self.instruction_pointer = if self.tape[self.data_pointer] == 0 {
                    partner + 1
                } else {
                    self.instruction_pointer + 1
                };
            }
            Operation::LoopEnd { partner } => {
                self.instruction_pointer = if self.tape[self.data_pointer] != 0 {
                    partner
                } else {
                    self.instruction_pointer + 1
                };
            }
            Operation::Spill => {
                if self.output.len() >= self.limits.max_output_bytes {
                    let error = RuntimeError::new(
                        RuntimeErrorCode::OutputLimitExceeded,
                        "The output spool is full; raise the output limit to continue.",
                        Some(instruction.span),
                    );
                    self.fail(error.clone());
                    return Err(error);
                }
                if self.output.try_reserve(1).is_err() {
                    let error = RuntimeError::new(
                        RuntimeErrorCode::AllocationFailed,
                        "Needle couldn't reserve memory for another output byte.",
                        Some(instruction.span),
                    );
                    self.fail(error.clone());
                    return Err(error);
                }
                self.output.push(self.tape[self.data_pointer]);
                self.instruction_pointer += 1;
            }
            Operation::Snag => {
                if let Some(byte) = self.input.get(self.input_cursor).copied() {
                    previous_cell = Some((self.data_pointer, self.tape[self.data_pointer]));
                    self.tape[self.data_pointer] = byte;
                    self.input_cursor += 1;
                }
                self.instruction_pointer += 1;
            }
        }

        self.total_steps_executed = self.total_steps_executed.saturating_add(1);
        self.status = if self.instruction_pointer >= self.program.instructions.len() {
            ExecutionStatus::Halted
        } else {
            ExecutionStatus::Running
        };
        self.last_error = None;

        if self.history.len() == self.limits.max_trace_entries {
            self.history.pop_front();
        }
        self.history.push_back(Delta {
            instruction_pointer: old_ip,
            data_pointer: old_dp,
            input_cursor: old_input_cursor,
            output_length: old_output_len,
            previous_cell,
            status: old_status,
        });
        Ok(true)
    }

    /// Execute at most `max_steps` source operations and return the number executed in this batch.
    pub fn run_steps(&mut self, max_steps: usize) -> usize {
        let budget = max_steps.min(MAX_BATCH_STEPS);
        let mut executed = 0;
        while executed < budget {
            if !matches!(self.status, ExecutionStatus::Ready | ExecutionStatus::Running) {
                break;
            }
            match self.step_forward() {
                Ok(true) => executed += 1,
                Ok(false) | Err(_) => break,
            }
        }
        executed
    }

    /// Undo the most recent successful source operation, if retained in the trace.
    pub fn step_backward(&mut self) -> bool {
        let Some(delta) = self.history.pop_back() else {
            return false;
        };
        if let Some((index, value)) = delta.previous_cell {
            self.tape[index] = value;
        }
        self.instruction_pointer = delta.instruction_pointer;
        self.data_pointer = delta.data_pointer;
        self.input_cursor = delta.input_cursor;
        self.output.truncate(delta.output_length);
        self.status = delta.status;
        self.last_error = None;
        true
    }

    /// Return a compact snapshot; output bytes are retrieved independently.
    pub fn inspect(&self) -> ExecutionSnapshot {
        let window_start = self.data_pointer.saturating_sub(TAPE_WINDOW_RADIUS);
        let window_end = self
            .data_pointer
            .saturating_add(TAPE_WINDOW_RADIUS + 1)
            .min(self.tape.len());
        let tape_window = self.tape[window_start..window_end].to_vec();
        let current_span = self
            .program
            .instructions
            .get(self.instruction_pointer)
            .map(|instruction| instruction.span);

        ExecutionSnapshot {
            instruction_pointer: self.instruction_pointer,
            data_pointer: self.data_pointer,
            current_cell: self.tape[self.data_pointer],
            tape_window_start: window_start,
            tape_window,
            tape_window_pointer_offset: self.data_pointer - window_start,
            output_length: self.output.len(),
            input_cursor: self.input_cursor,
            total_steps_executed: self.total_steps_executed,
            status: self.status,
            current_span,
            last_error: self.last_error.clone(),
            can_step_backward: !self.history.is_empty(),
            history_depth: self.history.len(),
        }
    }

    /// Copy only the requested output tail, useful for incremental UI updates.
    pub fn output_bytes_from(&self, start: usize) -> &[u8] {
        self.output.get(start.min(self.output.len())..).unwrap_or_default()
    }

    /// Reset the interpreter to the initial state while preserving source and input.
    pub fn reset(&mut self) {
        self.tape.fill(0);
        self.instruction_pointer = 0;
        self.data_pointer = 0;
        self.input_cursor = 0;
        self.output.clear();
        self.total_steps_executed = 0;
        self.history.clear();
        self.last_error = None;
        self.status = if self.program.instructions.is_empty() {
            ExecutionStatus::Halted
        } else {
            ExecutionStatus::Ready
        };
    }

    /// Byte count of accumulated output.
    pub fn output_len(&self) -> usize {
        self.output.len()
    }

    /// Status of this execution session.
    pub fn status(&self) -> ExecutionStatus {
        self.status
    }

    fn fail(&mut self, error: RuntimeError) {
        self.status = ExecutionStatus::Failed;
        self.last_error = Some(error);
    }
}

fn validate_program(program: &Program) -> Result<(), RuntimeError> {
    if program.source_len > MAX_SOURCE_BYTES {
        return Err(RuntimeError::new(
            RuntimeErrorCode::SourceTooLarge,
            format!("Needle source is limited to {MAX_SOURCE_BYTES} bytes."),
            None,
        ));
    }
    if program.instructions.len() > program.source_len {
        return Err(RuntimeError::new(
            RuntimeErrorCode::InvalidProgram,
            "The instruction stream is not a validated Needle program.",
            None,
        ));
    }

    let mut loop_stack = Vec::new();
    for (index, instruction) in program.instructions.iter().enumerate() {
        let span = instruction.span;
        if span.start >= span.end
            || span.end > program.source_len
            || span.line == 0
            || span.column == 0
            || span.end_line < span.line
            || span.end_column == 0
        {
            return Err(RuntimeError::new(
                RuntimeErrorCode::InvalidProgram,
                "An instruction has an invalid source span.",
                None,
            ));
        }

        match &instruction.operation {
            Operation::Poke { count }
            | Operation::Nick { count }
            | Operation::Next { count }
            | Operation::Back { count } if !(1..=MAX_COUNT).contains(count) => {
                return Err(RuntimeError::new(
                    RuntimeErrorCode::InvalidProgram,
                    "A counted instruction is outside Needle's supported range.",
                    Some(span),
                ));
            }
            Operation::LoopStart { partner } => {
                let valid_partner = *partner > index
                    && *partner < program.instructions.len()
                    && matches!(
                        &program.instructions[*partner].operation,
                        Operation::LoopEnd { partner: open } if *open == index
                    );
                if !valid_partner {
                    return Err(RuntimeError::new(
                        RuntimeErrorCode::InvalidProgram,
                        "A loop opener has an invalid partner index.",
                        Some(span),
                    ));
                }
                loop_stack.push(index);
            }
            Operation::LoopEnd { partner }
                if loop_stack.pop() != Some(*partner) => {
                    return Err(RuntimeError::new(
                        RuntimeErrorCode::InvalidProgram,
                        "Loop pairs are not correctly nested.",
                        Some(span),
                    ));
                }
            _ => {}
        }
    }

    if !loop_stack.is_empty() {
        return Err(RuntimeError::new(
            RuntimeErrorCode::InvalidProgram,
            "The program contains an unclosed loop.",
            None,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn session(source: &str, input: &[u8], limits: ExecutionLimits) -> ExecutionSession {
        ExecutionSession::new(parse(source).expect("valid source"), input.to_vec(), limits)
            .expect("valid session")
    }

    #[test]
    fn wraps_cell_arithmetic() {
        let mut vm = session("nick\nspill", &[], ExecutionLimits::default());
        vm.run_steps(10);
        assert_eq!(vm.output_bytes_from(0), &[255]);
        let mut vm = session("poke 255\npoke\nspill", &[], ExecutionLimits::default());
        vm.run_steps(10);
        assert_eq!(vm.output_bytes_from(0), &[0]);
    }

    #[test]
    fn eof_leaves_cell_unchanged() {
        let mut vm = session("poke 42\nsnag\nspill", &[], ExecutionLimits::default());
        vm.run_steps(10);
        assert_eq!(vm.output_bytes_from(0), &[42]);
    }

    #[test]
    fn reverse_restores_tape_pointer_and_output() {
        let mut vm = session("poke 5\nnext\npoke 8\nspill", &[], ExecutionLimits::default());
        vm.run_steps(10);
        assert_eq!(vm.output_bytes_from(0), &[8]);
        assert!(vm.step_backward());
        assert_eq!(vm.output_len(), 0);
        assert!(vm.step_backward());
        assert_eq!(vm.inspect().current_cell, 0);
        assert_eq!(vm.inspect().data_pointer, 1);
        assert!(vm.step_backward());
        assert_eq!(vm.inspect().data_pointer, 0);
        assert!(vm.step_backward());
        assert_eq!(vm.inspect().current_cell, 0);
    }

    #[test]
    fn pointer_failure_is_structured() {
        let mut vm = session("back", &[], ExecutionLimits::default());
        let error = vm.step_forward().expect_err("pointer underflow should fail");
        assert_eq!(error.code, RuntimeErrorCode::PointerOutOfBounds);
        assert_eq!(vm.status(), ExecutionStatus::Failed);
        assert!(!vm.step_backward());
        assert_eq!(vm.status(), ExecutionStatus::Failed);
        assert_eq!(vm.inspect().last_error.unwrap().code, RuntimeErrorCode::PointerOutOfBounds);
    }

    #[test]
    fn execution_budget_is_monotonic_across_rewind() {
        let limits = ExecutionLimits { max_instructions: 3, ..ExecutionLimits::default() };
        let mut vm = session("poke\nnext\nback\nspill", &[], limits);
        assert_eq!(vm.run_steps(100), 3);
        assert_eq!(vm.status(), ExecutionStatus::Failed);
        assert_eq!(vm.inspect().total_steps_executed, 3);
        assert!(vm.step_backward());
        assert_eq!(vm.inspect().total_steps_executed, 3);
        assert!(vm.step_forward().is_err());
    }

    #[test]
    fn rejects_malformed_in_memory_programs() {
        use crate::{Instruction, Operation};

        let program = Program {
            source_len: 4,
            instructions: vec![Instruction {
                operation: Operation::LoopStart { partner: usize::MAX },
                span: SourceSpan::new(0, 4, 1, 1, 1, 5),
            }],
        };
        let result = ExecutionSession::new(program, Vec::new(), ExecutionLimits::default());
        assert!(matches!(result, Err(ref error) if error.code == RuntimeErrorCode::InvalidProgram));
    }

    #[test]
    fn output_limit_is_enforced() {
        let limits = ExecutionLimits { max_output_bytes: 1, ..ExecutionLimits::default() };
        let mut vm = session("spill\nspill", &[], limits);
        assert_eq!(vm.run_steps(10), 1);
        assert_eq!(vm.output_len(), 1);
        assert_eq!(vm.status(), ExecutionStatus::Failed);
        assert_eq!(vm.inspect().last_error.unwrap().code, RuntimeErrorCode::OutputLimitExceeded);
    }

    #[test]
    fn reversing_input_restores_cursor_and_cell() {
        let mut vm = session("snag\nspill", &[0xE2, 0x98, 0x83], ExecutionLimits::default());
        assert!(vm.step_forward().unwrap());
        assert_eq!(vm.inspect().input_cursor, 1);
        assert_eq!(vm.inspect().current_cell, 0xE2);
        assert!(vm.step_backward());
        assert_eq!(vm.inspect().input_cursor, 0);
        assert_eq!(vm.inspect().current_cell, 0);
        assert!(vm.step_forward().unwrap());
        assert!(vm.step_forward().unwrap());
        assert_eq!(vm.output_bytes_from(0), &[0xE2]);
    }

    #[test]
    fn invalid_limits_are_rejected() {
        let limits = ExecutionLimits { max_trace_entries: 0, ..ExecutionLimits::default() };
        let program = parse("poke").unwrap();
        let result = ExecutionSession::new(program, Vec::new(), limits);
        assert!(matches!(
            result,
            Err(ref error) if error.code == RuntimeErrorCode::InvalidLimits
        ));
    }
}
