use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum ResetVfQuirk {
    #[default]
    Reset,
    NoReset
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum MemoryIncrementQuirk {
    #[default]
    Increment,
    NoIncrement
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum DisplayWaitQuirk {
    #[default]
    Wait,
    NoWait
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum ClippingQuirk {
    #[default]
    Clip,
    Wrap
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum ShiftingQuirk {
    #[default]
    Vy,
    Vx,
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum JumpingQuirk {
    #[default]
    V0,
    Vx
}
