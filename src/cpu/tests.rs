use crate::{
    examples::{fn_add_twice, fn_main},
    isa::addr::Addr,
};

use super::*;
use Instruction::*;
use Register::*;

fn load(x: Register, nn: u8) -> Instruction {
    LoadImmediate { x, nn }
}

fn add(x: Register, nn: u8) -> Instruction {
    AddImmediate { x, nn }
}

fn add_with_carry(x: Register, y: Register) -> Instruction {
    AddWithCarry { x, y }
}

fn call(addr: u16) -> Instruction {
    Call {
        addr: Addr::new(addr).unwrap(),
    }
}

const CYCLE_LIMIT: usize = 10_000;

fn cpu<T: Copy + Into<Opcode>>(segs: &[(usize, &[T])]) -> Cpu {
    Cpu::new(&Rom::new(segs).unwrap())
}

#[test]
fn new_cpu_has_zeroed_registers() {
    let c = cpu(&[(0, &[Halt])]);
    assert_eq!(c.registers, Registers::default());
}

#[test]
fn execution_starts_at_address_zero() {
    let mut c = cpu(&[(0, &[load(V0, 7), Halt])]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 7);
}

#[test]
fn halt_stops_before_later_instructions() {
    let mut c = cpu(&[(0, &[Halt, load(V0, 5)])]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn code_in_the_last_memory_slot_runs() {
    let mut c = cpu(&[(0, &[call(0xFFE), load(V0, 1), Halt]), (0xFFE, &[Return])]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 1);
}

#[test]
fn load_and_add_immediate() {
    let mut c = cpu(&[(0, &[load(V0, 5), add(V0, 3), Halt])]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 8);
}

#[test]
fn add_immediate_wraps() {
    let mut c = cpu(&[(0, &[load(V0, 0xFF), add(V0, 2), Halt])]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 1);
}

#[test]
fn add_with_carry_sets_vf() {
    let mut c = cpu(&[(
        0,
        &[load(V0, 0xF0), load(V1, 0x20), add_with_carry(V0, V1), Halt],
    )]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 0x10);
    assert_eq!(c.get_register_value(VF), 1);
}

#[test]
fn add_with_carry_uses_the_named_registers() {
    let mut c = cpu(&[(0, &[load(VA, 3), load(VB, 4), add_with_carry(VA, VB), Halt])]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(VA), 7);
    assert_eq!(c.get_register_value(VB), 4);
    assert_eq!(c.get_register_value(V0), 0);
    assert_eq!(c.get_register_value(VF), 0);
}

#[test]
fn add_with_carry_into_vf_keeps_the_flag() {
    let mut c = cpu(&[(
        0,
        &[load(VF, 0xFF), load(V0, 2), add_with_carry(VF, V0), Halt],
    )]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(VF), 1);
}

#[test]
fn add_with_carry_clears_a_stale_flag() {
    let mut c = cpu(&[(
        0,
        &[
            load(VF, 1),
            load(V0, 1),
            load(V1, 1),
            add_with_carry(V0, V1),
            Halt,
        ],
    )]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 2);
    assert_eq!(c.get_register_value(VF), 0);
}

#[test]
fn the_real_main_program_via_two_segments() {
    let main = fn_main();
    let mut c = cpu(&[(0, &main), (0x100, &fn_add_twice())]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 45);
}

#[test]
fn segment_order_in_literal_does_not_matter() {
    let main = fn_main();
    let mut c = cpu(&[(0x100, &fn_add_twice()), (0, &main)]);
    c.run_for(CYCLE_LIMIT).unwrap();
    assert_eq!(c.get_register_value(V0), 45);
}

fn run_err<T: Copy + Into<Opcode>>(segs: &[(usize, &[T])]) -> CpuError {
    cpu(segs).run_for(CYCLE_LIMIT).unwrap_err()
}

#[test]
fn ret_with_empty_stack_is_underflow_at_the_next_pc() {
    let err = run_err(&[(0, &[Return])]);
    assert!(matches!(err, CpuError::StackUnderflow { pc: 2 }));
}

#[test]
fn infinite_recursion_is_stack_overflow() {
    let err = run_err(&[(0, &[call(0)])]);
    match err {
        CpuError::StackOverflow { pc, addr, stack } => {
            assert_eq!((pc, addr.get()), (2, 0));
            assert_eq!(stack, vec![2; STACK_SIZE]);
        }
        other => panic!("expected StackOverflow, got {other:?}"),
    }
}

fn function_addr(index: usize) -> usize {
    index * 0x10
}

fn call_function(index: usize) -> Instruction {
    call(u16::try_from(function_addr(index)).unwrap())
}

fn run_call_chain(depth: usize) -> Result<(), CpuError> {
    let main = [call_function(1), Halt];
    let callers: Vec<[Instruction; 2]> = (1..depth)
        .map(|index| [call_function(index + 1), Return])
        .collect();
    let deepest = [Return];

    let mut segs: Vec<(usize, &[Instruction])> = vec![(0, &main)];
    segs.extend(
        callers
            .iter()
            .zip(1..)
            .map(|(function, index)| (function_addr(index), &function[..])),
    );
    segs.push((function_addr(depth), &deepest));

    cpu(&segs).run_for(CYCLE_LIMIT)
}

#[test]
fn call_depth_at_the_stack_limit_is_ok() {
    run_call_chain(STACK_SIZE).unwrap();
}

#[test]
fn one_call_past_the_stack_limit_overflows() {
    let err = run_call_chain(STACK_SIZE + 1).unwrap_err();
    assert!(matches!(err, CpuError::StackOverflow { .. }));
}

#[test]
fn unknown_opcode_reports_its_own_address() {
    let err = run_err(&[(0, &[0x6001u16, 0xF000])]);
    assert!(matches!(
        err,
        CpuError::UnknownOpcode { opcode, pc: 2 } if opcode == Opcode::new(0xF000)
    ));
}

#[test]
fn running_off_the_end_of_memory_is_pc_out_of_bounds() {
    let err = run_err(&[(0, &[call(0xFFE)]), (0xFFE, &[load(V0, 1)])]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc: 0x1000 }));
}

#[test]
fn odd_pc_on_the_last_byte_is_pc_out_of_bounds() {
    let err = run_err(&[(0, &[call(0xFFF)])]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc: 0xFFF }));
}

#[test]
fn error_messages() {
    assert_eq!(
        run_err(&[(0, &[Return])]).to_string(),
        "Stack underflow at pc 0x0002"
    );
    assert_eq!(
        run_err(&[(0, &[0x6001u16, 0xF000])]).to_string(),
        "Opcode 0xf000 at pc 0x0002 is unrecognized or is not supported yet"
    );
    assert_eq!(
        run_err(&[(0, &[call(0xFFE)]), (0xFFE, &[load(V0, 1)])]).to_string(),
        "Program counter 0x1000 out of bounds"
    );
    assert!(
        run_err(&[(0, &[call(0)])])
            .to_string()
            .starts_with("Stack overflow: CALL 0x0000 at pc 0x0002")
    );
    assert_eq!(
        cpu(&[(0, &[load(V0, 1)])])
            .run_for(0)
            .unwrap_err()
            .to_string(),
        "Cycle limit reached before halting, at pc 0x0000"
    );
}

#[test]
fn new_cpu_is_running() {
    let c = cpu(&[(0, &[Halt])]);
    assert_eq!(c.state, CpuState::Running);
}

#[test]
fn step_runs_exactly_one_instruction() {
    let mut c = cpu(&[(0, &[load(V0, 5), add(V0, 3), Halt])]);

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
    let mut c = cpu(&[(0, &[Halt])]);
    c.step().unwrap();
    assert_eq!(c.state, CpuState::Halt);
}

#[test]
fn step_after_halt_stays_halted() {
    let mut c = cpu(&[(0, &[Halt, load(V0, 5)])]);
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
    let mut c = cpu(&[(0, &[call(0x100), Halt]), (0x100, &[Return])]);

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
    let mut c = cpu(&[(0, &[Return])]);
    assert!(matches!(c.step(), Err(CpuError::StackUnderflow { pc: 2 })));
    assert_eq!(c.state, CpuState::Running);
}

#[test]
fn stepping_matches_run() {
    let main = fn_main();
    let segs: &[(usize, &[Instruction])] = &[(0, &main), (0x100, &fn_add_twice())];

    let mut stepped = cpu(segs);
    let mut steps = 0;
    while stepped.state != CpuState::Halt {
        stepped.step().unwrap();
        steps += 1;
    }
    let loads = 2;
    let calls_with_two_adds_and_a_return = 2 * 4;
    let halt_step = 1;
    assert_eq!(steps, loads + calls_with_two_adds_and_a_return + halt_step);

    let mut ran = cpu(segs);
    ran.run().unwrap();
    assert_eq!(stepped.registers, ran.registers);
    assert_eq!(stepped.pc, ran.pc);
}

fn count_to_three_then_halt() -> [Instruction; 4] {
    [add(V0, 1), add(V0, 1), add(V0, 1), Halt]
}

#[test]
fn run_goes_until_halt() {
    let mut c = cpu(&[(0, &count_to_three_then_halt())]);
    c.run().unwrap();
    assert_eq!(c.get_register_value(V0), 3);
    assert_eq!(c.state, CpuState::Halt);
}

#[test]
fn run_returns_errors() {
    let mut c = cpu(&[(0, &[load(V0, 1), Return])]);
    assert!(matches!(c.run(), Err(CpuError::StackUnderflow { pc: 4 })));
}

#[test]
fn run_on_a_halted_cpu_does_nothing() {
    let mut c = cpu(&[(0, &[Halt, load(V0, 5)])]);
    c.run().unwrap();
    c.run().unwrap();
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn halt_on_the_last_allowed_cycle_is_ok() {
    let mut c = cpu(&[(0, &count_to_three_then_halt())]);
    c.run_for(4).unwrap();
    assert_eq!(c.get_register_value(V0), 3);
}

#[test]
fn one_cycle_short_of_halt_is_limit_reached() {
    let mut c = cpu(&[(0, &count_to_three_then_halt())]);
    assert!(matches!(
        c.run_for(3),
        Err(CpuError::CycleLimitReached { pc: 6 })
    ));
    assert_eq!(c.get_register_value(V0), 3);
    assert_eq!(c.state, CpuState::Running);
}

#[test]
fn zero_cycle_limit_runs_nothing() {
    let mut c = cpu(&[(0, &count_to_three_then_halt())]);
    assert!(matches!(
        c.run_for(0),
        Err(CpuError::CycleLimitReached { pc: 0 })
    ));
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn run_for_resumes_after_hitting_the_limit() {
    let mut c = cpu(&[(0, &count_to_three_then_halt())]);
    assert!(c.run_for(2).is_err());
    assert_eq!(c.get_register_value(V0), 2);
    c.run_for(2).unwrap();
    assert_eq!(c.get_register_value(V0), 3);
}

#[test]
fn run_for_on_a_halted_cpu_is_ok() {
    let mut c = cpu(&[(0, &[Halt, load(V0, 5)])]);
    c.run().unwrap();
    c.run_for(5).unwrap();
    assert_eq!(c.get_register_value(V0), 0);
}

#[test]
fn an_error_on_the_last_allowed_cycle_wins_over_the_limit() {
    let mut c = cpu(&[(0, &[load(V0, 1), Return])]);
    assert!(matches!(c.run_for(2), Err(CpuError::StackUnderflow { .. })));
}
