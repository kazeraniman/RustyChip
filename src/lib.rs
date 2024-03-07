use std::{io, fs};

const CLEAR_SCREEN_OPCODE: [u8; 2] = [0x00, 0xE0];
const RETURN_OPCODE: [u8; 2] = [0x00, 0xEE];
const LOWER_NIBBLE_MASK: u8 = 0xF;
const UPPER_NIBBLE_MASK: u8 = 0xF0;

#[derive(PartialEq, Debug)]
enum Opcode {
    SystemAddr(u16),
    ClearScreen,
    Return,
    JumpAddr(u16),
    CallAddr(u16),
    SkipRegisterEqualsValue(u8, u8),
    SkipRegisterNotEqualsValue(u8, u8),
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
    AddRegisterI(u8),
    SetIHexSpriteLocation(u8),
    BinaryCodedDecimal(u8),
    StoreRegisters(u8),
    LoadRegisters(u8)
}

pub fn read_game_file(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

fn get_opcode(opcode_bytes: &[u8]) -> Opcode {
    // TODO: Extract this as some struct so that this check does not need to be duplicated in various methods
    if opcode_bytes.len() != 2 {
        panic!("Opcodes must be two bytes.");
    }

    if opcode_bytes == CLEAR_SCREEN_OPCODE {
        return Opcode::ClearScreen;
    }

    if opcode_bytes == RETURN_OPCODE {
        return Opcode::Return;
    }

    let first_nibble = get_upper_nibble(opcode_bytes[0]);
    let last_nibble = get_lower_nibble(opcode_bytes[1]);
    let opcode_selection_info = (first_nibble, last_nibble, opcode_bytes[1]);
    match opcode_selection_info {
        (0x0, _, _) => Opcode::SystemAddr(get_addr(opcode_bytes)),
        (0x1, _, _) => Opcode::JumpAddr(get_addr(opcode_bytes)),
        (0x2, _, _) => Opcode::CallAddr(get_addr(opcode_bytes)),
        (0x3, _, _) => Opcode::SkipRegisterEqualsValue(get_lower_nibble(opcode_bytes[0]), opcode_bytes[1]),
        (0x4, _, _) => Opcode::SkipRegisterNotEqualsValue(get_lower_nibble(opcode_bytes[0]), opcode_bytes[1]),
        (0x5, 0x0, _) => Opcode::SkipRegistersEqual(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x6, _, _) => Opcode::LoadValue(get_lower_nibble(opcode_bytes[0]), opcode_bytes[1]),
        (0x7, _, _) => Opcode::AddValue(get_lower_nibble(opcode_bytes[0]), opcode_bytes[1]),
        (0x8, 0x0, _) => Opcode::LoadRegisterValue(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0x1, _) => Opcode::Or(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0x2, _) => Opcode::And(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0x3, _) => Opcode::Xor(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0x4, _) => Opcode::AddRegisters(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0x5, _) => Opcode::SubtractFromFirstRegister(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0x6, _) => Opcode::BitShiftRight(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0x7, _) => Opcode::SubtractFromSecondRegister(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x8, 0xE, _) => Opcode::BitShiftLeft(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0x9, 0x0, _) => Opcode::SkipRegistersNotEqual(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1])),
        (0xA, _, _) => Opcode::SetRegisterI(get_addr(opcode_bytes)),
        (0xB, _, _) => Opcode::JumpAddrV0(get_addr(opcode_bytes)),
        (0xC, _, _) => Opcode::Random(get_lower_nibble(opcode_bytes[0]), opcode_bytes[1]),
        (0xD, _, _) => Opcode::Draw(get_lower_nibble(opcode_bytes[0]), get_upper_nibble(opcode_bytes[1]), get_lower_nibble(opcode_bytes[1])),
        (0xE, _, 0x9E) => Opcode::SkipKeyPressed(get_lower_nibble(opcode_bytes[0])),
        (0xE, _, 0xA1) => Opcode::SkipKeyNotPressed(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x07) => Opcode::LoadDelayTimer(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x0A) => Opcode::LoadKeyPress(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x15) => Opcode::SetDelayTimer(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x18) => Opcode::SetSoundTimer(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x1E) => Opcode::AddRegisterI(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x29) => Opcode::SetIHexSpriteLocation(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x33) => Opcode::BinaryCodedDecimal(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x55) => Opcode::StoreRegisters(get_lower_nibble(opcode_bytes[0])),
        (0xF, _, 0x65) => Opcode::LoadRegisters(get_lower_nibble(opcode_bytes[0])),
        _ => panic!("Unrecognized opcode: {:0>2X?}{:0>2X?}", opcode_bytes[0], opcode_bytes[1])
    }
}

fn get_upper_nibble(byte: u8) -> u8 {
    (byte & UPPER_NIBBLE_MASK) >> 4
}

fn get_lower_nibble(byte: u8) -> u8 {
    byte & LOWER_NIBBLE_MASK
}

fn get_addr(opcode_bytes: &[u8]) -> u16 {
    if opcode_bytes.len() != 2 {
        panic!("Opcodes must be two bytes.");
    }

    ((get_lower_nibble(opcode_bytes[0]) as u16) << 8) + (opcode_bytes[1] as u16)
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
    fn get_nibble() {
        let byte = 0xAE;
        assert_eq!(get_upper_nibble(byte), 0xA);
        assert_eq!(get_lower_nibble(byte), 0xE);
    }

    #[test]
    fn get_addr_value() {
        let opcode = [0x8A, 0x78];
        assert_eq!(get_addr(&opcode), 0xA78)
    }

    #[test]
    #[should_panic]
    fn get_improper_opcode() {
        let opcode = [0x0];
        get_opcode(&opcode[0..1]);
    }

    #[test]
    #[should_panic(expected = "Unrecognized opcode")]
    fn get_unrecognized_opcode() {
        let opcode = [0x51, 0xC7];
        get_opcode(&opcode);
    }

    #[test]
    fn get_sys_addr_opcode() {
        let opcode = [0x0A, 0x78];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SystemAddr(0xA78));
    }

    #[test]
    fn get_clear_screen_opcode() {
        let opcode = CLEAR_SCREEN_OPCODE;
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::ClearScreen);
    }

    #[test]
    fn get_return_opcode() {
        let opcode = RETURN_OPCODE;
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::Return);
    }

    #[test]
    fn get_jump_addr_opcode() {
        let opcode = [0x1B, 0xEE];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::JumpAddr(0xBEE));
    }

    #[test]
    fn get_call_addr_opcode() {
        let opcode = [0x23, 0x10];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::CallAddr(0x310));
    }

    #[test]
    fn get_skip_register_equals_value_opcode() {
        let opcode = [0x36, 0x5A];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SkipRegisterEqualsValue(0x6, 0x5A));
    }

    #[test]
    fn get_skip_register_not_equals_value_opcode() {
        let opcode = [0x47, 0x1F];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SkipRegisterNotEqualsValue(0x7, 0x1F));
    }

    #[test]
    fn get_skip_registers_equal_opcode() {
        let opcode = [0x51, 0xC0];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SkipRegistersEqual(0x1, 0xC));
    }

    #[test]
    fn get_load_value_opcode() {
        let opcode = [0x64, 0x88];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::LoadValue(0x4, 0x88));
    }

    #[test]
    fn get_add_value_opcode() {
        let opcode = [0x7B, 0xEF];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::AddValue(0xB, 0xEF));
    }

    #[test]
    fn get_load_register_value_value_opcode() {
        let opcode = [0x80, 0x60];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::LoadRegisterValue(0x0, 0x6));
    }

    #[test]
    fn get_or_opcode() {
        let opcode = [0x8D, 0x91];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::Or(0xD, 0x9));
    }

    #[test]
    fn get_and_opcode() {
        let opcode = [0x85, 0x42];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::And(0x5, 0x4));
    }

    #[test]
    fn get_xor_opcode() {
        let opcode = [0x87, 0x23];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::Xor(0x7, 0x2));
    }

    #[test]
    fn get_add_registers_opcode() {
        let opcode = [0x83, 0xA4];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::AddRegisters(0x3, 0xA));
    }

    #[test]
    fn get_subtract_from_first_register_opcode() {
        let opcode = [0x88, 0x95];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SubtractFromFirstRegister(0x8, 0x9));
    }

    #[test]
    fn get_bit_shift_right_opcode() {
        let opcode = [0x85, 0x36];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::BitShiftRight(0x5, 0x3));
    }

    #[test]
    fn get_subtract_from_second_register_opcode() {
        let opcode = [0x81, 0x07];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SubtractFromSecondRegister(0x1, 0x0));
    }

    #[test]
    fn get_bit_shift_left_opcode() {
        let opcode = [0x8E, 0xCE];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::BitShiftLeft(0xE, 0xC));
    }

    #[test]
    fn get_skip_registers_not_equal_opcode() {
        let opcode = [0x97, 0x50];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SkipRegistersNotEqual(0x7, 0x5));
    }

    #[test]
    fn get_set_register_i_opcode() {
        let opcode = [0xAB, 0xF3];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SetRegisterI(0xBF3));
    }

    #[test]
    fn get_jump_addr_v0_opcode() {
        let opcode = [0xB2, 0x09];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::JumpAddrV0(0x209));
    }

    #[test]
    fn get_random_opcode() {
        let opcode = [0xCF, 0x58];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::Random(0xF, 0x58));
    }

    #[test]
    fn get_draw_opcode() {
        let opcode = [0xDA, 0xBC];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::Draw(0xA, 0xB, 0xC));
    }

    #[test]
    fn get_skip_key_pressed_opcode() {
        let opcode = [0xEB, 0x9E];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SkipKeyPressed(0xB));
    }

    #[test]
    fn get_skip_key_not_pressed_opcode() {
        let opcode = [0xED, 0xA1];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SkipKeyNotPressed(0xD));
    }

    #[test]
    fn get_load_delay_timer_opcode() {
        let opcode = [0xFC, 0x07];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::LoadDelayTimer(0xC));
    }

    #[test]
    fn get_load_key_press_opcode() {
        let opcode = [0xF3, 0x0A];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::LoadKeyPress(0x3));
    }

    #[test]
    fn get_set_delay_timer_opcode() {
        let opcode = [0xF8, 0x15];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SetDelayTimer(0x8));
    }

    #[test]
    fn get_set_sound_timer_opcode() {
        let opcode = [0xF6, 0x18];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SetSoundTimer(0x6));
    }

    #[test]
    fn get_add_register_i_opcode() {
        let opcode = [0xF2, 0x1E];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::AddRegisterI(0x2));
    }

    #[test]
    fn get_set_i_hex_sprite_location_opcode() {
        let opcode = [0xFB, 0x29];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::SetIHexSpriteLocation(0xB));
    }

    #[test]
    fn get_binary_coded_decimal_opcode() {
        let opcode = [0xF7, 0x33];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::BinaryCodedDecimal(0x7));
    }

    #[test]
    fn get_store_registers_opcode() {
        let opcode = [0xF3, 0x55];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::StoreRegisters(0x3));
    }

    #[test]
    fn get_load_registers_opcode() {
        let opcode = [0xFA, 0x65];
        let opcode = get_opcode(&opcode);
        assert_eq!(opcode, Opcode::LoadRegisters(0xA));
    }
}
