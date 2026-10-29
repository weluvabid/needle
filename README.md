# Needle 🪡

Needle is a witty little esoteric programming language that compiles to Brainfuck. Poke, nick, stitch, and spill your way through a tiny language with a compiler, reference interpreter, and interactive playground featuring bidirectional stepping and source mapping.

## Features

- A deliberately small source language with eight commands.
- A Rust frontend and direct Brainfuck compiler with source mappings.
- A browser-independent reference interpreter with fixed, explicit execution limits.
- Reversible single-step execution backed by bounded deltas instead of full tape snapshots.
- A Rust/WebAssembly adapter and TypeScript playground running execution in a Web Worker.
- Differential tests that compare the reference interpreter with an independent Brainfuck test runner.

## Language at a glance

```needle
# A tiny greeting.
poke 72
spill
poke 33
spill
nick 72
spill
```

The program outputs `Hi!`.

| Command | Meaning | Brainfuck |
| --- | --- | --- |
| `poke [n]` | Increment the current cell `n` times | `+` repeated `n` times |
| `nick [n]` | Decrement the current cell `n` times | `-` repeated `n` times |
| `next [n]` | Move right `n` cells | `>` repeated `n` times |
| `back [n]` | Move left `n` cells | `<` repeated `n` times |
| `loop` | Begin a loop while the current cell is nonzero | `[` |
| `stitch` | End a loop | `]` |
| `spill` | Output the current cell as one byte | `.` |
| `snag` | Read one byte into the current cell | `,` |

Counts are optional for `poke`, `nick`, `next`, and `back`; a missing count means `1`. A count must be on the same line as its command. Counts must be between `1` and `1,000,000`. Commands are case-sensitive. Whitespace separates commands, and `#` begins a comment through the end of the line. There are no variables, expressions, functions, or other conventional-language features.

## Semantics and limits

- Tape: 30,000 cells by default, each an unsigned 8-bit byte.
- Cell arithmetic: wraps modulo 256.
- Pointer: starts at cell zero; moving outside the tape fails with a source-located runtime error. A counted move is checked atomically before it changes the pointer.
- Input: UTF-8 bytes supplied to the playground; `snag` consumes one byte. EOF leaves the current cell unchanged.
- Output: `spill` appends the current cell as one byte. The playground displays output decoded as UTF-8 with replacement characters for invalid sequences.
- Source size: at most 256 KiB.
- Generated Brainfuck: at most 1,000,000 instructions.
- Input and output: at most 1 MiB each.
- Execution budget: 1,000,000 successfully executed Needle operations per session.
- Tape configuration: hard maximum of 1,000,000 cells.
- Reversible history: retains up to 100,000 operations by default, with a hard maximum of 250,000. Older deltas are evicted first.
- One step executes one Needle source operation. A counted operation is atomic and counts as one Needle step; the compiler emits the repeated Brainfuck operations.
- Rewinding does not refund the session's cumulative execution budget.

These rules are Needle's contract rather than implementation-dependent Brainfuck behavior. Differential equivalence is expected for programs that finish within the defined model; failure state details for programs that attempt to leave the tape are Needle-specific.

## Repository layout

```text
crates/
  needle-core/    Lexer, parser, AST, diagnostics, compiler, source maps, interpreter, trace
  needle-wasm/    Thin wasm-bindgen adapter
web/              TypeScript playground and Web Worker
scripts/check.sh  Formatting, core Clippy, and native core tests
.github/workflows/ci.yml
```

## Development

Requirements: stable Rust with `rustfmt` and `clippy`, `wasm32-unknown-unknown`, `wasm-pack`, and Node.js 22 with npm.

From the repository root:

```sh
./scripts/check.sh
./scripts/build-wasm.sh
cd web
npm install
npm run check
npm run dev
```

Open the local URL printed by Vite. The playground runs the compiler and reference interpreter through WebAssembly in a Web Worker. The core itself has no browser dependencies and is tested natively. The WASM adapter is compiled for `wasm32-unknown-unknown` as part of `npm run build`.

For a production web build:

```sh
cd web
npm install
npm run build
npm run preview
```

The WASM binding is generated during `npm run build` by `scripts/build-wasm.sh`; its generated JS and binary are build artifacts, not source-controlled files. The first build needs a working Rust toolchain and dependency access.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy -p needle-core --all-targets -- -D warnings
cargo test -p needle-core
```

## License

MIT. See [LICENSE](LICENSE).
