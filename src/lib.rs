use std::{io, fs};

const CLEAR_SCREEN_OPCODE: [u8; 2] = [0x00, 0xE0];
const RETURN_OPCODE: [u8; 2] = [0x00, 0xEE];

#[derive(PartialEq, Debug)]
enum Opcode {
    SystemAddr(u16),
    ClearScreen,
    Return,
    JumpAddr(u16),
    CallAddr(u16),
    SkipRegisterEqualValue(u8, u8),
    SkipRegisterNotEqualValue(u8, u8),
    SkipRegistersEqual(u8, u8),
    LoadValue(u8, u8),
    AddValue(u8, u8),
    LoadRegisterValue(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    AddRegisters(u8, u8),
    SubtractFromFirstRegister(u8, u8),
    BitShiftRight(u8, u8),
    SubtractFromSecondRegister(u8, u8),
    BitShiftLeft(u8, u8),
    SkipRegistersNotEqual(u8, u8),
    SetRegisterI(u16),
    JumpAddrV0(u16),
    Random(u8, u8),
    Draw(u8, u8, u8),
    SkipKeyPressed(u8),
    SkipKeyNotPressed(u8),
    LoadDelayTimer(u8),
    LoadKeyPress(u8),
    SetDelayTimer(u8),
    SetSoundTimer(u8),
    AddI(u8),
    SetIHexSpriteLocation(u8),
    BinaryCodedDecimal(u8),
    StoreRegisters(u8),
    LoadRegisters(u8)
}

pub fn read_game_file(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

fn get_opcode(opcode_bytes: &[u8]) -> Opcode {
    if opcode_bytes.len() != 2 {
        panic!("Opcodes must be two bytes.");
    }

    if opcode_bytes == CLEAR_SCREEN_OPCODE {
        return Opcode::ClearScreen;
    }

    if opcode_bytes == RETURN_OPCODE {
        return Opcode::Return;
    }

    panic!("Unrecognized opcode: {:0>2X?}{:0>2X?}", opcode_bytes[0], opcode_bytes[1])
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

    #[test]
    #[should_panic]
    fn get_improper_opcode() {
        let improper_opcode = [0x0];
        get_opcode(&improper_opcode[0..1]);
    }

    #[test]
    fn get_sys_addr_opcode() {
        let opcode = [0x0A, 0x78];
        let opcode = get_opcode(&opcode[..]);
        assert_eq!(opcode, Opcode::SystemAddr(0xA78));
    }

    #[test]
    fn get_clear_screen_opcode() {
        let opcode = CLEAR_SCREEN_OPCODE;
        let opcode = get_opcode(&opcode[..]);
        assert_eq!(opcode, Opcode::ClearScreen);
    }

    #[test]
    fn get_return_opcode() {
        let opcode = RETURN_OPCODE;
        let opcode = get_opcode(&opcode[..]);
        assert_eq!(opcode, Opcode::Return);
    }
}
