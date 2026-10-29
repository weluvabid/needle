use crate::ast::{Instruction, Operation, Program};
use crate::lexer::{Token, TokenKind, tokenize};
use crate::{Diagnostic, DiagnosticCode, SourceSpan};

/// Maximum source size accepted by the frontend.
pub const MAX_SOURCE_BYTES: usize = 256 * 1024;
/// Maximum literal count accepted by a single counted operation.
pub const MAX_COUNT: u32 = 1_000_000;

const COMMANDS: [&str; 8] = [
    "poke", "nick", "next", "back", "loop", "stitch", "spill", "snag",
];

/// Tokenize and parse source into a validated program.
pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(vec![Diagnostic::error(
            DiagnosticCode::SourceTooLarge,
            format!("Needle source is limited to {MAX_SOURCE_BYTES} bytes."),
            SourceSpan::point(),
        )]);
    }

    let (tokens, mut diagnostics) = tokenize(source);
    let mut parser = Parser {
        tokens,
        position: 0,
        instructions: Vec::new(),
        loop_stack: Vec::new(),
        diagnostics: Vec::new(),
    };
    parser.run();
    diagnostics.extend(parser.diagnostics);

    if diagnostics.is_empty() {
        Ok(Program {
            instructions: parser.instructions,
            source_len: source.len(),
        })
    } else {
        Err(diagnostics)
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    instructions: Vec<Instruction>,
    loop_stack: Vec<usize>,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn run(&mut self) {
        while self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            match token.kind {
                TokenKind::Number(_) => self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnexpectedNumber,
                    "A number needs a command to stitch it to.",
                    token.span,
                )),
                TokenKind::Word(word) => self.command(word, token.span),
            }
        }

        for &index in &self.loop_stack {
            let instruction = &self.instructions[index];
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnclosedLoop,
                "Unfinished stitch: this `loop` never closes with `stitch`.",
                instruction.span,
            ));
        }
    }

    fn command(&mut self, word: String, word_span: SourceSpan) {
        match word.as_str() {
            "poke" => self.counted(word_span, Operation::Poke { count: 1 }),
            "nick" => self.counted(word_span, Operation::Nick { count: 1 }),
            "next" => self.counted(word_span, Operation::Next { count: 1 }),
            "back" => self.counted(word_span, Operation::Back { count: 1 }),
            "loop" => {
                self.reject_count_after(&word, word_span);
                let index = self.instructions.len();
                self.instructions.push(Instruction {
                    operation: Operation::LoopStart { partner: usize::MAX },
                    span: word_span,
                });
                self.loop_stack.push(index);
            }
            "stitch" => {
                self.reject_count_after(&word, word_span);
                if let Some(open_index) = self.loop_stack.pop() {
                    let close_index = self.instructions.len();
                    self.instructions[open_index].operation = Operation::LoopStart {
                        partner: close_index,
                    };
                    self.instructions.push(Instruction {
                        operation: Operation::LoopEnd { partner: open_index },
                        span: word_span,
                    });
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::UnmatchedStitch,
                        "Loose thread: `stitch` has no matching `loop`.",
                        word_span,
                    ));
                }
            }
            "spill" => {
                self.reject_count_after(&word, word_span);
                self.instructions.push(Instruction {
                    operation: Operation::Spill,
                    span: word_span,
                });
            }
            "snag" => {
                self.reject_count_after(&word, word_span);
                self.instructions.push(Instruction {
                    operation: Operation::Snag,
                    span: word_span,
                });
            }
            _ => {
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnknownCommand,
                    format!("Unknown stitch: `{word}` is not a Needle command."),
                    word_span,
                );
                if let Some(suggestion) = suggestion(&word) {
                    diagnostic = diagnostic.with_hint(format!("Did you mean `{suggestion}`?"));
                }
                self.diagnostics.push(diagnostic);
            }
        }
    }

    fn counted(&mut self, word_span: SourceSpan, default: Operation) {
        let count_token = self.tokens.get(self.position).and_then(|token| {
            if token.span.line != word_span.end_line {
                return None;
            }
            if let TokenKind::Number(value) = &token.kind {
                Some((*value, token.span))
            } else {
                None
            }
        });

        let mut span = word_span;
        let count = if let Some((value, count_span)) = count_token {
            self.position += 1;
            span.end = count_span.end;
            span.end_line = count_span.end_line;
            span.end_column = count_span.end_column;
            if value == 0 || value > MAX_COUNT {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidCount,
                    format!("Counts must be between 1 and {MAX_COUNT} (or omitted for 1)."),
                    count_span,
                ));
                value.clamp(1, MAX_COUNT)
            } else {
                value
            }
        } else {
            1
        };

        let operation = match default {
            Operation::Poke { .. } => Operation::Poke { count },
            Operation::Nick { .. } => Operation::Nick { count },
            Operation::Next { .. } => Operation::Next { count },
            Operation::Back { .. } => Operation::Back { count },
            _ => unreachable!("only counted operations reach counted()"),
        };
        self.instructions.push(Instruction { operation, span });
    }

    fn reject_count_after(&mut self, word: &str, word_span: SourceSpan) {
        if let Some(Token { kind: TokenKind::Number(_), span }) = self
            .tokens
            .get(self.position)
            .filter(|token| token.span.line == word_span.end_line)
        {
            let span = *span;
            self.position += 1;
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnexpectedNumber,
                format!("`{word}` doesn't take a count; its syntax is already stitched."),
                span,
            ).with_hint(format!("Remove the number after `{word}`.")));
        }
    }
}

fn suggestion(word: &str) -> Option<&'static str> {
    COMMANDS
        .iter()
        .copied()
        .map(|command| (command, edit_distance(word, command)))
        .filter(|(_, distance)| *distance <= 2)
        .min_by_key(|(_, distance)| *distance)
        .map(|(command, _)| command)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    let mut current = vec![0; right_chars.len() + 1];

    for (row, left_char) in left.chars().enumerate() {
        current[0] = row + 1;
        for (column, right_char) in right_chars.iter().enumerate() {
            let substitution = previous[column] + if left_char == *right_char { 0 } else { 1 };
            current[column + 1] = substitution
                .min(previous[column + 1] + 1)
                .min(current[column] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}
