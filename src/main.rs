fn main() {
    let game_file = rusty_chip::read_game_file("games/BLINKY.chip8").expect("It should find this.");
    for b in game_file.iter() {
        println!("{:0>2X?}", b);
    }
}
