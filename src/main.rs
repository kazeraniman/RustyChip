use std::process;
use clap::Parser;
use rusty_chip::quirks::{ResetVfQuirk, MemoryIncrementQuirk, DisplayWaitQuirk, ClippingQuirk, ShiftingQuirk, JumpingQuirk};

const CYCLES_PER_FRAME: u32 = 10;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long_help = "Path to the game file.")]
    game: String,

    #[arg(short, long, default_value_t = CYCLES_PER_FRAME, long_help = "The number of instructions that will run in a single frame.")]
    cycles_per_frame: u32,

    // Quirk flags
    #[arg(long, default_value_t, value_enum, long_help = "True if the AND, OR, and XOR opcodes should reset the flags register to 0, false if the flag register should be untouched.")]
    quirk_reset_vf: ResetVfQuirk,
    #[arg(long, default_value_t, value_enum, long_help = "True if the save and load register opcodes should increment the index register, false if the index register should be untouched.")]
    quirk_memory: MemoryIncrementQuirk,
    #[arg(long, default_value_t, value_enum, long_help = "True if the draw opcode should wait for a frame draw before writing, false if it should draw immediately even if it should result in partial sprite draws.")]
    quirk_display_wait: DisplayWaitQuirk,
    #[arg(long, default_value_t, value_enum, long_help = "True if the draw opcode clip sprites going off the screen and wrap sprites which are fully off the screen, false if all sprites should wrap.")]
    quirk_clipping: ClippingQuirk,
    #[arg(long, default_value_t, value_enum, long_help = "True if the bit shift opcodes should operate on vX, false if they should operate on vY and store the result in vX.")]
    quirk_shifting: ShiftingQuirk,
    #[arg(long, default_value_t, value_enum, long_help = "True if the jump v0 opcode should use vX instead (the highest nibble of nnn), false if it should use v0.")]
    quirk_jumping: JumpingQuirk,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = rusty_chip::run(cli.game, cli.cycles_per_frame, cli.quirk_reset_vf, cli.quirk_memory, cli.quirk_display_wait, cli.quirk_clipping, cli.quirk_shifting, cli.quirk_jumping) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
