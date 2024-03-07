fn main() {
    let game_file = rusty_chip::read_game_file("games/INVADERS.chip8").expect("It should find this.");
    for w in game_file.windows(2) {
        println!("{:0>2X?}{:0>2X?}", w[0], w[1]);
    }
}
