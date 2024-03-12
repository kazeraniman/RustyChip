use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum ResetVfQuirk {
    #[default]
    Reset,
    NoReset
}

#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum MemoryIncrementQuirk {
    #[default]
    Increment,
    NoIncrement
}

#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum DisplayWaitQuirk {
    #[default]
    Wait,
    NoWait
}

#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum ClippingQuirk {
    #[default]
    Clip,
    Wrap
}

#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum ShiftingQuirk {
    #[default]
    Vy,
    Vx,
}

#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum JumpingQuirk {
    #[default]
    V0,
    Vx
}

pub struct QuirkConfig {
    pub reset_vf: ResetVfQuirk,
    pub memory: MemoryIncrementQuirk,
    pub display_wait: DisplayWaitQuirk,
    pub clipping: ClippingQuirk,
    pub shifting: ShiftingQuirk,
    pub jumping: JumpingQuirk
}

impl QuirkConfig {
    #[must_use]
    pub fn new() -> QuirkConfig {
        QuirkConfig {
            reset_vf: ResetVfQuirk::default(),
            memory: MemoryIncrementQuirk::default(),
            display_wait: DisplayWaitQuirk::default(),
            clipping: ClippingQuirk::default(),
            shifting: ShiftingQuirk::default(),
            jumping: JumpingQuirk::default(),
        }
    }
}

impl Default for QuirkConfig {
    fn default() -> Self {
        QuirkConfig::new()
    }
}
