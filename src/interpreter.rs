use std::collections::{HashSet};
use rand::random;
use sdl2::audio::AudioDevice;
use sdl2::keyboard::Keycode;
use crate::audio::SquareWave;
use crate::opcodes::{Opcode, OpcodeBytes};

const PROGRAM_START_ADDRESS: u16 = 0x200;
const PROGRAM_COUNTER_INCREMENT: u16 = 0x2;
const BYTE_MASK: u16 = u8::MAX as u16;
const LEAST_SIGNIFICANT_BIT_MASK: u8 = 0x1;
const MOST_SIGNIFICANT_BIT_MASK: u8 = 0x80;
const HEXADECIMAL_DIGIT_SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];

pub struct Interpreter<'a> {
    ram: [u8; 4096],
    registers: [u8; 16],
    register_i: u16,
    register_f: bool,
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    stack_pointer: usize,
    stack: [u16; 16],
    keyboard: HashSet<u8>,
    should_wait_for_key: bool,
    wait_for_key_register: usize,
    audio_device: Option<&'a AudioDevice<SquareWave>>
}

impl<'a> Interpreter<'a> {
    pub fn new_with_audio(audio_device: Option<&'a AudioDevice<SquareWave>>) -> Interpreter {
        let mut ram = [0; 4096];
        for i in 0..HEXADECIMAL_DIGIT_SPRITES.len() {
            ram[i] = HEXADECIMAL_DIGIT_SPRITES[i];
        }

        Interpreter {
            ram,
            registers: [0; 16],
            register_i: 0,
            register_f: false,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0,
            stack_pointer: 0,
            stack: [0; 16],
            keyboard: HashSet::new(),
            should_wait_for_key: false,
            wait_for_key_register: 0,
            audio_device
        }
    }

    #[cfg(test)]
    fn new() -> Interpreter<'a> {
        Self::new_with_audio(None)
    }

    pub fn load_game(&mut self, game_data: Vec<u8>) {
        for (i, byte) in game_data.iter().enumerate() {
            self.ram[PROGRAM_START_ADDRESS as usize + i] = *byte;
        }

        self.program_counter = PROGRAM_START_ADDRESS;
    }

    fn get_key_mapping(keycode: Keycode) -> Option<u8> {
        let key = match keycode {
            Keycode::Num1 => 0x1,
            Keycode::Num2 => 0x2,
            Keycode::Num3 => 0x3,
            Keycode::Num4 => 0xC,
            Keycode::Q => 0x4,
            Keycode::W => 0x5,
            Keycode::E => 0x6,
            Keycode::R => 0xD,
            Keycode::A => 0x7,
            Keycode::S => 0x8,
            Keycode::D => 0x9,
            Keycode::F => 0xE,
            Keycode::Z => 0xA,
            Keycode::X => 0x0,
            Keycode::C => 0xB,
            Keycode::V => 0xF,
            _ => return None
        };

        Some(key)
    }

    pub fn handle_key_press(&mut self, keycode: Keycode) {
        if let Some(key) = Self::get_key_mapping(keycode) {
            if self.should_wait_for_key {
                self.registers[self.wait_for_key_register] = key;
                self.should_wait_for_key = false;
            }

            self.keyboard.insert(key);
        }
    }

    pub fn handle_key_release(&mut self, keycode: Keycode) {
        if let Some(key) = Self::get_key_mapping(keycode) {
            self.keyboard.remove(&key);
        }
    }

    pub fn handle_cycle(&mut self) {
        if self.should_wait_for_key {
            return;
        }

        let opcode = OpcodeBytes::build(&self.ram[self.program_counter as usize..=(self.program_counter + 1) as usize]);
        let opcode = opcode.get_opcode();
        self.program_counter += PROGRAM_COUNTER_INCREMENT;
        self.handle_opcode(opcode);
        self.handle_timers();
    }

    fn handle_timers(&mut self) {
        let old_sound_timer = self.sound_timer;
        self.sound_timer = self.sound_timer.saturating_sub(1);
        self.delay_timer = self.delay_timer.saturating_sub(1);

        if old_sound_timer != 0 && self.sound_timer == 0 {
            self.set_audio_status();
        }
    }

    fn set_audio_status(&self) {
        if let Some(audio_device) = self.audio_device {
            if self.sound_timer > 0 { audio_device.resume() } else { audio_device.pause() };
        }
    }

    fn handle_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::ClearScreen => {}
            Opcode::Return => self.return_from_subroutine(),
            Opcode::JumpAddr(address) => self.jump_addr(address),
            Opcode::SystemAddr(address) | Opcode::CallAddr(address) => self.call_addr(address),
            Opcode::SkipRegisterEqualsValue(register, value) => self.skip_register_equals_value(register, value),
            Opcode::SkipRegisterNotEqualsValue(register, value) => self.skip_register_not_equals_value(register, value),
            Opcode::SkipRegistersEqual(first_register, second_register) => self.skip_registers_equal(first_register, second_register),
            Opcode::LoadValue(register, value) => self.load_value(register, value),
            Opcode::AddValue(register, value) => self.add_value(register, value),
            Opcode::LoadRegisterValue(first_register, second_register) => self.load_register_value(first_register, second_register),
            Opcode::Or(first_register, second_register) => self.or(first_register, second_register),
            Opcode::And(first_register, second_register) => self.and(first_register, second_register),
            Opcode::Xor(first_register, second_register) => self.xor(first_register, second_register),
            Opcode::AddRegisters(first_register, second_register) => self.add_registers(first_register, second_register),
            Opcode::SubtractFromFirstRegister(first_register, second_register) => self.bounded_subtraction(first_register, second_register, first_register),
            Opcode::BitShiftRight(register) => self.bit_shift_right(register),
            Opcode::SubtractFromSecondRegister(first_register, second_register) => self.bounded_subtraction(second_register, first_register, first_register),
            Opcode::BitShiftLeft(register) => self.bit_shift_left(register),
            Opcode::SkipRegistersNotEqual(first_register, second_register) => self.skip_registers_not_equal(first_register, second_register),
            Opcode::LoadRegisterI(address) => self.load_register_i(address),
            Opcode::JumpAddrV0(address) => self.jump_address_v0(address),
            Opcode::Random(register, value) => self.random(register, value),
            Opcode::Draw(_, _, _) => {}
            Opcode::SkipKeyPressed(register) => self.skip_key_pressed(register),
            Opcode::SkipKeyNotPressed(register) => self.skip_key_not_pressed(register),
            Opcode::LoadDelayTimer(register) => self.load_delay_timer(register),
            Opcode::LoadKeyPress(register) => self.load_key_press(register),
            Opcode::SetDelayTimer(register) => self.set_delay_timer(register),
            Opcode::SetSoundTimer(register) => self.set_sound_timer(register),
            Opcode::AddRegisterI(register) => self.add_register_i(register),
            Opcode::SetIHexSpriteLocation(register) => self.set_register_i_hex_sprite_location(register),
            Opcode::BinaryCodedDecimal(register) => self.binary_coded_decimal(register),
            Opcode::StoreRegisters(register) => self.store_registers(register),
            Opcode::LoadRegisters(register) => self.load_registers(register)
        }
    }

    fn jump_addr(&mut self, address: u16) {
        self.program_counter = address;
    }

    fn skip_register_equals_value(&mut self, register: usize, value: u8) {
        if self.registers[register] == value {
            self.program_counter += PROGRAM_COUNTER_INCREMENT;
        }
    }

    fn skip_register_not_equals_value(&mut self, register: usize, value: u8) {
        if self.registers[register] != value {
            self.program_counter += PROGRAM_COUNTER_INCREMENT;
        }
    }

    fn skip_registers_equal(&mut self, first_register: usize, second_register: usize) {
        if self.registers[first_register] == self.registers[second_register] {
            self.program_counter += PROGRAM_COUNTER_INCREMENT;
        }
    }

    fn skip_registers_not_equal(&mut self, first_register: usize, second_register: usize) {
        if self.registers[first_register] != self.registers[second_register] {
            self.program_counter += PROGRAM_COUNTER_INCREMENT;
        }
    }

    fn load_value(&mut self, register: usize, value: u8) {
        self.registers[register] = value;
    }

    fn add_value(&mut self, register: usize, value: u8) {
        self.registers[register] = ((self.registers[register] as u16 + value as u16) & BYTE_MASK) as u8;
    }

    fn load_register_value(&mut self, first_register: usize, second_register: usize) {
        self.registers[first_register] = self.registers[second_register];
    }

    fn or(&mut self, first_register: usize, second_register: usize) {
        self.registers[first_register] |= self.registers[second_register];
    }

    fn and(&mut self, first_register: usize, second_register: usize) {
        self.registers[first_register] &= self.registers[second_register];
    }

    fn xor(&mut self, first_register: usize, second_register: usize) {
        self.registers[first_register] ^= self.registers[second_register];
    }

    fn random(&mut self, register: usize, value: u8) {
        let random_byte: u8 = random();
        self.registers[register] = random_byte & value;
    }

    fn store_registers(&mut self, register: usize) {
        let start_address = self.register_i as usize;
        for i in 0..=register {
            self.ram[start_address + i] = self.registers[i];
        }
    }

    fn load_registers(&mut self, register: usize) {
        let start_address = self.register_i as usize;
        for i in 0..=register {
            self.registers[i] = self.ram[start_address + i];
        }
    }

    fn load_register_i(&mut self, address: u16) {
        self.register_i = address;
    }

    fn jump_address_v0(&mut self, address: u16) {
        self.jump_addr(address + self.registers[0] as u16);
    }

    fn load_delay_timer(&mut self, register: usize) {
        self.registers[register] = self.delay_timer;
    }

    fn set_delay_timer(&mut self, register: usize) {
        self.delay_timer = self.registers[register];
    }

    fn set_sound_timer(&mut self, register: usize) {
        self.sound_timer = self.registers[register];
        self.set_audio_status();
    }

    fn add_register_i(&mut self, register: usize) {
        self.register_i += self.registers[register] as u16;
    }

    fn add_registers(&mut self, first_register: usize, second_register: usize) {
        let sum: u16 = self.registers[first_register] as u16 + self.registers[second_register] as u16;
        let max_u8 = BYTE_MASK;
        self.register_f = sum > max_u8;
        self.registers[first_register] = (sum & max_u8) as u8;
    }

    fn bounded_subtraction(&mut self, minuend_register: usize, subtrahend_register: usize, result_register: usize) {
        let (difference, did_underflow) = self.registers[minuend_register].overflowing_sub(self.registers[subtrahend_register]);
        self.register_f = did_underflow;
        self.registers[result_register] = difference;
    }

    fn bit_shift_right(&mut self, register: usize) {
        self.register_f = (self.registers[register] & LEAST_SIGNIFICANT_BIT_MASK) == 0x1;
        self.registers[register] >>= 0x1;
    }

    fn bit_shift_left(&mut self, register: usize) {
        self.register_f = ((self.registers[register] & MOST_SIGNIFICANT_BIT_MASK) >> 7) == 0x1;
        self.registers[register] <<= 0x1;
    }

    fn binary_coded_decimal(&mut self, register: usize) {
        let mut value = self.registers[register];

        for i in (0..=2).rev() {
            self.ram[(self.register_i + i) as usize] = value % 10;
            value /= 10;
        }
    }

    fn call_addr(&mut self, address: u16) {
        self.stack[self.stack_pointer] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = address;
    }

    fn return_from_subroutine(&mut self) {
        self.program_counter = self.stack[self.stack_pointer - 1];
        self.stack_pointer -= 1;
    }

    fn set_register_i_hex_sprite_location(&mut self, register: usize) {
        self.register_i = (self.registers[register] as u16) * 0x5;
    }

    fn skip_key_pressed(&mut self, register: usize) {
        if self.keyboard.contains(&self.registers[register]) {
            self.program_counter += PROGRAM_COUNTER_INCREMENT;
        }
    }

    fn skip_key_not_pressed(&mut self, register: usize) {
        if !self.keyboard.contains(&self.registers[register]) {
            self.program_counter += PROGRAM_COUNTER_INCREMENT;
        }
    }

    fn load_key_press(&mut self, register: usize) {
        self.should_wait_for_key = true;
        self.wait_for_key_register = register;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_interpreter() {
        let interpreter = Interpreter::new();
        assert_eq!(interpreter.register_i, 0, "Register I initialized incorrectly.");
        assert!(!interpreter.register_f, "Register F initialized incorrectly.");
        assert_eq!(interpreter.delay_timer, 0, "Delay timer initialized incorrectly.");
        assert_eq!(interpreter.sound_timer, 0, "Sound timer initialized incorrectly.");
        assert_eq!(interpreter.program_counter, 0, "Program counter initialized incorrectly.");
        assert_eq!(interpreter.stack_pointer, 0, "Stack pointer initialized incorrectly.");
        assert_eq!(interpreter.keyboard.len(), 0, "Keyboard initialized incorrectly.");
        assert!(!interpreter.should_wait_for_key, "Should wait for key initialized incorrectly.");
        assert_eq!(interpreter.wait_for_key_register, 0, "Wait for key register initialized incorrectly.");

        let hex_digit_sprite_length = HEXADECIMAL_DIGIT_SPRITES.len();
        for (i, byte) in interpreter.ram.iter().enumerate() {
            assert_eq!(byte, if i < hex_digit_sprite_length { &HEXADECIMAL_DIGIT_SPRITES[i] } else { &0 }, "RAM initialized incorrectly.");
        }

        for byte in interpreter.registers.iter() {
            assert_eq!(byte, &0, "Registers initialized incorrectly.");
        }

        for address in interpreter.stack.iter() {
            assert_eq!(address, &0, "Stack initialized incorrectly.");
        }
    }

    #[test]
    pub fn load_game() {
        let mut interpreter = Interpreter::new();

        let fake_game_data = vec![0x23, 0x78, 0x93];
        interpreter.load_game(fake_game_data.clone());
        for i in 0..fake_game_data.len() {
            assert_eq!(interpreter.ram[PROGRAM_START_ADDRESS as usize + i], fake_game_data[i], "Loaded game data does not match the original game data.");
        }
    }

    #[test]
    pub fn handle_cycle() {
        let mut interpreter = Interpreter::new();

        let program_start_usize = PROGRAM_START_ADDRESS as usize;
        interpreter.ram[program_start_usize] = 0xAA;
        interpreter.ram[program_start_usize + 1] = 0xAA;
        interpreter.ram[program_start_usize + 2] = 0x1B;
        interpreter.ram[program_start_usize + 3] = 0xBB;
        interpreter.delay_timer = 0x1;
        interpreter.sound_timer = 0x1;
        interpreter.program_counter = PROGRAM_START_ADDRESS;
        interpreter.handle_cycle();
        assert_eq!(interpreter.register_i, 0xAAA, "Opcode not handled.");
        assert_eq!(interpreter.program_counter, PROGRAM_START_ADDRESS + PROGRAM_COUNTER_INCREMENT, "Program counter not incremented.");
        assert_eq!(interpreter.delay_timer, 0x0, "Delay timer not decremented.");
        assert_eq!(interpreter.sound_timer, 0x0, "Sound timer not decremented.");

        interpreter.handle_cycle();
        assert_eq!(interpreter.program_counter, 0xBBB, "Program counter incremented after jump.");
    }

    #[test]
    fn handle_timers() {
        let mut interpreter = Interpreter::new();

        interpreter.delay_timer = 0x3;
        interpreter.sound_timer = 0x2;
        interpreter.handle_timers();
        assert_eq!(interpreter.delay_timer, 0x2, "Delay timer not decremented.");
        assert_eq!(interpreter.sound_timer, 0x1, "Sound timer not decremented.");

        interpreter.handle_timers();
        assert_eq!(interpreter.delay_timer, 0x1, "Delay timer not decremented.");
        assert_eq!(interpreter.sound_timer, 0x0, "Sound timer not decremented.");

        interpreter.handle_timers();
        assert_eq!(interpreter.delay_timer, 0x0, "Delay timer not decremented.");
        assert_eq!(interpreter.sound_timer, 0x0, "Sound timer not saturated at 0.");

        interpreter.handle_timers();
        assert_eq!(interpreter.delay_timer, 0x0, "Delay timer not saturated at 0.");
        assert_eq!(interpreter.sound_timer, 0x0, "Sound timer not saturated at 0.");
    }

    #[test]
    fn get_key_mapping() {
        assert_eq!(Interpreter::get_key_mapping(Keycode::Num1), Some(0x1), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::Num2), Some(0x2), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::Num3), Some(0x3), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::Num4), Some(0xC), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::Q), Some(0x4), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::W), Some(0x5), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::E), Some(0x6), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::R), Some(0xD), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::A), Some(0x7), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::S), Some(0x8), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::D), Some(0x9), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::F), Some(0xE), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::Z), Some(0xA), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::X), Some(0x0), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::C), Some(0xB), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::V), Some(0xF), "Incorrect key mapping.");
        assert_eq!(Interpreter::get_key_mapping(Keycode::G), None, "Ignored key is mapped.");
    }

    #[test]
    fn handle_key_press() {
        let mut interpreter = Interpreter::new();

        let q_key_mapping = &Interpreter::get_key_mapping(Keycode::Q).unwrap();
        let f_key_mapping = &Interpreter::get_key_mapping(Keycode::F).unwrap();
        interpreter.handle_key_press(Keycode::Q);
        assert!(interpreter.keyboard.contains(q_key_mapping), "Key press not stored.");
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of keypresses stored.");

        // Testing that repeated press doesn't break anything
        interpreter.handle_key_press(Keycode::Q);
        assert!(interpreter.keyboard.contains(q_key_mapping), "Key press not stored.");
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of keypresses stored.");

        interpreter.handle_key_press(Keycode::F);
        assert!(interpreter.keyboard.contains(f_key_mapping), "Key press not stored.");
        assert!(interpreter.keyboard.contains(q_key_mapping), "Stored key press removed.");
        assert_eq!(interpreter.keyboard.len(), 2, "Wrong number of keypresses stored.");
    }

    #[test]
    fn handle_key_release() {
        let mut interpreter = Interpreter::new();

        let q_key_mapping = &Interpreter::get_key_mapping(Keycode::Q).unwrap();
        let f_key_mapping = &Interpreter::get_key_mapping(Keycode::F).unwrap();
        interpreter.keyboard.insert(*q_key_mapping);
        interpreter.keyboard.insert(*f_key_mapping);
        interpreter.handle_key_release(Keycode::L);
        assert!(interpreter.keyboard.contains(q_key_mapping), "Stored key press removed.");
        assert!(interpreter.keyboard.contains(f_key_mapping), "Stored key press removed.");
        assert_eq!(interpreter.keyboard.len(), 2, "Wrong number of keypresses stored.");

        interpreter.handle_key_release(Keycode::Q);
        assert!(!interpreter.keyboard.contains(q_key_mapping), "Key press stored.");
        assert!(interpreter.keyboard.contains(f_key_mapping), "Key press not stored.");
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of keypresses stored.");

        // Testing that repeated release doesn't break anything
        interpreter.handle_key_release(Keycode::Q);
        assert!(!interpreter.keyboard.contains(q_key_mapping), "Key press stored.");
        assert!(interpreter.keyboard.contains(f_key_mapping), "Key press not stored.");
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of keypresses stored.");

        interpreter.handle_key_release(Keycode::F);
        assert!(!interpreter.keyboard.contains(f_key_mapping), "Key press stored.");
        assert!(!interpreter.keyboard.contains(q_key_mapping), "Key press stored.");
        assert_eq!(interpreter.keyboard.len(), 0, "Wrong number of keypresses stored.");
    }

    #[cfg(test)]
    mod opcode_tests {
        use super::*;

        #[test]
        fn handle_jump_addr_opcode() {
            let mut interpreter = Interpreter::new();

            let address = 0x381;
            interpreter.handle_opcode(Opcode::JumpAddr(address));
            assert_eq!(interpreter.program_counter, address, "Program counter not updated.");
        }

        #[test]
        fn handle_skip_register_equals_value_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0x5;
            let value = 0xA2;
            interpreter.handle_opcode(Opcode::SkipRegisterEqualsValue(register, value));
            assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when register value doesn't match.");
            assert_eq!(interpreter.registers[register], 0x0, "Register value modified.");

            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::SkipRegisterEqualsValue(register, value));
            assert_eq!(interpreter.program_counter, PROGRAM_COUNTER_INCREMENT, "Program counter not updated when register value matches.");
            assert_eq!(interpreter.registers[register], value, "Register value modified.");
        }

        #[test]
        fn handle_skip_register_not_equals_value_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0xA;
            let first_value = 0x23;
            interpreter.registers[register] = first_value;
            interpreter.handle_opcode(Opcode::SkipRegisterNotEqualsValue(register, first_value));
            assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when register value matches.");
            assert_eq!(interpreter.registers[register], first_value, "Register value modified.");

            let second_value = 0x24;
            interpreter.registers[register] = second_value;
            interpreter.handle_opcode(Opcode::SkipRegisterNotEqualsValue(register, first_value));
            assert_eq!(interpreter.program_counter, PROGRAM_COUNTER_INCREMENT, "Program counter not updated when register value doesn't match.");
            assert_eq!(interpreter.registers[register], second_value, "Register value modified.");
        }

        #[test]
        fn handle_skip_registers_equal_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x1;
            let second_register = 0x2;
            let first_value = 0x4;
            let second_value = 0x5;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::SkipRegistersEqual(first_register, second_register));
            assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when registers don't match.");
            assert_eq!(interpreter.registers[first_register], first_value, "First register value modified.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");

            interpreter.registers[first_register] = second_value;
            interpreter.handle_opcode(Opcode::SkipRegistersEqual(first_register, second_register));
            assert_eq!(interpreter.program_counter, PROGRAM_COUNTER_INCREMENT, "Program counter not updated when registers match.");
            assert_eq!(interpreter.registers[first_register], second_value, "First register value modified.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");
        }

        #[test]
        fn handle_skip_registers_not_equal_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x1;
            let second_register = 0x2;
            let first_value = 0x4;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = first_value;
            interpreter.handle_opcode(Opcode::SkipRegistersNotEqual(first_register, second_register));
            assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when registers match.");
            assert_eq!(interpreter.registers[first_register], first_value, "First register value modified.");
            assert_eq!(interpreter.registers[second_register], first_value, "Second register value modified.");

            let second_value = 0x5;
            interpreter.registers[first_register] = second_value;
            interpreter.handle_opcode(Opcode::SkipRegistersNotEqual(first_register, second_register));
            assert_eq!(interpreter.program_counter, PROGRAM_COUNTER_INCREMENT, "Program counter not updated when registers don't match.");
            assert_eq!(interpreter.registers[first_register], second_value, "First register value modified.");
            assert_eq!(interpreter.registers[second_register], first_value, "Second register value modified.");
        }

        #[test]
        fn handle_load_value_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0x0;
            let value = 0x58;
            interpreter.handle_opcode(Opcode::LoadValue(register, value));
            assert_eq!(interpreter.registers[register], value, "Value not loaded into register.");
        }

        #[test]
        fn handle_add_value_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0x6;
            let value = 0xAB;
            let first_added_value = 0x11;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::AddValue(register, first_added_value));
            assert_eq!(interpreter.registers[register], value + first_added_value, "Regular addition failed.");

            let second_added_value = 0xAA;
            interpreter.handle_opcode(Opcode::AddValue(register, second_added_value));
            let sum_with_truncation = 0x66;
            assert_eq!(interpreter.registers[register], sum_with_truncation, "Overflow not handled. Should truncate.");
        }

        #[test]
        fn handle_load_register_value_opcode() {
            let mut interpreter = Interpreter::new();

            let original_value = 0xDD;
            let first_register = 0x0;
            let second_register = 0x1;
            interpreter.registers[second_register] = original_value;
            interpreter.handle_opcode(Opcode::LoadRegisterValue(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], original_value, "Value not loaded into register.");
            assert_eq!(interpreter.registers[second_register], original_value, "Original register value modified.");
        }

        #[test]
        fn handle_or_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x2;
            let second_register = 0x4;
            let first_value = 0xCC;
            let second_value = 0xC3;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::Or(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], first_value | second_value, "Bitwise OR not applied correctly.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");
        }

        #[test]
        fn handle_and_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x6;
            let second_register = 0x3;
            let first_value = 0xAA;
            let second_value = 0xCC;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::And(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], first_value & second_value, "Bitwise AND not applied correctly.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");
        }

        #[test]
        fn handle_xor_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0xB;
            let second_register = 0xF;
            let first_value = 0x33;
            let second_value = 0x55;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::Xor(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], first_value ^ second_value, "Bitwise XOR not applied correctly.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");
        }

        #[test]
        fn handle_random_opcode() {
            let mut interpreter = Interpreter::new();

            // Since the result is random, we basically just check to make sure that it doesn't panic
            interpreter.handle_opcode(Opcode::Random(0x9, 0x53));
        }

        #[test]
        fn handle_store_registers_opcode() {
            let mut interpreter = Interpreter::new();

            let register_values = &[0x32, 0xBC, 0x12, 0xFF, 0x74];
            let register = 0x4;
            let starting_address = 0x834;
            let starting_address_usize = starting_address as usize;
            interpreter.register_i = starting_address;
            for i in 0..=register {
                interpreter.registers[i] = register_values[i];
            }

            interpreter.handle_opcode(Opcode::StoreRegisters(register));

            assert_eq!(interpreter.ram[starting_address_usize - 0x1], 0x0, "Ram location before starting address modified.");
            assert_eq!(interpreter.ram[starting_address_usize + register + 0x1], 0x0, "Ram location past modification area modified.");
            assert_eq!(interpreter.register_i, starting_address, "Register I value modified.");

            for i in 0..=register {
                assert_eq!(interpreter.ram[starting_address_usize + i], register_values[i], "Register value not stored.");
            }

            for i in 0..=register {
                assert_eq!(interpreter.registers[i], register_values[i], "Register value modified.");
            }
        }

        #[test]
        fn handle_load_registers_opcode() {
            let mut interpreter = Interpreter::new();

            let ram_values = &[0x32, 0xBC, 0x12, 0xFF, 0x74, 0x92, 0x11, 0xF0];
            let register = 0x7;
            let starting_address = 0x900;
            let starting_address_usize = starting_address as usize;
            interpreter.register_i = starting_address;
            for i in 0..=register {
                interpreter.ram[starting_address_usize + i] = ram_values[i];
            }

            interpreter.handle_opcode(Opcode::LoadRegisters(register));

            assert_eq!(interpreter.registers[register + 0x1], 0x0, "Register after modification area modified.");
            assert_eq!(interpreter.register_i, starting_address, "Register I value modified.");

            for i in 0..=register {
                assert_eq!(interpreter.registers[i], ram_values[i], "Register value not loaded.");
            }

            for i in 0..=register {
                assert_eq!(interpreter.ram[starting_address_usize + i], ram_values[i], "Ram value modified.");
            }
        }

        #[test]
        fn handle_load_register_i_opcode() {
            let mut interpreter = Interpreter::new();

            let address = 0x246;
            interpreter.handle_opcode(Opcode::LoadRegisterI(address));
            assert_eq!(interpreter.register_i, address, "Register I not updated.");
        }

        #[test]
        fn handle_jump_address_v0_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0x0;
            let value = 0x34;
            let address = 0x111;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::JumpAddrV0(address));
            assert_eq!(interpreter.program_counter, value as u16 + address, "Program counter not updated.");
            assert_eq!(interpreter.registers[register], value, "Register 0 modified.");
        }

        #[test]
        fn handle_load_delay_timer_opcode() {
            let mut interpreter = Interpreter::new();

            let value = 0x54;
            let register = 0x6;
            interpreter.delay_timer = value;
            interpreter.handle_opcode(Opcode::LoadDelayTimer(register));
            assert_eq!(interpreter.registers[register], value, "Register not updated.");
            assert_eq!(interpreter.delay_timer, value, "Delay timer modified.");
        }

        #[test]
        fn handle_set_delay_timer_opcode() {
            let mut interpreter = Interpreter::new();

            let value = 0x20;
            let register = 0x9;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::SetDelayTimer(register));
            assert_eq!(interpreter.delay_timer, value, "Delay timer not updated.");
            assert_eq!(interpreter.registers[register], value, "Register modified.");
        }

        #[test]
        fn handle_set_sound_timer_opcode() {
            let mut interpreter = Interpreter::new();

            let value = 0x77;
            let register = 0x4;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::SetSoundTimer(register));
            assert_eq!(interpreter.sound_timer, value, "Sound timer not updated.");
            assert_eq!(interpreter.registers[register], value, "Register modified.");
        }

        #[test]
        fn handle_add_register_i_opcode() {
            let mut interpreter = Interpreter::new();

            let value = 0x52;
            let starting_address: u16 = 0x894;
            let register = 0x7;
            interpreter.register_i = starting_address;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::AddRegisterI(register));
            assert_eq!(interpreter.register_i, starting_address + value as u16, "Register I not updated.");
            assert_eq!(interpreter.registers[register], value, "Register modified.");
        }

        #[test]
        fn handle_add_registers_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x0;
            let second_register = 0x1;
            let first_value = 0x4;
            let second_value = 0x8;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::AddRegisters(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], first_value + second_value, "Basic addition failed.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.register_f, false, "Overflow bit incorrectly set.");

            let first_value = 0xEE;
            let second_value = 0xDD;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::AddRegisters(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], 0xCB, "Addition with overflow failed.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.register_f, true, "Overflow bit incorrectly not set.");
        }

        #[test]
        fn handle_subtract_from_register_opcodes() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x7;
            let second_register = 0x6;
            let first_value: u8 = 0xF;
            let second_value: u8 = 0x2;
            let third_value: u8 = 0xE;
            let difference = first_value - second_value;
            let underflowing_difference = difference.overflowing_sub(third_value).0;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::SubtractFromFirstRegister(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], difference, "Basic subtraction failed.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.register_f, false, "Borrow bit incorrectly set.");

            interpreter.registers[second_register] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromFirstRegister(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], underflowing_difference, "Underflowing subtraction failed.");
            assert_eq!(interpreter.registers[second_register], third_value, "Second register modified.");
            assert_eq!(interpreter.register_f, true, "Borrow bit incorrectly not set.");

            interpreter.registers[second_register] = first_value;
            interpreter.registers[first_register] = second_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], difference, "Basic subtraction failed.");
            assert_eq!(interpreter.registers[second_register], first_value, "Second register modified.");
            assert_eq!(interpreter.register_f, false, "Borrow bit incorrectly set.");

            interpreter.registers[second_register] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(second_register, first_register));
            assert_eq!(interpreter.registers[second_register], underflowing_difference, "Underflowing subtraction failed.");
            assert_eq!(interpreter.registers[first_register], difference, "First register modified.");
            assert_eq!(interpreter.register_f, true, "Borrow bit incorrectly not set.");
        }

        #[test]
        fn handle_bit_shift_right_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0x3;
            let value = 0xAA;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::BitShiftRight(register));
            assert_eq!(interpreter.registers[register], value >> 1, "Bit shift right failed.");
            assert_eq!(interpreter.register_f, false, "Shift bit incorrectly set");

            interpreter.handle_opcode(Opcode::BitShiftRight(register));
            assert_eq!(interpreter.registers[register], value >> 2, "Bit shift right failed.");
            assert_eq!(interpreter.register_f, true, "Shift bit incorrectly not set");
        }

        #[test]
        fn handle_bit_shift_left_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0xC;
            let value = 0xAA;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::BitShiftLeft(register));
            assert_eq!(interpreter.registers[register], value << 1, "Bit shift left failed.");
            assert_eq!(interpreter.register_f, true, "Shift bit incorrectly not set");

            interpreter.handle_opcode(Opcode::BitShiftLeft(register));
            assert_eq!(interpreter.registers[register], value << 2, "Bit shift left failed.");
            assert_eq!(interpreter.register_f, false, "Shift bit incorrectly set");
        }

        #[test]
        fn handle_binary_encoded_decimal_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0xA;
            let value = 0xDA;
            let starting_address = 0x783;
            interpreter.registers[register] = value;
            interpreter.register_i = starting_address;
            interpreter.handle_opcode(Opcode::BinaryCodedDecimal(register));
            assert_eq!(interpreter.ram[starting_address as usize], 0x2, "Binary encoded decimal hundreds digit incorrect.");
            assert_eq!(interpreter.ram[(starting_address + 0x1) as usize], 0x1, "Binary encoded decimal tens digit incorrect.");
            assert_eq!(interpreter.ram[(starting_address + 0x2) as usize], 0x8, "Binary encoded decimal units digit incorrect.");
            assert_eq!(interpreter.registers[register], value, "Register modified.");
            assert_eq!(interpreter.register_i, starting_address, "Register I modified.");
        }

        #[test]
        fn handle_call_addr_oppcode() {
            let mut interpreter = Interpreter::new();

            let original_program_counter = 0x7E2;
            let first_address = 0x999;
            let second_address = 0x432;
            let current_program_counter = original_program_counter;
            interpreter.program_counter = current_program_counter;
            interpreter.handle_opcode(Opcode::CallAddr(first_address));
            assert_eq!(interpreter.program_counter, first_address, "Program counter not updated.");
            assert_eq!(interpreter.stack[interpreter.stack_pointer - 1], current_program_counter, "Program counter not placed on the stack.");
            assert_eq!(interpreter.stack_pointer, 0x1, "Stack pointer not incremented.");

            let current_program_counter = first_address;
            interpreter.handle_opcode(Opcode::SystemAddr(second_address));
            assert_eq!(interpreter.program_counter, second_address, "Program counter not updated.");
            assert_eq!(interpreter.stack[interpreter.stack_pointer - 1], current_program_counter, "Program counter not placed on the stack.");
            assert_eq!(interpreter.stack_pointer, 0x2, "Stack pointer not incremented.");
            assert_eq!(interpreter.stack[interpreter.stack_pointer - 2], original_program_counter, "Previous address on the stack modified.");
        }

        #[test]
        fn handle_return_opcode() {
            let mut interpreter = Interpreter::new();

            let top_address = 0x234;
            let bottom_address = 0x123;
            interpreter.stack_pointer = 0x2;
            interpreter.stack[0x0] = bottom_address;
            interpreter.stack[0x1] = top_address;
            interpreter.handle_opcode(Opcode::Return);
            assert_eq!(interpreter.program_counter, top_address, "Return from subroutine failed after one call.");
            assert_eq!(interpreter.stack_pointer, 0x1, "Stack pointer not decremented.");
            assert_eq!(interpreter.stack[interpreter.stack_pointer], top_address, "Top address on the stack modified.");
            assert_eq!(interpreter.stack[interpreter.stack_pointer - 1], bottom_address, "Bottom address on the stack modified.");

            interpreter.handle_opcode(Opcode::Return);
            assert_eq!(interpreter.program_counter, bottom_address, "Return from subroutine failed after two calls.");
            assert_eq!(interpreter.stack_pointer, 0x0, "Stack pointer not decremented.");
            assert_eq!(interpreter.stack[interpreter.stack_pointer + 1], top_address, "Top address on the stack modified.");
            assert_eq!(interpreter.stack[interpreter.stack_pointer], bottom_address, "Bottom address on the stack modified.");
        }

        #[test]
        fn handle_set_register_i_hex_sprite_location_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0x3;
            let value = 0xE;
            interpreter.registers[register] = value;
            interpreter.handle_opcode(Opcode::SetIHexSpriteLocation(register));
            assert_eq!(interpreter.register_i, 0x46, "Register I not set correctly.");
            assert_eq!(interpreter.registers[register], value, "Register value modified.");
        }

        #[test]
        fn handle_skip_key_pressed_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0xA;
            let value = 0x5;
            let other_key = 0x8;
            interpreter.registers[register] = value;
            interpreter.keyboard.insert(other_key);
            interpreter.handle_opcode(Opcode::SkipKeyPressed(register));
            assert_eq!(interpreter.program_counter, 0x0, "Program counter incremented incorrectly.");
            assert_eq!(interpreter.registers[register], value, "Register value modified.");
            assert!(interpreter.keyboard.contains(&other_key), "Keys pressed modified.");
            assert_eq!(interpreter.keyboard.len(), 0x1, "Number of keys pressed is incorrect.");

            interpreter.keyboard.insert(value);
            interpreter.handle_opcode(Opcode::SkipKeyPressed(register));
            assert_eq!(interpreter.program_counter, PROGRAM_COUNTER_INCREMENT, "Program counter not incremented.");
            assert_eq!(interpreter.registers[register], value, "Register value modified.");
            assert!(interpreter.keyboard.contains(&value), "Keys pressed modified.");
            assert!(interpreter.keyboard.contains(&other_key), "Keys pressed modified.");
            assert_eq!(interpreter.keyboard.len(), 0x2, "Number of keys pressed is incorrect.");
        }

        #[test]
        fn handle_skip_key_not_pressed_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0xA;
            let value = 0x5;
            let other_key = 0x8;
            interpreter.registers[register] = value;
            interpreter.keyboard.insert(value);
            interpreter.keyboard.insert(other_key);
            interpreter.handle_opcode(Opcode::SkipKeyNotPressed(register));
            assert_eq!(interpreter.program_counter, 0x0, "Program counter incremented incorrectly.");
            assert_eq!(interpreter.registers[register], value, "Register value modified.");
            assert!(interpreter.keyboard.contains(&value), "Keys pressed modified.");
            assert!(interpreter.keyboard.contains(&other_key), "Keys pressed modified.");
            assert_eq!(interpreter.keyboard.len(), 0x2, "Number of keys pressed is incorrect.");

            interpreter.keyboard.remove(&value);
            interpreter.handle_opcode(Opcode::SkipKeyNotPressed(register));
            assert_eq!(interpreter.program_counter, PROGRAM_COUNTER_INCREMENT, "Program counter not incremented.");
            assert_eq!(interpreter.registers[register], value, "Register value modified.");
            assert!(interpreter.keyboard.contains(&other_key), "Keys pressed modified.");
            assert_eq!(interpreter.keyboard.len(), 0x1, "Number of keys pressed is incorrect.");
        }

        #[test]
        fn handle_load_key_press_opcode() {
            let mut interpreter = Interpreter::new();

            let register = 0x7;
            let program_start_usize = PROGRAM_START_ADDRESS as usize;
            interpreter.program_counter = PROGRAM_START_ADDRESS;
            interpreter.ram[program_start_usize] = 0xAA;
            interpreter.ram[program_start_usize + 1] = 0xAA;
            interpreter.handle_opcode(Opcode::LoadKeyPress(register));
            interpreter.handle_cycle();
            assert_eq!(interpreter.register_i, 0x0, "Opcode handled when execution should have been paused.");
            assert_eq!(interpreter.program_counter, PROGRAM_START_ADDRESS, "Program counter incremented incorrectly.");
            assert!(interpreter.should_wait_for_key, "Not waiting for key press.");
            assert_eq!(interpreter.wait_for_key_register, register, "Wrong register set for loading.");

            interpreter.handle_key_press(Keycode::Q);
            interpreter.handle_cycle();
            assert_eq!(interpreter.register_i, 0xAAA, "Opcode not handled.");
            assert_eq!(interpreter.program_counter, PROGRAM_START_ADDRESS + PROGRAM_COUNTER_INCREMENT, "Program counter not incremented.");
            assert!(!interpreter.should_wait_for_key, "Waiting for key press.");
            assert_eq!(interpreter.registers[register], 0x4, "Wrong key loaded into register.");
        }
    }
}
