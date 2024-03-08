use std::process;

fn main() {
    if let Err(e) = rusty_chip::run() {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
