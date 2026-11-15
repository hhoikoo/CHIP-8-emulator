#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    pub jump: JumpQuirk,
    pub logic: LogicQuirk,
    pub shift: ShiftQuirk,
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
