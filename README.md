# Needle 🪡

[Open the live playground](https://weluvabid.github.io/needle/).

Needle is a witty little esoteric programming language that compiles to Brainfuck. Poke, nick, stitch, and spill your way through a tiny language with a compiler, reference interpreter, and browser playground with source mapping and runtime inspection.

## Features

- A small source language with eight commands.
- A Rust frontend and direct Brainfuck compiler with source mappings.
- A browser-independent reference interpreter with fixed, explicit execution limits.
- A reference interpreter with reversible single-step execution backed by bounded deltas instead of full tape snapshots.
- A compact Rust/WebAssembly program runner with curated execution and compiler stress examples.
- Runtime inspection for memory, output bytes, and execution metrics, with execution isolated in a Web Worker.
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
- Output: `spill` appends the current cell as one byte. The playground exposes the output as ASCII, hexadecimal, and decimal byte views.
- Source size: at most 256 KiB.
- Generated Brainfuck: at most 1,000,000 instructions.
- Input and output: at most 1 MiB each.
- Execution budget: 100,000,000 successfully executed Needle operations per session.
- Tape configuration: hard maximum of 1,000,000 cells.
- Reversible history: retains up to 100,000 operations by default, with a hard maximum of 250,000. Older deltas are evicted first.
- One step executes one Needle source operation. A counted operation is atomic and counts as one Needle step; the compiler emits the repeated Brainfuck operations.
- Rewinding does not refund the session's cumulative execution budget.

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

Requirements: stable Rust with `rustfmt` and `clippy`, `wasm32-unknown-unknown`, `wasm-pack`, and Node.js 26 with npm.

From the repository root:

```sh
./scripts/check.sh
./scripts/build-wasm.sh
cd web
npm install
npm run check
npm run dev
```

Open the local URL printed by Vite. The playground loads curated Needle programs and runs the compiler and reference interpreter through WebAssembly in a Web Worker. It exposes the selected program's output, memory tape, and execution metrics without requiring an editor. The core itself has no browser dependencies and is tested natively. The WASM adapter is compiled for `wasm32-unknown-unknown` as part of `npm run build`.

GitHub Actions deploys the production playground to GitHub Pages when changes are pushed to `main`.

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


The playground offers five execution budgets: Very Careful (100 thousand source instructions), Careful (1 million), Steady (10 million), Fast (60 million), and Maximum (100 million). The selected pace controls the execution budget and worker batch size and delay. The pace can be changed while a program is running.
