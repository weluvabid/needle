use crate::{
    DiagnosticCode, ExecutionLimits, ExecutionSession, ExecutionStatus, MAX_COUNT,
    MAX_SOURCE_BYTES, Operation, compile, parse,
};

#[test]
fn parses_all_operations_and_pairs_nested_loops() {
    let program = parse("poke 3 next back nick loop loop spill stitch stitch snag").unwrap();
    assert_eq!(program.instructions.len(), 10);
    assert_eq!(
        program.instructions[0].operation,
        Operation::Poke { count: 3 }
    );
    assert_eq!(
        program.instructions[4].operation,
        Operation::LoopStart { partner: 8 }
    );
    assert_eq!(
        program.instructions[5].operation,
        Operation::LoopStart { partner: 7 }
    );
    assert_eq!(
        program.instructions[7].operation,
        Operation::LoopEnd { partner: 5 }
    );
    assert_eq!(
        program.instructions[8].operation,
        Operation::LoopEnd { partner: 4 }
    );
}

#[test]
fn comments_and_unicode_whitespace_are_accepted() {
    let program = parse("# hi 🪡\npoke 2\u{2003}spill # output").unwrap();
    assert_eq!(program.instructions.len(), 2);
    assert_eq!(program.instructions[1].span.line, 2);
    assert!(parse("poke 2\u{2003}spill").is_ok());
}

#[test]
fn reports_unknown_command_with_suggestion_and_source_location() {
    let errors = parse("\n  pokee 2").unwrap_err();
    assert_eq!(errors[0].code, DiagnosticCode::UnknownCommand);
    assert_eq!(errors[0].span.line, 2);
    assert_eq!(errors[0].span.column, 3);
    assert_eq!(errors[0].hint.as_deref(), Some("Did you mean `poke`?"));
}

#[test]
fn rejects_bad_counts_and_loop_structure() {
    assert_eq!(
        parse("poke 0").unwrap_err()[0].code,
        DiagnosticCode::InvalidCount
    );
    assert_eq!(
        parse(&format!("poke {}", MAX_COUNT + 1)).unwrap_err()[0].code,
        DiagnosticCode::InvalidCount
    );
    assert_eq!(
        parse("stitch").unwrap_err()[0].code,
        DiagnosticCode::UnmatchedStitch
    );
    assert_eq!(
        parse("loop\nnick").unwrap_err()[0].code,
        DiagnosticCode::UnclosedLoop
    );
    assert!(parse("loop 3 stitch").is_err());
}

#[test]
fn rejects_oversized_source() {
    let source = " ".repeat(MAX_SOURCE_BYTES + 1);
    assert_eq!(
        parse(&source).unwrap_err()[0].code,
        DiagnosticCode::SourceTooLarge
    );
}

#[test]
fn compiler_is_deterministic_and_maps_source_ranges() {
    let source = "poke 3\nnext 2\nloop\nnick\nstitch\nspill";
    let first = compile(source).unwrap();
    let second = compile(source).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.brainfuck, "+++>>[-].");
    assert_eq!(first.source_map.len(), 6);
    assert_eq!(
        (
            first.source_map[0].generated_start,
            first.source_map[0].generated_end
        ),
        (0, 3)
    );
    assert_eq!(
        (
            first.source_map[4].generated_start,
            first.source_map[4].generated_end
        ),
        (7, 8)
    );
    assert_eq!(first.source_map[4].source.line, 5);
}

#[test]
fn loops_clear_a_cell_and_halt() {
    let program = parse("poke 7\nloop\n  nick\nstitch\nspill").unwrap();
    let mut session =
        ExecutionSession::new(program, Vec::new(), ExecutionLimits::default()).unwrap();
    session.run_steps(100);
    assert_eq!(session.output_bytes_from(0), &[0]);
    assert_eq!(session.status(), ExecutionStatus::Halted);
}

#[test]
fn empty_program_is_already_halted() {
    let mut session = ExecutionSession::new(
        parse("# only a comment").unwrap(),
        Vec::new(),
        ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(session.status(), ExecutionStatus::Halted);
    assert!(!session.step_forward().unwrap());
}

#[test]
fn large_counts_compile_without_intermediate_string_allocations() {
    let compiled = compile("poke 100000").unwrap();
    assert_eq!(compiled.brainfuck.len(), 100_000);
    assert_eq!(compiled.source_map[0].generated_end, 100_000);
}

#[test]
fn source_spans_keep_utf8_offsets_and_scalar_columns() {
    let source = "# a 🪡\npokee";
    let errors = parse(source).unwrap_err();
    assert_eq!(errors[0].span.start, source.find("pokee").unwrap());
    assert_eq!(errors[0].span.line, 2);
    assert_eq!(errors[0].span.column, 1);
}

#[test]
fn compiler_rejects_oversized_generated_programs_before_emitting() {
    let source = "poke 1000000\npoke 1";
    let errors = compile(source).unwrap_err();
    assert_eq!(errors[0].code, DiagnosticCode::GeneratedProgramTooLarge);
    assert_eq!(errors[0].span.line, 2);
}
