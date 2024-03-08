use std::{io, fs, time::Duration};
use sdl2::{pixels::Color, event::Event, keyboard::Keycode};
use interpreter::Interpreter;

pub mod opcodes;
pub mod interpreter;

pub fn run() -> Result<(), String> {
    // Create the emulator and load the game
    let game_file = read_game_file("games/INVADERS.chip8")
        .map_err(|err| err.to_string())?;
    let mut interpreter = Interpreter::new();
    interpreter.load_game(game_file);

    // Initialize SDL
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Create the window
    let window = video_subsystem.window("RustyChip", 800, 600)
        .position_centered()
        .build()
        .map_err(|window_build_error| window_build_error.to_string())?;

    // Prepare the canvas
    let mut canvas = window.into_canvas()
        .build()
        .map_err(|integer_or_sdl_error| integer_or_sdl_error.to_string())?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // Prepare for events
    let mut event_pump = sdl_context.event_pump()?;

    // The main game loop
    'game_loop: loop {
        // Go through each event and handle them
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'game_loop;
                },
                _ => {}
            }
        }

        // Run the interpreter logic
        interpreter.handle_cycle();

        // Wait the requisite time for the next iteration. Effectively sets it to 60fps / 60Hz.
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    // Return success
    Ok(())
}

fn read_game_file(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_existing_game_file() {
        assert!(read_game_file("games/15PUZZLE.chip8").is_ok());
    }

    #[test]
    fn read_non_existing_game_file() {
        assert!(read_game_file("games/FAKE.chip8").is_err());
    }
}
