#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    pub jump: JumpQuirk,
    pub logic: LogicQuirk,
    pub shift: ShiftQuirk,
    pub memory: MemoryQuirk,
    pub clip: ClipQuirk,
    pub display_wait: DisplayWaitQuirk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JumpQuirk {
    #[default]
    AddV0,
    AddVx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogicQuirk {
    #[default]
    ResetVf,
    KeepVf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShiftQuirk {
    #[default]
    ShiftVy,
    ShiftVxInPlace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemoryQuirk {
    #[default]
    AdvanceI,
    LeaveI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipQuirk {
    #[default]
    Clip,
    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayWaitQuirk {
    #[default]
    WaitForFrame,
    DrawImmediately,
}
