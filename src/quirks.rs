//! A module to contain the types related to the quirk configuration.  
//! For more information on CHIP-8 quirks, please see [this section](https://github.com/Timendus/chip8-test-suite#quirks-test) of the test suite.

use clap::ValueEnum;

/// Denotes the enabled/disabled status of the reset register F quirk.  
/// This quirk can cause the AND, OR, and XOR opcodes to reset the value of register F.
#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum ResetVfQuirk {
    #[default]
    Reset,
    NoReset
}

/// Denotes the enabled/disabled status of the store/load registers opcodes' register I increment quirk.  
/// This quirk can cause the store/load registers opcodes to increment register I as they operate. 
#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum MemoryIncrementQuirk {
    #[default]
    Increment,
    NoIncrement
}

/// Denotes the enabled/disabled status of the display wait quirk.  
/// This quirk can cause the draw opcode to wait for a screen refresh prior to drawing to prevent partial draws.
#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum DisplayWaitQuirk {
    #[default]
    Wait,
    NoWait
}

/// Denotes the enabled/disabled status of the clipping quirk.  
/// This quirk can cause the draw opcode to either clip sprites drawn on the edges or have them wrap around the screen.
#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum ClippingQuirk {
    #[default]
    Clip,
    Wrap
}

/// Denotes the enabled/disabled status of the shifting quirk.  
/// This quirk can cause the shift register opcodes to operate on a single register or on a second one while storing the result in the first.
#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum ShiftingQuirk {
    #[default]
    Vy,
    Vx,
}

/// Denotes the enabled/disabled status of the jumping quirk.  
/// This quirk can cause the jump to address + register 0 opcode to operate on a different register instead.
#[derive(Debug, Clone, PartialEq, ValueEnum, Default)]
pub enum JumpingQuirk {
    #[default]
    V0,
    Vx
}

/// Stores all the quirk settings together.
pub struct QuirkConfig {
    pub reset_vf: ResetVfQuirk,
    pub memory: MemoryIncrementQuirk,
    pub display_wait: DisplayWaitQuirk,
    pub clipping: ClippingQuirk,
    pub shifting: ShiftingQuirk,
    pub jumping: JumpingQuirk
}

impl QuirkConfig {
    /// Returns a new `QuirkConfig` with default values for all members.
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
