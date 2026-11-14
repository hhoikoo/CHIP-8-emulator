use crate::{
    example_fn::FN_ADD_TWICE,
    op::{OP_HALT, OP_RETURN},
};

use super::*;
use Register::*;

/// Generous limit so a broken test fails instead of hanging.
const LIMIT: usize = 10_000;

fn cpu(segs: &[(usize, &[u16])]) -> Cpu {
    Cpu::new(&Rom::new(segs).unwrap())
}

#[test]
fn new_cpu_has_zeroed_registers() {
    let c = cpu(&[(0, &[OP_HALT])]);
    assert_eq!(c.registers, Registers::default());
}

#[test]
fn execution_starts_at_address_zero() {
    let mut c = cpu(&[(0, &[0x6007, OP_HALT])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 7);
}

#[test]
fn halt_stops_before_later_instructions() {
    let mut c = cpu(&[(0, &[OP_HALT, 0x6005])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn code_in_the_last_memory_slot_runs() {
    // CALL the function at 0xFFE, whose RET is the final opcode in memory
    let mut c = cpu(&[(0, &[0x2FFE, 0x6001, OP_HALT]), (0xFFE, &[OP_RETURN])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 1);
}

#[test]
fn load_and_add_immediate() {
    let mut c = cpu(&[(0, &[0x6005, 0x7003, OP_HALT])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 8);
}

#[test]
fn add_immediate_wraps() {
    let mut c = cpu(&[(0, &[0x60FF, 0x7002, OP_HALT])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 1);
}

#[test]
fn add_with_carry_sets_vf() {
    let mut c = cpu(&[(0, &[0x60F0, 0x6120, 0x8014, OP_HALT])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 0x10);
    assert_eq!(c.get_register_value(VF), 1);
}

#[test]
fn add_with_carry_uses_the_named_registers() {
    // VA += VB; V0 and V1 must stay untouched
    let mut c = cpu(&[(0, &[0x6A03, 0x6B04, 0x8AB4, OP_HALT])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(VA), 7);
    assert_eq!(c.get_register_value(VB), 4);
    assert_eq!(c.get_register_value(V0), 0);
    assert_eq!(c.get_register_value(VF), 0);
}

#[test]
fn add_with_carry_into_vf_keeps_the_flag() {
    // VF += V0 overflows: the carry flag overwrites the sum
    let mut c = cpu(&[(0, &[0x6FFF, 0x6002, 0x8F04, OP_HALT])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(VF), 1);
}

#[test]
fn add_with_carry_clears_a_stale_flag() {
    let mut c = cpu(&[(0, &[0x6F01, 0x6001, 0x6101, 0x8014, OP_HALT])]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 2);
    assert_eq!(c.get_register_value(VF), 0);
}

#[test]
fn the_real_main_program_via_two_segments() {
    let main = [0x6005, 0x610A, 0x2100, 0x2100, OP_HALT];
    let mut c = cpu(&[(0, &main), (0x100, &FN_ADD_TWICE)]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 45);
}

#[test]
fn segment_order_in_literal_does_not_matter() {
    let main = [0x6005, 0x610A, 0x2100, 0x2100, OP_HALT];
    let mut c = cpu(&[(0x100, &FN_ADD_TWICE), (0, &main)]);
    c.run_for(LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 45);
}

fn run_err(segs: &[(usize, &[u16])]) -> CpuError {
    cpu(segs).run_for(LIMIT).unwrap_err()
}

#[test]
fn ret_with_empty_stack_is_underflow() {
    // pc has already advanced past the RET when the error is raised
    let err = run_err(&[(0, &[OP_RETURN])]);
    assert!(matches!(err, CpuError::StackUnderflow { pc: 2 }));
}

#[test]
fn infinite_recursion_is_stack_overflow() {
    let err = run_err(&[(0, &[0x2000])]);
    match err {
        CpuError::StackOverflow { pc, addr, stack } => {
            assert_eq!((pc, addr), (2, 0));
            assert_eq!(stack, vec![2; STACK_SIZE]);
        }
        other => panic!("expected StackOverflow, got {other:?}"),
    }
}

/// Runs a program that nests `depth` CALLs, unwinds them all, then halts.
/// Function `i` lives at `i * 0x10` and calls function `i + 1`.
fn run_nested_calls(depth: usize) -> Result<(), CpuError> {
    let main = [0x2010, OP_HALT];
    let middle: Vec<[u16; 2]> = (1..depth)
        .map(|i| [0x2000 | ((i + 1) * 0x10) as u16, OP_RETURN])
        .collect();
    let last = [OP_RETURN];

    let mut segs: Vec<(usize, &[u16])> = vec![(0, &main)];
    segs.extend(
        middle
            .iter()
            .enumerate()
            .map(|(i, f)| ((i + 1) * 0x10, &f[..])),
    );
    segs.push((depth * 0x10, &last));

    cpu(&segs).run_for(LIMIT)
}

#[test]
fn call_depth_at_the_stack_limit_is_ok() {
    run_nested_calls(STACK_SIZE).unwrap();
}

#[test]
fn one_call_past_the_stack_limit_overflows() {
    let err = run_nested_calls(STACK_SIZE + 1).unwrap_err();
    assert!(matches!(err, CpuError::StackOverflow { .. }));
}

#[test]
fn unknown_opcode_reports_its_own_address() {
    let err = run_err(&[(0, &[0x6001, 0xF000])]);
    assert!(matches!(
        err,
        CpuError::UnknownOpcode {
            opcode: 0xF000,
            pc: 2
        }
    ));
}

#[test]
fn running_off_the_end_of_memory_is_pc_out_of_bounds() {
    // CALL the last opcode slot, which has no HALT after it
    let err = run_err(&[(0, &[0x2FFE]), (0xFFE, &[0x6001])]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc: 0x1000 }));
}

#[test]
fn odd_pc_on_the_last_byte_is_pc_out_of_bounds() {
    // 0xFFF has a first byte but no second byte
    let err = run_err(&[(0, &[0x2FFF])]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc: 0xFFF }));
}

#[test]
fn error_messages() {
    assert_eq!(
        run_err(&[(0, &[OP_RETURN])]).to_string(),
        "Stack underflow at pc 0x0002"
    );
    assert_eq!(
        run_err(&[(0, &[0x6001, 0xF000])]).to_string(),
        "Opcode 0xf000 at pc 0x0002 is unrecognized or is not supported yet"
    );
    assert_eq!(
        run_err(&[(0, &[0x2FFE]), (0xFFE, &[0x6001])]).to_string(),
        "Program counter 0x1000 out of bounds"
    );
    assert!(
        run_err(&[(0, &[0x2000])])
            .to_string()
            .starts_with("Stack overflow: CALL 0x0000 at pc 0x0002")
    );
    assert_eq!(
        cpu(&[(0, &[0x6001])]).run_for(0).unwrap_err().to_string(),
        "Cycle limit reached before halting, at pc 0x0000"
    );
}

// ---- step() ----

#[test]
fn new_cpu_is_running() {
    let c = cpu(&[(0, &[OP_HALT])]);
    assert_eq!(c.state, CpuState::Running);
}

#[test]
fn step_runs_exactly_one_instruction() {
    let mut c = cpu(&[(0, &[0x6005, 0x7003, OP_HALT])]);

    c.step().unwrap();
    assert_eq!(c.get_register_value(V0), 5);
    assert_eq!(c.pc, 2);
    assert_eq!(c.state, CpuState::Running);

    c.step().unwrap();
    assert_eq!(c.get_register_value(V0), 8);
    assert_eq!(c.pc, 4);
    assert_eq!(c.state, CpuState::Running);
}

#[test]
fn step_on_halt_sets_halt_state() {
    let mut c = cpu(&[(0, &[OP_HALT])]);
    c.step().unwrap();
    assert_eq!(c.state, CpuState::Halt);
}

#[test]
fn step_after_halt_stays_halted() {
    // the load after HALT must never run
    let mut c = cpu(&[(0, &[OP_HALT, 0x6005])]);
    c.step().unwrap();
    let pc_at_halt = c.pc;

    for _ in 0..3 {
        c.step().unwrap();
        assert_eq!(c.state, CpuState::Halt);
        assert_eq!(c.pc, pc_at_halt);
        assert_eq!(c.get_register_value(V0), 0);
    }
}

#[test]
fn step_through_call_and_return() {
    let mut c = cpu(&[(0, &[0x2100, OP_HALT]), (0x100, &[OP_RETURN])]);

    c.step().unwrap();
    assert_eq!(c.pc, 0x100);
    assert_eq!(c.stack_ptr, 1);

    c.step().unwrap();
    assert_eq!(c.pc, 2);
    assert_eq!(c.stack_ptr, 0);

    c.step().unwrap();
    assert_eq!(c.state, CpuState::Halt);
}

#[test]
fn step_returns_errors() {
    let mut c = cpu(&[(0, &[OP_RETURN])]);
    assert!(matches!(c.step(), Err(CpuError::StackUnderflow { pc: 2 })));
    assert_eq!(c.state, CpuState::Running);
}

#[test]
fn stepping_matches_run() {
    let main = [0x6005, 0x610A, 0x2100, 0x2100, OP_HALT];
    let segs: &[(usize, &[u16])] = &[(0, &main), (0x100, &FN_ADD_TWICE)];

    let mut stepped = cpu(segs);
    let mut steps = 0;
    while stepped.state != CpuState::Halt {
        stepped.step().unwrap();
        steps += 1;
    }
    // 2 loads + 2 * (call + 2 adds + return) + halt
    assert_eq!(steps, 11);

    let mut ran = cpu(segs);
    ran.run().unwrap();
    assert_eq!(stepped.registers, ran.registers);
    assert_eq!(stepped.pc, ran.pc);
}

// ---- run() and run_for() ----

/// Three instructions, then HALT: 4 cycles in total.
const COUNT_TO_THREE: [u16; 4] = [0x7001, 0x7001, 0x7001, OP_HALT];

#[test]
fn run_goes_until_halt() {
    let mut c = cpu(&[(0, &COUNT_TO_THREE)]);
    c.run().unwrap();
    assert_eq!(c.get_register_value(V0), 3);
    assert_eq!(c.state, CpuState::Halt);
}

#[test]
fn run_returns_errors() {
    let mut c = cpu(&[(0, &[0x6001, OP_RETURN])]);
    assert!(matches!(c.run(), Err(CpuError::StackUnderflow { pc: 4 })));
}

#[test]
fn run_on_a_halted_cpu_does_nothing() {
    let mut c = cpu(&[(0, &[OP_HALT, 0x6005])]);
    c.run().unwrap();
    c.run().unwrap();
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn halt_on_the_last_allowed_cycle_is_ok() {
    let mut c = cpu(&[(0, &COUNT_TO_THREE)]);
    c.run_for(4).unwrap();
    assert_eq!(c.get_register_value(V0), 3);
}

#[test]
fn one_cycle_short_of_halt_is_limit_reached() {
    let mut c = cpu(&[(0, &COUNT_TO_THREE)]);
    // pc points at the HALT that didn't get to run
    assert!(matches!(
        c.run_for(3),
        Err(CpuError::CycleLimitReached { pc: 6 })
    ));
    assert_eq!(c.get_register_value(V0), 3);
    assert_eq!(c.state, CpuState::Running);
}

#[test]
fn zero_cycle_limit_runs_nothing() {
    let mut c = cpu(&[(0, &COUNT_TO_THREE)]);
    assert!(matches!(
        c.run_for(0),
        Err(CpuError::CycleLimitReached { pc: 0 })
    ));
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn run_for_resumes_after_hitting_the_limit() {
    let mut c = cpu(&[(0, &COUNT_TO_THREE)]);
    assert!(c.run_for(2).is_err());
    assert_eq!(c.get_register_value(V0), 2);
    c.run_for(2).unwrap();
    assert_eq!(c.get_register_value(V0), 3);
}

#[test]
fn run_for_on_a_halted_cpu_is_ok() {
    let mut c = cpu(&[(0, &[OP_HALT, 0x6005])]);
    c.run().unwrap();
    c.run_for(5).unwrap();
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn errors_win_over_the_cycle_limit() {
    // the error happens on the last allowed cycle
    let mut c = cpu(&[(0, &[0x6001, OP_RETURN])]);
    assert!(matches!(c.run_for(2), Err(CpuError::StackUnderflow { .. })));
}
