//! Differential tests comparing Needle execution with an independent
//! Brainfuck interpreter to verify that compilation preserves program behavior.

use needle_core::{ExecutionLimits, ExecutionSession, ExecutionStatus, Program, compile};

#[derive(Debug)]
struct BrainfuckResult {
    tape: Vec<u8>,
    pointer: usize,
    input_cursor: usize,
    output: Vec<u8>,
}

#[derive(Debug)]
struct NeedleResult {
    output: Vec<u8>,
    pointer: usize,
    input_cursor: usize,
    tape_window_start: usize,
    tape_window: Vec<u8>,
}

fn run_brainfuck(code: &str, input: &[u8]) -> BrainfuckResult {
    let instructions: Vec<char> = code.chars().collect();
    let mut partners = vec![0; instructions.len()];
    let mut stack = Vec::new();
    for (index, instruction) in instructions.iter().enumerate() {
        match instruction {
            '[' => stack.push(index),
            ']' => {
                let open = stack.pop().expect("compiler emits matched loops");
                partners[open] = index;
                partners[index] = open;
            }
            _ => {}
        }
    }
    assert!(stack.is_empty());

    let mut tape = vec![0_u8; 30_000];
    let mut pointer = 0_usize;
    let mut ip = 0_usize;
    let mut input_cursor = 0_usize;
    let mut output = Vec::new();
    let mut steps = 0_usize;
    while ip < instructions.len() {
        steps += 1;
        assert!(steps < 5_000_000, "compiled program exceeded test budget");
        match instructions[ip] {
            '+' => tape[pointer] = tape[pointer].wrapping_add(1),
            '-' => tape[pointer] = tape[pointer].wrapping_sub(1),
            '>' => {
                pointer += 1;
                assert!(pointer < tape.len());
            }
            '<' => {
                assert!(pointer > 0);
                pointer -= 1;
            }
            '.' => output.push(tape[pointer]),
            ',' => {
                if let Some(byte) = input.get(input_cursor).copied() {
                    tape[pointer] = byte;
                    input_cursor += 1;
                }
            }
            '[' if tape[pointer] == 0 => ip = partners[ip],
            ']' if tape[pointer] != 0 => ip = partners[ip],
            _ => {}
        }
        ip += 1;
    }
    BrainfuckResult {
        tape,
        pointer,
        input_cursor,
        output,
    }
}

fn run_needle(program: Program, input: &[u8]) -> NeedleResult {
    let mut session =
        ExecutionSession::new(program, input.to_vec(), ExecutionLimits::default()).unwrap();
    session.run_steps(1_000_000);
    let state = session.inspect();
    assert_eq!(state.status, ExecutionStatus::Halted);
    NeedleResult {
        output: session.output_bytes_from(0).to_vec(),
        pointer: state.data_pointer,
        input_cursor: state.input_cursor,
        tape_window_start: state.tape_window_start,
        tape_window: state.tape_window,
    }
}

fn assert_equivalent(source: &str, input: &[u8]) {
    let compiled = compile(source).unwrap();
    let interpreted = run_needle(compiled.program, input);
    let generated = run_brainfuck(&compiled.brainfuck, input);

    assert_eq!(
        interpreted.output, generated.output,
        "output mismatch for {source}"
    );
    assert_eq!(
        interpreted.pointer, generated.pointer,
        "pointer mismatch for {source}"
    );
    assert_eq!(
        interpreted.input_cursor, generated.input_cursor,
        "input mismatch for {source}"
    );
    for (index, value) in interpreted.tape_window.iter().enumerate() {
        let tape_index = interpreted.tape_window_start + index;
        assert_eq!(
            *value, generated.tape[tape_index],
            "tape mismatch for {source} at {tape_index}"
        );
    }
}

#[test]
fn compiled_and_reference_execution_agree_on_handwritten_programs() {
    let programs = [
        "poke 72 spill poke 33 spill nick 72 spill",
        "poke 7 loop nick stitch spill",
        "poke 3 next 2 poke 255 next back nick 2 back spill next spill",
        "snag spill snag spill snag spill",
        "poke 2 loop next poke 4 back nick stitch next spill",
        "poke 255 poke 2 spill nick 1 spill",
    ];
    let inputs: [&[u8]; 6] = [b"", b"", b"", b"ABC", b"", b""];

    for (source, input) in programs.into_iter().zip(inputs) {
        assert_equivalent(source, input);
    }
}

#[test]
fn deterministic_generated_programs_agree() {
    let mut seed = 0x9e37_79b9_u32;
    for _case in 0..256 {
        let mut source = String::new();
        let mut pointer = 0_i32;
        for _ in 0..12 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            match seed % 8 {
                0 => source.push_str("poke 1\n"),
                1 => source.push_str("poke 3\n"),
                2 => source.push_str("nick 1\n"),
                3 => source.push_str("nick 2\n"),
                4 if pointer < 8 => {
                    source.push_str("next\n");
                    pointer += 1;
                }
                5 if pointer > 0 => {
                    source.push_str("back\n");
                    pointer -= 1;
                }
                4 | 5 => source.push_str("poke\n"),
                6 => source.push_str("spill\n"),
                _ => source.push_str("snag\n"),
            }
        }
        assert_equivalent(&source, b"needletest");
    }
}
