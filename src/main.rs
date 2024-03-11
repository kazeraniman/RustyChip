use std::process;
use clap::Parser;

const CYCLES_PER_FRAME: u32 = 10;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long_help = "Path to the game file.")]
    game: String,

    #[arg(short, long, default_value_t = CYCLES_PER_FRAME, long_help = "The number of instructions that will run in a single frame.")]
    cycles_per_frame: u32,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = rusty_chip::run(cli.game, cli.cycles_per_frame) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
