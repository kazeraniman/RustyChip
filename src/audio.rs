//! A module to contain minimal audio functionality of the emulator.  
//! This is taken from the example provided by the SDL2 crate.  
//! Minor modifications made for access from another file.  
//! Web-viewable documentation [here](https://docs.rs/sdl2/latest/sdl2/audio/index.html).

use sdl2::audio::AudioCallback;

/// Stores the information to produce a square wave.
pub struct SquareWave {
    pub phase_inc: f32,
    pub phase: f32,
    pub volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    /// Generates a square wave.
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
