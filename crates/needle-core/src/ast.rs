use serde::Serialize;

use crate::SourceSpan;

/// One validated source operation. Loop instructions point to their partner index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    /// Operation kind and, where relevant, its compile-time count or loop partner.
    pub operation: Operation,
    /// Source range including an optional count literal.
    pub span: SourceSpan,
}

/// The small, fixed vocabulary of Needle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Operation {
    /// Increment the current 8-bit cell.
    Poke {
        /// Number of increments, within the supported count range.
        count: u32,
    },
    /// Decrement the current 8-bit cell.
    Nick {
        /// Number of decrements, within the supported count range.
        count: u32,
    },
    /// Move the pointer right.
    Next {
        /// Number of cells to move, within the supported count range.
        count: u32,
    },
    /// Move the pointer left.
    Back {
        /// Number of cells to move, within the supported count range.
        count: u32,
    },
    /// Open a loop.
    LoopStart {
        /// Index of the matching `stitch` instruction.
        partner: usize,
    },
    /// Close a loop.
    LoopEnd {
        /// Index of the matching `loop` instruction.
        partner: usize,
    },
    /// Write the current cell as one byte.
    Spill,
    /// Read one byte into the current cell; EOF leaves the cell unchanged.
    Snag,
}

/// Validated flat AST. Flat loop pairs keep stepping and source mapping simple.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    /// Source operations in execution order.
    pub instructions: Vec<Instruction>,
    /// Original UTF-8 source size in bytes.
    pub source_len: usize,
}
