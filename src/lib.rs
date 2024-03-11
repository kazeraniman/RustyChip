use std::{io, fs, time::Duration};
use sdl2::{event::Event, keyboard::Keycode};
use interpreter::Interpreter;
use sdl2::audio::{AudioSpecDesired};
use audio::SquareWave;

pub mod opcodes;
pub mod interpreter;
pub mod audio;

pub fn run(path: String, cycles_per_frame: u32) -> Result<(), String> {
    // Read the game file
    let game_file = read_game_file(&path)
        .map_err(|err| err.to_string())?;

    // Initialize SDL
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Create the window
    let window = video_subsystem.window("RustyChip", interpreter::SCALED_WIDTH, interpreter::SCALED_HEIGHT)
        .position_centered()
        .build()
        .map_err(|window_build_error| window_build_error.to_string())?;

    // Prepare the canvas
    let mut canvas = window.into_canvas()
        .build()
        .map_err(|integer_or_sdl_error| integer_or_sdl_error.to_string())?;

    // Prepare the audio
    // Mostly taken from the example provided by the crate
    let audio_subsystem = sdl_context.audio()?;
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };
    let audio_device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    })?;

    // Prepare for events
    let mut event_pump = sdl_context.event_pump()?;

    // Prepare the emulator
    let mut interpreter = Interpreter::new_with_sdl(Some(&mut canvas), Some(&audio_device));
    interpreter.load_game(game_file);

    // The main game loop
    'game_loop: loop {
        // Go through each event and handle them
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'game_loop;
                },
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    interpreter.handle_key_press(keycode);
                },
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    interpreter.handle_key_release(keycode);
                },
                _ => {}
            }
        }

        // Run the interpreter logic
        for _ in 0..cycles_per_frame {
            interpreter.handle_cycle();
        }

        // Draw the frame
        interpreter.handle_frame();

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
