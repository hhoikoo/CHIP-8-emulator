#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    pub jump: JumpQuirk,
    pub logic: LogicQuirk,
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
