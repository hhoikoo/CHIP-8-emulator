#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    pub jump: JumpQuirk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JumpQuirk {
    #[default]
    AddV0,
    AddVx,
}
