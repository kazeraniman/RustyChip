//! A module to contain the types and operations related to determining opcodes.  
//! For more information on CHIP-8 opcodes, please see [this section](https://en.wikipedia.org/wiki/CHIP-8#Opcode_table) of the wikipedia page.

use std::fmt::{Display, Formatter};

const CLEAR_SCREEN_OPCODE_FIRST_BYTE: u8 = 0x00;
const CLEAR_SCREEN_OPCODE_SECOND_BYTE: u8 = 0xE0;
const RETURN_OPCODE_OPCODE_FIRST_BYTE: u8 = 0x00;
const RETURN_OPCODE_OPCODE_SECOND_BYTE: u8 = 0xEE;
const LOWER_NIBBLE_MASK: u8 = 0xF;
const UPPER_NIBBLE_MASK: u8 = 0xF0;

/// Denotes a particular opcode and stores the necessary information to process it.
#[derive(PartialEq, Debug)]
pub enum Opcode {
    /// 0nnn
    SystemAddr(u16),

    /// 00E0
    ClearScreen,

    /// 00EE
    Return,

    /// 1nnn
    JumpAddr(u16),

    /// 2nnn
    CallAddr(u16),

    /// 3xkk
    SkipRegisterEqualsValue(usize, u8),

    /// 4xkk
    SkipRegisterNotEqualsValue(usize, u8),

    /// 5xy0
    SkipRegistersEqual(usize, usize),

    /// 6xkk
    LoadValue(usize, u8),

    /// 7xkk
    AddValue(usize, u8),

    /// 8xy0
    LoadRegisterValue(usize, usize),

    /// 8xy1
    Or(usize, usize),

    /// 8xy2
    And(usize, usize),

    /// 8xy3
    Xor(usize, usize),

    /// 8xy4
    AddRegisters(usize, usize),

    /// 8xy5
    SubtractFromFirstRegister(usize, usize),

    /// 8xy6
    BitShiftRight(usize, usize),

    /// 8xy7
    SubtractFromSecondRegister(usize, usize),

    /// 8xyE
    BitShiftLeft(usize, usize),

    /// 9xy0
    SkipRegistersNotEqual(usize, usize),

    /// Annn
    LoadRegisterI(u16),

    /// Bnnn
    JumpAddrV0(u16),

    /// Cxkk
    Random(usize, u8),

    /// Dxyn
    Draw(usize, usize, u8),

    /// Ex9E
    SkipKeyPressed(usize),

    /// ExA1
    SkipKeyNotPressed(usize),

    /// Fx07
    LoadDelayTimer(usize),

    /// Fx0A
    LoadKeyPress(usize),

    /// Fx15
    SetDelayTimer(usize),

    /// Fx18
    SetSoundTimer(usize),

    /// Fx1E
    AddRegisterI(usize),

    /// Fx29
    SetIHexSpriteLocation(usize),

    /// Fx33
    BinaryCodedDecimal(usize),

    /// Fx55
    StoreRegisters(usize),

    /// Fx65
    LoadRegisters(usize)
}

/// Stores the information necessary to determine an [Opcode](Opcode) from a pair of bytes read from memory. 
pub struct OpcodeBytes {
    first_byte: u8,
    second_byte: u8,
    first_nibble: u8,
    last_nibble: u8
}

impl OpcodeBytes {
    /// Returns an opcode-convertible structure from a slice of `u8`.
    /// The resulting structure can be used to get a proper [Opcode](Opcode).
    ///
    /// # Arguments
    ///
    /// * `opcode_bytes` - A `u8` slice that holds the bytes necessary to form an opcode.
    ///
    /// # Panics
    ///
    /// Will panic if `opcode_bytes` does not have exactly 2 elements as 2 bytes make up a valid opcode.
    #[must_use]
    pub fn build(opcode_bytes: &[u8]) -> OpcodeBytes {
        assert_eq!(opcode_bytes.len(), 2, "Improper opcode format: Opcodes must be two bytes.");

        OpcodeBytes {
            first_byte: opcode_bytes[0],
            second_byte: opcode_bytes[1],
            first_nibble: Self::get_upper_nibble_u8(opcode_bytes[0]),
            last_nibble: Self::get_lower_nibble_u8(opcode_bytes[1])
        }
    }

    /// Returns a `u8` containing the first 4 bits of a byte.
    ///
    /// # Parameters
    ///
    /// * `byte` - The byte from which to read the nibble.
    fn get_upper_nibble_u8(byte: u8) -> u8 {
        (byte & UPPER_NIBBLE_MASK) >> 4
    }

    /// Returns a `u8` containing the last 4 bits of a byte.
    ///
    /// # Parameters
    ///
    /// * `byte` - The byte from which to read the nibble.
    fn get_lower_nibble_u8(byte: u8) -> u8 {
        byte & LOWER_NIBBLE_MASK
    }

    /// Returns a `usize` containing the first 4 bits of a byte.
    ///
    /// # Parameters
    ///
    /// * `byte` - The byte from which to read the nibble.
    fn get_upper_nibble(byte: u8) -> usize {
        Self::get_upper_nibble_u8(byte) as usize
    }

    /// Returns a `usize` containing the last 4 bits of a byte.
    ///
    /// # Parameters
    ///
    /// * `byte` - The byte from which to read the nibble.
    fn get_lower_nibble(byte: u8) -> usize {
        Self::get_lower_nibble_u8(byte) as usize
    }

    /// Returns the 12-bit address contained in an opcode as a `u16`.
    #[allow(clippy::cast_possible_truncation)]
    fn get_addr(&self) -> u16 {
        ((Self::get_lower_nibble(self.first_byte) as u16) << 8) + (u16::from(self.second_byte))
    }

    /// Returns a proper [Opcode](Opcode) with the data needed to handle it.
    ///
    /// # Panics
    ///
    /// Will panic if the opcode is not recognized as a valid one. See [wikipedia](https://en.wikipedia.org/wiki/CHIP-8#Opcode_table) for a list of valid opcodes.
    #[must_use]
    pub fn get_opcode(&self) -> Opcode {
        let opcode_selection_info = (self.first_nibble, self.last_nibble, self.first_byte, self.second_byte);
        match opcode_selection_info {
            (_, _, CLEAR_SCREEN_OPCODE_FIRST_BYTE, CLEAR_SCREEN_OPCODE_SECOND_BYTE) => Opcode::ClearScreen,
            (_, _, RETURN_OPCODE_OPCODE_FIRST_BYTE, RETURN_OPCODE_OPCODE_SECOND_BYTE) => Opcode::Return,
            (0x0, _, _, _) => Opcode::SystemAddr(self.get_addr()),
            (0x1, _, _, _) => Opcode::JumpAddr(self.get_addr()),
            (0x2, _, _, _) => Opcode::CallAddr(self.get_addr()),
            (0x3, _, _, _) => Opcode::SkipRegisterEqualsValue(OpcodeBytes::get_lower_nibble(self.first_byte), self.second_byte),
            (0x4, _, _, _) => Opcode::SkipRegisterNotEqualsValue(OpcodeBytes::get_lower_nibble(self.first_byte), self.second_byte),
            (0x5, 0x0, _, _) => Opcode::SkipRegistersEqual(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x6, _, _, _) => Opcode::LoadValue(OpcodeBytes::get_lower_nibble(self.first_byte), self.second_byte),
            (0x7, _, _, _) => Opcode::AddValue(OpcodeBytes::get_lower_nibble(self.first_byte), self.second_byte),
            (0x8, 0x0, _, _) => Opcode::LoadRegisterValue(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0x1, _, _) => Opcode::Or(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0x2, _, _) => Opcode::And(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0x3, _, _) => Opcode::Xor(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0x4, _, _) => Opcode::AddRegisters(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0x5, _, _) => Opcode::SubtractFromFirstRegister(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0x6, _, _) => Opcode::BitShiftRight(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0x7, _, _) => Opcode::SubtractFromSecondRegister(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x8, 0xE, _, _) => Opcode::BitShiftLeft(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0x9, 0x0, _, _) => Opcode::SkipRegistersNotEqual(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte)),
            (0xA, _, _, _) => Opcode::LoadRegisterI(self.get_addr()),
            (0xB, _, _, _) => Opcode::JumpAddrV0(self.get_addr()),
            (0xC, _, _, _) => Opcode::Random(OpcodeBytes::get_lower_nibble(self.first_byte), self.second_byte),
            (0xD, _, _, _) => Opcode::Draw(OpcodeBytes::get_lower_nibble(self.first_byte), OpcodeBytes::get_upper_nibble(self.second_byte), OpcodeBytes::get_lower_nibble_u8(self.second_byte)),
            (0xE, _, _, 0x9E) => Opcode::SkipKeyPressed(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xE, _, _, 0xA1) => Opcode::SkipKeyNotPressed(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x07) => Opcode::LoadDelayTimer(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x0A) => Opcode::LoadKeyPress(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x15) => Opcode::SetDelayTimer(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x18) => Opcode::SetSoundTimer(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x1E) => Opcode::AddRegisterI(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x29) => Opcode::SetIHexSpriteLocation(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x33) => Opcode::BinaryCodedDecimal(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x55) => Opcode::StoreRegisters(OpcodeBytes::get_lower_nibble(self.first_byte)),
            (0xF, _, _, 0x65) => Opcode::LoadRegisters(OpcodeBytes::get_lower_nibble(self.first_byte)),
            _ => panic!("Unrecognized opcode: {self}")
        }
    }
}

impl Display for OpcodeBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>2X?}{:0>2X?}", self.first_byte, self.second_byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xAB, 0xCD]);
        assert_eq!(opcode_bytes.first_byte, 0xAB);
        assert_eq!(opcode_bytes.second_byte, 0xCD);
        assert_eq!(opcode_bytes.first_nibble, 0xA);
        assert_eq!(opcode_bytes.last_nibble, 0xD);
    }

    #[test]
    #[should_panic(expected = "Improper opcode format")]
    fn build_improper_opcode_short() {
        let _ = OpcodeBytes::build(&[0x68]);
    }

    #[test]
    #[should_panic(expected = "Improper opcode format")]
    fn build_improper_opcode_long() {
        let _ = OpcodeBytes::build(&[0x23, 0x81, 0x54]);
    }

    #[test]
    fn get_nibble() {
        let byte = 0xAE;
        assert_eq!(OpcodeBytes::get_upper_nibble(byte), 0xA);
        assert_eq!(OpcodeBytes::get_lower_nibble(byte), 0xE);
    }

    #[test]
    fn get_addr_value() {
        let opcode_bytes = OpcodeBytes::build(&[0x8A, 0x78]);
        assert_eq!(opcode_bytes.get_addr(), 0xA78);
    }

    #[test]
    fn to_string() {
        let opcode_bytes = OpcodeBytes::build(&[0x36, 0x91]);
        assert_eq!(opcode_bytes.to_string(), String::from("3691"));
    }

    #[test]
    #[should_panic(expected = "Unrecognized opcode")]
    fn get_unrecognized_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x51, 0xC7]);
        let _ = opcode_bytes.get_opcode();
    }

    #[test]
    fn get_sys_addr_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x0A, 0x78]);
        let opcode = opcode_bytes.get_opcode();
        assert_eq!(opcode, Opcode::SystemAddr(0xA78));
    }

    #[test]
    fn get_clear_screen_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[CLEAR_SCREEN_OPCODE_FIRST_BYTE, CLEAR_SCREEN_OPCODE_SECOND_BYTE]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::ClearScreen);
    }

    #[test]
    fn get_return_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[RETURN_OPCODE_OPCODE_FIRST_BYTE, RETURN_OPCODE_OPCODE_SECOND_BYTE]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::Return);
    }

    #[test]
    fn get_jump_addr_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x1B, 0xEE]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::JumpAddr(0xBEE));
    }

    #[test]
    fn get_call_addr_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x23, 0x10]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::CallAddr(0x310));
    }

    #[test]
    fn get_skip_register_equals_value_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x36, 0x5A]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SkipRegisterEqualsValue(0x6, 0x5A));
    }

    #[test]
    fn get_skip_register_not_equals_value_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x47, 0x1F]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SkipRegisterNotEqualsValue(0x7, 0x1F));
    }

    #[test]
    fn get_skip_registers_equal_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x51, 0xC0]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SkipRegistersEqual(0x1, 0xC));
    }

    #[test]
    fn get_load_value_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x64, 0x88]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::LoadValue(0x4, 0x88));
    }

    #[test]
    fn get_add_value_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x7B, 0xEF]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::AddValue(0xB, 0xEF));
    }

    #[test]
    fn get_load_register_value_value_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x80, 0x60]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::LoadRegisterValue(0x0, 0x6));
    }

    #[test]
    fn get_or_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x8D, 0x91]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::Or(0xD, 0x9));
    }

    #[test]
    fn get_and_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x85, 0x42]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::And(0x5, 0x4));
    }

    #[test]
    fn get_xor_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x87, 0x23]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::Xor(0x7, 0x2));
    }

    #[test]
    fn get_add_registers_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x83, 0xA4]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::AddRegisters(0x3, 0xA));
    }

    #[test]
    fn get_subtract_from_first_register_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x88, 0x95]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SubtractFromFirstRegister(0x8, 0x9));
    }

    #[test]
    fn get_bit_shift_right_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x85, 0x36]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::BitShiftRight(0x5, 0x3));
    }

    #[test]
    fn get_subtract_from_second_register_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x81, 0x07]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SubtractFromSecondRegister(0x1, 0x0));
    }

    #[test]
    fn get_bit_shift_left_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x8E, 0xCE]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::BitShiftLeft(0xE, 0xC));
    }

    #[test]
    fn get_skip_registers_not_equal_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0x97, 0x50]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SkipRegistersNotEqual(0x7, 0x5));
    }

    #[test]
    fn get_set_register_i_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xAB, 0xF3]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::LoadRegisterI(0xBF3));
    }

    #[test]
    fn get_jump_addr_v0_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xB2, 0x09]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::JumpAddrV0(0x209));
    }

    #[test]
    fn get_random_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xCF, 0x58]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::Random(0xF, 0x58));
    }

    #[test]
    fn get_draw_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xDA, 0xBC]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::Draw(0xA, 0xB, 0xC));
    }

    #[test]
    fn get_skip_key_pressed_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xEB, 0x9E]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SkipKeyPressed(0xB));
    }

    #[test]
    fn get_skip_key_not_pressed_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xED, 0xA1]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SkipKeyNotPressed(0xD));
    }

    #[test]
    fn get_load_delay_timer_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xFC, 0x07]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::LoadDelayTimer(0xC));
    }

    #[test]
    fn get_load_key_press_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xF3, 0x0A]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::LoadKeyPress(0x3));
    }

    #[test]
    fn get_set_delay_timer_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xF8, 0x15]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SetDelayTimer(0x8));
    }

    #[test]
    fn get_set_sound_timer_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xF6, 0x18]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SetSoundTimer(0x6));
    }

    #[test]
    fn get_add_register_i_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xF2, 0x1E]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::AddRegisterI(0x2));
    }

    #[test]
    fn get_set_i_hex_sprite_location_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xFB, 0x29]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::SetIHexSpriteLocation(0xB));
    }

    #[test]
    fn get_binary_coded_decimal_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xF7, 0x33]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::BinaryCodedDecimal(0x7));
    }

    #[test]
    fn get_store_registers_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xF3, 0x55]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::StoreRegisters(0x3));
    }

    #[test]
    fn get_load_registers_opcode() {
        let opcode_bytes = OpcodeBytes::build(&[0xFA, 0x65]);
        assert_eq!(opcode_bytes.get_opcode(), Opcode::LoadRegisters(0xA));
    }
}
