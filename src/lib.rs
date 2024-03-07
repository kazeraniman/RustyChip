use std::{io, fs};

pub fn read_game_file(path: &str) -> io::Result<Vec<u8>> {
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
