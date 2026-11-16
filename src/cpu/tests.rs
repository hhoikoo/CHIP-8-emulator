use crate::{
    examples::{FN_ADD_TWICE_ADDR, fn_add_twice, fn_main},
    isa::addr::Addr,
    memory::PROGRAM_START,
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

fn jump(addr: u16) -> Instruction {
    Jump {
        addr: Addr::new(addr).unwrap(),
    }
}

fn load_index(addr: u16) -> Instruction {
    LoadIndex {
        addr: Addr::new(addr).unwrap(),
    }
}

fn jump_with_offset(addr: u16) -> Instruction {
    JumpWithOffset {
        addr: Addr::new(addr).unwrap(),
    }
}

const CYCLE_LIMIT: usize = 10_000;
const START: usize = 0x200;

fn add_twice_addr() -> usize {
    usize::from(FN_ADD_TWICE_ADDR)
}

fn cpu<T: Copy + Into<Opcode>>(segs: &[(usize, &[T])]) -> Cpu {
    Cpu::new(&Rom::new(segs).unwrap())
}

fn cpu_with_quirks<T: Copy + Into<Opcode>>(segs: &[(usize, &[T])], quirks: Quirks) -> Cpu {
    Cpu::with_quirks(&Rom::new(segs).unwrap(), quirks)
}

fn looping(start: usize, instructions: &[Instruction]) -> Vec<Instruction> {
    let end = start + 2 * instructions.len();
    let mut program = instructions.to_vec();
    program.push(jump(u16::try_from(end).unwrap()));
    program
}

fn run_cycles(c: &mut Cpu) -> Result<(), CpuError> {
    c.run_for(CYCLE_LIMIT)
}

#[test]
fn new_cpu_has_zeroed_registers() {
    let c = cpu(&[(START, &looping(START, &[]))]);
    assert_eq!(c.registers, Registers::default());
}

#[test]
fn execution_starts_at_the_program_start() {
    let mut c = cpu(&[
        (0, &looping(0, &[load(V0, 1)])),
        (START, &looping(START, &[load(V0, 7)])),
    ]);
    assert_eq!(c.pc, ProgramCounter::from(PROGRAM_START));
    assert_eq!(usize::from(PROGRAM_START), START);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 7);
}

#[test]
fn code_in_the_last_memory_slot_runs() {
    let mut c = cpu(&[
        (START, &looping(START, &[call(0xFFE), load(V0, 1)])),
        (0xFFE, &[Return]),
    ]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 1);
}

#[test]
fn load_and_add_immediate() {
    let mut c = cpu(&[(START, &looping(START, &[load(V0, 5), add(V0, 3)]))]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 8);
}

#[test]
fn add_immediate_wraps() {
    let mut c = cpu(&[(START, &looping(START, &[load(V0, 0xFF), add(V0, 2)]))]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 1);
}

#[test]
fn add_with_carry_sets_vf() {
    let mut c = cpu(&[(
        START,
        &looping(
            START,
            &[load(V0, 0xF0), load(V1, 0x20), add_with_carry(V0, V1)],
        ),
    )]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 0x10);
    assert_eq!(c.get_register_value(VF), 1);
}

#[test]
fn add_with_carry_uses_the_named_registers() {
    let mut c = cpu(&[(
        START,
        &looping(START, &[load(VA, 3), load(VB, 4), add_with_carry(VA, VB)]),
    )]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(VA), 7);
    assert_eq!(c.get_register_value(VB), 4);
    assert_eq!(c.get_register_value(V0), 0);
    assert_eq!(c.get_register_value(VF), 0);
}

#[test]
fn add_with_carry_into_vf_keeps_the_flag() {
    let mut c = cpu(&[(
        START,
        &looping(
            START,
            &[load(VF, 0xFF), load(V0, 2), add_with_carry(VF, V0)],
        ),
    )]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(VF), 1);
}

#[test]
fn add_with_carry_clears_a_stale_flag() {
    let mut c = cpu(&[(
        START,
        &looping(
            START,
            &[
                load(VF, 1),
                load(V0, 1),
                load(V1, 1),
                add_with_carry(V0, V1),
            ],
        ),
    )]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 2);
    assert_eq!(c.get_register_value(VF), 0);
}

#[test]
fn the_real_main_program_via_two_segments() {
    let main = fn_main(PROGRAM_START);
    let mut c = cpu(&[(START, &main), (add_twice_addr(), &fn_add_twice())]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 45);
}

#[test]
fn segment_order_in_literal_does_not_matter() {
    let main = fn_main(PROGRAM_START);
    let mut c = cpu(&[(add_twice_addr(), &fn_add_twice()), (START, &main)]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 45);
}

fn run_err<T: Copy + Into<Opcode>>(segs: &[(usize, &[T])]) -> CpuError {
    cpu(segs).run_for(CYCLE_LIMIT).unwrap_err()
}

#[test]
fn ret_with_empty_stack_is_underflow_at_the_next_pc() {
    let err = run_err(&[(START, &[Return])]);
    assert!(matches!(err, CpuError::StackUnderflow { pc } if pc.get() == 0x202));
}

#[test]
fn infinite_recursion_is_stack_overflow() {
    let err = run_err(&[(START, &[call(0x200)])]);
    match err {
        CpuError::StackOverflow { pc, addr, stack } => {
            assert_eq!((pc.get(), addr.get()), (0x202, 0x200));
            assert_eq!(stack, vec![ProgramCounter::new(0x202); STACK_SIZE]);
        }
        other => panic!("expected StackOverflow, got {other:?}"),
    }
}

fn function_addr(index: usize) -> usize {
    START + index * 0x10
}

fn call_function(index: usize) -> Instruction {
    call(u16::try_from(function_addr(index)).unwrap())
}

fn run_call_chain(depth: usize) -> Result<(), CpuError> {
    let main = looping(START, &[call_function(1)]);
    let callers: Vec<[Instruction; 2]> = (1..depth)
        .map(|index| [call_function(index + 1), Return])
        .collect();
    let deepest = [Return];

    let mut segs: Vec<(usize, &[Instruction])> = vec![(START, &main)];
    segs.extend(
        callers
            .iter()
            .zip(1..)
            .map(|(function, index)| (function_addr(index), &function[..])),
    );
    segs.push((function_addr(depth), &deepest));

    run_cycles(&mut cpu(&segs))
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
    let err = run_err(&[(START, &[0x6001u16, 0xF000])]);
    assert!(matches!(
        err,
        CpuError::UnknownOpcode { opcode, pc } if opcode == Opcode::new(0xF000) && pc.get() == 0x202
    ));
}

#[test]
fn running_into_zeroed_memory_is_an_unknown_opcode() {
    let err = run_err(&[(START, &[load(V0, 1)])]);
    assert!(matches!(
        err,
        CpuError::UnknownOpcode { opcode, pc } if opcode == Opcode::new(0x0000) && pc.get() == 0x202
    ));
}

#[test]
fn running_off_the_end_of_memory_is_pc_out_of_bounds() {
    let err = run_err(&[(START, &[call(0xFFE)]), (0xFFE, &[load(V0, 1)])]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc } if pc.get() == 0x1000));
}

#[test]
fn odd_pc_on_the_last_byte_is_pc_out_of_bounds() {
    let err = run_err(&[(START, &[call(0xFFF)])]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc } if pc.get() == 0xFFF));
}

#[test]
fn new_cpu_uses_the_original_quirks() {
    let c = cpu(&[(START, &looping(START, &[]))]);
    assert_eq!(c.quirks, Quirks::default());
    assert_eq!(c.quirks.jump, JumpQuirk::AddV0);
}

#[test]
fn jump_skips_the_instructions_in_between() {
    let mut c = cpu(&[
        (START, &looping(START, &[jump(0x206), load(V0, 1)])),
        (0x206, &looping(0x206, &[load(V1, 7)])),
    ]);
    run_cycles(&mut c).unwrap();
    assert_eq!(c.get_register_value(V0), 0);
    assert_eq!(c.get_register_value(V1), 7);
}

#[test]
fn jump_to_itself_keeps_the_pc_in_place() {
    let mut c = cpu(&[(START, &[jump(0x200)])]);
    c.run_for(10).unwrap();
    assert_eq!(c.pc.get(), 0x200);
}

fn va_after_jump_with_offset(quirks: Quirks) -> u8 {
    let main = [load(V0, 0x04), load(V1, 0x10), jump_with_offset(0x120)];
    let target_with_v0 = looping(0x124, &[load(VA, 1)]);
    let target_with_v1 = looping(0x030, &[load(VA, 2)]);
    let mut c = cpu_with_quirks(
        &[
            (START, &main),
            (0x124, &target_with_v0),
            (0x030, &target_with_v1),
        ],
        quirks,
    );
    run_cycles(&mut c).unwrap();
    c.get_register_value(VA)
}

#[test]
fn jump_with_offset_adds_v0_by_default() {
    assert_eq!(va_after_jump_with_offset(Quirks::default()), 1);
}

#[test]
fn jump_with_offset_adds_vx_with_the_vx_quirk() {
    let quirks = Quirks {
        jump: JumpQuirk::AddVx,
        ..Quirks::default()
    };
    assert_eq!(va_after_jump_with_offset(quirks), 2);
}

#[test]
fn jump_with_offset_past_the_end_of_memory_is_pc_out_of_bounds() {
    let err = run_err(&[(START, &[load(V0, 0xFF), jump_with_offset(0xFFF)])]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc } if pc.get() == 0x10FE));
}

#[test]
fn jump_with_offset_with_the_vx_quirk_adds_vx_to_the_low_byte() {
    let quirks = Quirks {
        jump: JumpQuirk::AddVx,
        ..Quirks::default()
    };
    let mut c = cpu_with_quirks(
        &[(START, &[load(VF, 0xFF), jump_with_offset(0xFFF)])],
        quirks,
    );
    c.step().unwrap();
    c.step().unwrap();
    assert_eq!(c.pc.get(), 0x1FE);
}

#[test]
fn error_messages() {
    assert_eq!(
        run_err(&[(START, &[Return])]).to_string(),
        "Stack underflow at pc 0x0202"
    );
    assert_eq!(
        run_err(&[(START, &[0x6001u16, 0xF000])]).to_string(),
        "Opcode 0xf000 at pc 0x0202 is unrecognized or is not supported yet"
    );
    assert_eq!(
        run_err(&[(START, &[call(0xFFE)]), (0xFFE, &[load(V0, 1)])]).to_string(),
        "Program counter 0x1000 out of bounds"
    );
    assert!(
        run_err(&[(START, &[call(0x200)])])
            .to_string()
            .starts_with("Stack overflow: CALL 0x0200 at pc 0x0202")
    );
    assert_eq!(
        CpuError::IndexOutOfBounds {
            pc: ProgramCounter::new(0x204),
            i: 0xFFE,
            len: 3,
        }
        .to_string(),
        "Memory access of 3 bytes at I = 0x0ffe (pc 0x0204) runs past the end of memory"
    );
}

#[test]
fn step_runs_exactly_one_instruction() {
    let mut c = cpu(&[(START, &looping(START, &[load(V0, 5), add(V0, 3)]))]);

    c.step().unwrap();
    assert_eq!(c.get_register_value(V0), 5);
    assert_eq!(c.pc.get(), 0x202);

    c.step().unwrap();
    assert_eq!(c.get_register_value(V0), 8);
    assert_eq!(c.pc.get(), 0x204);
}

#[test]
fn step_through_call_and_return() {
    let mut c = cpu(&[(START, &looping(START, &[call(0x300)])), (0x300, &[Return])]);

    c.step().unwrap();
    assert_eq!(c.pc.get(), 0x300);
    assert_eq!(c.stack_ptr, 1);

    c.step().unwrap();
    assert_eq!(c.pc.get(), 0x202);
    assert_eq!(c.stack_ptr, 0);

    c.step().unwrap();
    assert_eq!(c.pc.get(), 0x202);
}

#[test]
fn step_returns_errors() {
    let mut c = cpu(&[(START, &[Return])]);
    assert!(matches!(c.step(), Err(CpuError::StackUnderflow { pc }) if pc.get() == 0x202));
}

#[test]
fn run_returns_errors() {
    let mut c = cpu(&[(START, &[load(V0, 1), Return])]);
    assert!(matches!(c.run(|| false), Err(CpuError::StackUnderflow { pc }) if pc.get() == 0x204));
}

#[test]
fn run_stops_when_asked() {
    let mut c = cpu(&[(START, &count_to_three())]);
    let mut checks = 0;
    c.run(|| {
        checks += 1;
        checks > 2
    })
    .unwrap();
    assert_eq!(c.get_register_value(V0), 2);
}

#[test]
fn run_checks_before_the_first_step() {
    let mut c = cpu(&[(START, &count_to_three())]);
    c.run(|| true).unwrap();
    assert_eq!(c.pc.get(), 0x200);
}

fn count_to_three() -> Vec<Instruction> {
    looping(START, &[add(V0, 1), add(V0, 1), add(V0, 1)])
}

#[test]
fn run_for_runs_exactly_the_given_cycles() {
    let mut c = cpu(&[(START, &count_to_three())]);
    c.run_for(2).unwrap();
    assert_eq!(c.get_register_value(V0), 2);
    assert_eq!(c.pc.get(), 0x204);
}

#[test]
fn run_for_zero_cycles_runs_nothing() {
    let mut c = cpu(&[(START, &count_to_three())]);
    c.run_for(0).unwrap();
    assert_eq!(c.get_register_value(V0), 0);
    assert_eq!(c.pc.get(), 0x200);
}

#[test]
fn run_for_resumes_where_it_stopped() {
    let mut c = cpu(&[(START, &count_to_three())]);
    c.run_for(2).unwrap();
    c.run_for(2).unwrap();
    assert_eq!(c.get_register_value(V0), 3);
    assert_eq!(c.pc.get(), 0x206);
}

#[test]
fn run_for_returns_an_error_on_its_last_cycle() {
    let mut c = cpu(&[(START, &[load(V0, 1), Return])]);
    assert!(matches!(
        c.run_for(2),
        Err(CpuError::StackUnderflow { pc }) if pc.get() == 0x204
    ));
}

#[test]
fn the_main_program_reaches_its_loop_after_ten_steps() {
    let main = fn_main(PROGRAM_START);
    let mut c = cpu(&[(START, &main), (add_twice_addr(), &fn_add_twice())]);
    let loads = 2;
    let calls_with_two_adds_and_a_return = 2 * 4;
    c.run_for(loads + calls_with_two_adds_and_a_return).unwrap();
    assert_eq!(c.get_register_value(V0), 45);
    assert_eq!(c.pc.get(), 0x208);
}

fn skipped(setup: &[Instruction], skip: Instruction) -> bool {
    let mut program = setup.to_vec();
    program.extend([skip, load(VE, 1), load(VD, 1)]);
    let mut c = cpu(&[(START, &looping(START, &program))]);
    run_cycles(&mut c).unwrap();
    assert_eq!(
        c.get_register_value(VD),
        1,
        "the instruction after the skip runs"
    );
    c.get_register_value(VE) == 0
}

#[test]
fn skip_if_equal_immediate() {
    let setup = [load(V3, 7)];
    assert!(skipped(&setup, SkipIfEqualImmediate { x: V3, nn: 7 }));
    assert!(!skipped(&setup, SkipIfEqualImmediate { x: V3, nn: 8 }));
}

#[test]
fn skip_if_not_equal_immediate() {
    let setup = [load(V3, 7)];
    assert!(skipped(&setup, SkipIfNotEqualImmediate { x: V3, nn: 8 }));
    assert!(!skipped(&setup, SkipIfNotEqualImmediate { x: V3, nn: 7 }));
}

#[test]
fn skip_if_equal() {
    let equal = [load(V3, 7), load(V4, 7)];
    let different = [load(V3, 7), load(V4, 8)];
    assert!(skipped(&equal, SkipIfEqual { x: V3, y: V4 }));
    assert!(!skipped(&different, SkipIfEqual { x: V3, y: V4 }));
}

#[test]
fn skip_if_not_equal() {
    let equal = [load(V3, 7), load(V4, 7)];
    let different = [load(V3, 7), load(V4, 8)];
    assert!(skipped(&different, SkipIfNotEqual { x: V3, y: V4 }));
    assert!(!skipped(&equal, SkipIfNotEqual { x: V3, y: V4 }));
}

#[test]
fn skip_compares_the_named_registers() {
    let setup = [load(V3, 7), load(V4, 8), load(V5, 7)];
    assert!(skipped(&setup, SkipIfEqual { x: V3, y: V5 }));
    assert!(!skipped(&setup, SkipIfEqual { x: V3, y: V4 }));
}

#[test]
fn skip_on_the_last_instruction_is_pc_out_of_bounds() {
    let err = run_err(&[
        (START, &[jump(0xFFE)]),
        (0xFFE, &[SkipIfEqualImmediate { x: V0, nn: 0 }]),
    ]);
    assert!(matches!(err, CpuError::PcOutOfBounds { pc } if pc.get() == 0x1002));
}

fn registers_after(program: &[Instruction], quirks: Quirks) -> Registers {
    let mut c = cpu_with_quirks(&[(START, &looping(START, program))], quirks);
    run_cycles(&mut c).unwrap();
    c.registers
}

fn keep_vf() -> Quirks {
    Quirks {
        logic: LogicQuirk::KeepVf,
        ..Quirks::default()
    }
}

#[test]
fn new_cpu_resets_vf_on_logic_by_default() {
    assert_eq!(Quirks::default().logic, LogicQuirk::ResetVf);
}

#[test]
fn load_register_copies_vy_into_vx() {
    let r = registers_after(
        &[load(V1, 0x2A), load(VF, 7), LoadRegister { x: V0, y: V1 }],
        Quirks::default(),
    );
    assert_eq!(r[V0], 0x2A);
    assert_eq!(r[V1], 0x2A);
    assert_eq!(r[VF], 7);
}

#[test]
fn bitwise_operations() {
    let setup = [load(V0, 0b1100), load(V1, 0b1010)];
    let cases = [
        (Or { x: V0, y: V1 }, 0b1110),
        (And { x: V0, y: V1 }, 0b1000),
        (Xor { x: V0, y: V1 }, 0b0110),
    ];
    for (operation, expected) in cases {
        let mut program = setup.to_vec();
        program.push(operation);
        let r = registers_after(&program, Quirks::default());
        assert_eq!(r[V0], expected, "{operation:?}");
        assert_eq!(r[V1], 0b1010, "{operation:?}");
    }
}

fn bitwise_with_vf_set(operation: Instruction, quirks: Quirks) -> u8 {
    let program = [load(V0, 0b1100), load(V1, 0b1010), load(VF, 7), operation];
    registers_after(&program, quirks)[VF]
}

#[test]
fn bitwise_operations_reset_vf_by_default() {
    for operation in [
        Or { x: V0, y: V1 },
        And { x: V0, y: V1 },
        Xor { x: V0, y: V1 },
    ] {
        assert_eq!(
            bitwise_with_vf_set(operation, Quirks::default()),
            0,
            "{operation:?}"
        );
    }
}

#[test]
fn bitwise_operations_keep_vf_with_the_keep_quirk() {
    for operation in [
        Or { x: V0, y: V1 },
        And { x: V0, y: V1 },
        Xor { x: V0, y: V1 },
    ] {
        assert_eq!(
            bitwise_with_vf_set(operation, keep_vf()),
            7,
            "{operation:?}"
        );
    }
}

#[test]
fn load_register_never_resets_vf() {
    let program = [load(VF, 7), LoadRegister { x: V0, y: V1 }];
    assert_eq!(registers_after(&program, Quirks::default())[VF], 7);
}

#[test]
fn bitwise_into_vf_ends_with_the_quirk_result() {
    let program = [load(VF, 0b0001), load(V1, 0b0110), Or { x: VF, y: V1 }];
    assert_eq!(registers_after(&program, Quirks::default())[VF], 0);
    assert_eq!(registers_after(&program, keep_vf())[VF], 0b0111);
}

fn shift_vx_in_place() -> Quirks {
    Quirks {
        shift: ShiftQuirk::ShiftVxInPlace,
        ..Quirks::default()
    }
}

fn after(program: &[Instruction]) -> Registers {
    registers_after(program, Quirks::default())
}

fn index_after(program: &[Instruction]) -> u16 {
    let mut c = cpu(&[(START, &looping(START, program))]);
    run_cycles(&mut c).unwrap();
    c.i
}

#[test]
fn new_cpu_has_a_zeroed_index() {
    assert_eq!(cpu(&[(START, &looping(START, &[]))]).i, 0);
}

#[test]
fn load_index_sets_i() {
    assert_eq!(index_after(&[load_index(0xABC)]), 0xABC);
    assert_eq!(index_after(&[load_index(0x123), load_index(0x456)]), 0x456);
}

#[test]
fn load_index_leaves_the_registers_alone() {
    assert_eq!(after(&[load_index(0xFFF)]), Registers::default());
}

#[test]
fn add_to_index_adds_vx() {
    assert_eq!(
        index_after(&[load(V3, 0x10), load_index(0x300), AddToIndex { x: V3 }]),
        0x310
    );
}

#[test]
fn add_to_index_can_go_past_the_end_of_memory() {
    assert_eq!(
        index_after(&[load(V0, 0xFF), load_index(0xFFF), AddToIndex { x: V0 }]),
        0x10FE
    );
}

#[test]
fn add_to_index_leaves_vf_alone() {
    let r = after(&[
        load(VF, 7),
        load(V0, 0xFF),
        load_index(0xFFF),
        AddToIndex { x: V0 },
    ]);
    assert_eq!(r[VF], 7);
    let r = after(&[load(V0, 1), AddToIndex { x: V0 }]);
    assert_eq!(r[VF], 0);
}

#[test]
fn add_to_index_wraps_at_16_bits() {
    let mut c = cpu(&[(
        START,
        &looping(START, &[load(V0, 0x20), AddToIndex { x: V0 }]),
    )]);
    c.i = 0xFFF0;
    c.run_for(2).unwrap();
    assert_eq!(c.i, 0x0010);
}

#[test]
fn new_cpu_shifts_vy_by_default() {
    assert_eq!(Quirks::default().shift, ShiftQuirk::ShiftVy);
}

#[test]
fn subtract_without_borrow_sets_vf() {
    let r = after(&[
        load(V0, 9),
        load(V1, 4),
        SubtractWithBorrow { x: V0, y: V1 },
    ]);
    assert_eq!((r[V0], r[V1], r[VF]), (5, 4, 1));
}

#[test]
fn subtract_with_borrow_wraps_and_clears_vf() {
    let r = after(&[
        load(VF, 1),
        load(V0, 4),
        load(V1, 9),
        SubtractWithBorrow { x: V0, y: V1 },
    ]);
    assert_eq!((r[V0], r[VF]), (0xFB, 0));
}

#[test]
fn subtract_equal_values_is_no_borrow() {
    let r = after(&[
        load(V0, 7),
        load(V1, 7),
        SubtractWithBorrow { x: V0, y: V1 },
    ]);
    assert_eq!((r[V0], r[VF]), (0, 1));
}

#[test]
fn subtract_reversed_computes_vy_minus_vx() {
    let r = after(&[load(V0, 4), load(V1, 9), SubtractReversed { x: V0, y: V1 }]);
    assert_eq!((r[V0], r[V1], r[VF]), (5, 9, 1));
}

#[test]
fn subtract_reversed_with_borrow_wraps_and_clears_vf() {
    let r = after(&[
        load(VF, 1),
        load(V0, 9),
        load(V1, 4),
        SubtractReversed { x: V0, y: V1 },
    ]);
    assert_eq!((r[V0], r[VF]), (0xFB, 0));
}

#[test]
fn subtract_reversed_equal_values_is_no_borrow() {
    let r = after(&[load(V0, 7), load(V1, 7), SubtractReversed { x: V0, y: V1 }]);
    assert_eq!((r[V0], r[VF]), (0, 1));
}

#[test]
fn subtract_into_vf_keeps_the_flag() {
    let borrow = after(&[
        load(VF, 1),
        load(V0, 2),
        SubtractWithBorrow { x: VF, y: V0 },
    ]);
    assert_eq!(borrow[VF], 0);
    let no_borrow = after(&[
        load(VF, 5),
        load(V0, 2),
        SubtractWithBorrow { x: VF, y: V0 },
    ]);
    assert_eq!(no_borrow[VF], 1);
    let reversed = after(&[load(VF, 2), load(V0, 5), SubtractReversed { x: VF, y: V0 }]);
    assert_eq!(reversed[VF], 1);
}

#[test]
fn shift_right_shifts_vy_into_vx_by_default() {
    let r = after(&[load(V0, 0xFF), load(V1, 0b101), ShiftRight { x: V0, y: V1 }]);
    assert_eq!((r[V0], r[V1], r[VF]), (0b10, 0b101, 1));
}

#[test]
fn shift_left_shifts_vy_into_vx_by_default() {
    let r = after(&[
        load(V0, 0xFF),
        load(V1, 0b1000_0001),
        ShiftLeft { x: V0, y: V1 },
    ]);
    assert_eq!((r[V0], r[V1], r[VF]), (0b10, 0b1000_0001, 1));
}

#[test]
fn shifts_clear_vf_when_the_shifted_out_bit_is_zero() {
    let right = after(&[load(VF, 1), load(V1, 0b10), ShiftRight { x: V0, y: V1 }]);
    assert_eq!((right[V0], right[VF]), (1, 0));
    let left = after(&[
        load(VF, 1),
        load(V1, 0b0100_0000),
        ShiftLeft { x: V0, y: V1 },
    ]);
    assert_eq!((left[V0], left[VF]), (0b1000_0000, 0));
}

#[test]
fn shifts_with_the_in_place_quirk_ignore_vy() {
    let program = [
        load(V0, 0b1000_0011),
        load(V1, 0),
        ShiftRight { x: V0, y: V1 },
    ];
    let r = registers_after(&program, shift_vx_in_place());
    assert_eq!((r[V0], r[V1], r[VF]), (0b0100_0001, 0, 1));

    let program = [
        load(V0, 0b1000_0011),
        load(V1, 0),
        ShiftLeft { x: V0, y: V1 },
    ];
    let r = registers_after(&program, shift_vx_in_place());
    assert_eq!((r[V0], r[V1], r[VF]), (0b0000_0110, 0, 1));
}

#[test]
fn shift_into_vf_keeps_the_flag() {
    let right = after(&[load(V1, 0b10), ShiftRight { x: VF, y: V1 }]);
    assert_eq!(right[VF], 0);
    let left = after(&[load(V1, 0b1000_0000), ShiftLeft { x: VF, y: V1 }]);
    assert_eq!(left[VF], 1);
    let in_place = registers_after(
        &[load(VF, 0b11), ShiftRight { x: VF, y: V1 }],
        shift_vx_in_place(),
    );
    assert_eq!(in_place[VF], 1);
}
