use std::collections::{HashSet};
use rand::random;
use sdl2::audio::AudioDevice;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use crate::audio::SquareWave;
use crate::opcodes::{Opcode, OpcodeBytes};
use crate::quirks::{ResetVfQuirk, MemoryIncrementQuirk, DisplayWaitQuirk, ClippingQuirk, ShiftingQuirk, JumpingQuirk};

const PROGRAM_START_ADDRESS: u16 = 0x200;
const PROGRAM_COUNTER_INCREMENT: u16 = 0x2;
const BYTE_MASK: u16 = u8::MAX as u16;
const LEAST_SIGNIFICANT_BIT_MASK: u8 = 0x1;
const MOST_SIGNIFICANT_BIT_MASK: u8 = 0x80;
const REGISTER_F: usize = 0xF;
const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;
const SCREEN_SCALE: u32 = 10;
const DRAWING_BUFFER_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;
pub const SCALED_WIDTH: u32 = SCREEN_WIDTH * SCREEN_SCALE;
pub const SCALED_HEIGHT: u32 = SCREEN_HEIGHT * SCREEN_SCALE;
const HEXADECIMAL_DIGIT_SPRITE_LENGTH: u8 = 0x5;
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
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    stack_pointer: usize,
    stack: [u16; 16],
    keyboard: HashSet<u8>,
    should_wait_for_key: bool,
    wait_for_key_register: usize,
    should_wait_for_display_refresh: bool,
    wait_for_display_refresh_data: (usize, usize, u8),
    drawing_buffer: [bool; DRAWING_BUFFER_SIZE],
    audio_device: Option<&'a AudioDevice<SquareWave>>,
    canvas: Option<&'a mut WindowCanvas>,
    quirk_reset_vf: ResetVfQuirk,
    quirk_memory: MemoryIncrementQuirk,
    quirk_display_wait: DisplayWaitQuirk,
    quirk_clipping: ClippingQuirk,
    quirk_shifting: ShiftingQuirk,
    quirk_jumping: JumpingQuirk
}

impl<'a> Interpreter<'a> {
    pub fn new_with_sdl(canvas: Option<&'a mut WindowCanvas>, audio_device: Option<&'a AudioDevice<SquareWave>>, quirk_reset_vf: ResetVfQuirk, quirk_memory: MemoryIncrementQuirk, quirk_display_wait: DisplayWaitQuirk, quirk_clipping: ClippingQuirk, quirk_shifting: ShiftingQuirk, quirk_jumping: JumpingQuirk) -> Interpreter<'a> {
        let mut ram = [0; 4096];
        for i in 0..HEXADECIMAL_DIGIT_SPRITES.len() {
            ram[i] = HEXADECIMAL_DIGIT_SPRITES[i];
        }

        let mut interpreter = Interpreter {
            ram,
            registers: [0; 16],
            register_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0,
            stack_pointer: 0,
            stack: [0; 16],
            keyboard: HashSet::new(),
            should_wait_for_key: false,
            wait_for_key_register: 0,
            should_wait_for_display_refresh: false,
            wait_for_display_refresh_data: (0, 0, 0),
            drawing_buffer: [false; DRAWING_BUFFER_SIZE],
            canvas,
            audio_device,
            quirk_reset_vf,
            quirk_memory,
            quirk_display_wait,
            quirk_clipping,
            quirk_shifting,
            quirk_jumping
        };

        interpreter.clear_screen();

        interpreter
    }

    #[cfg(test)]
    fn new() -> Interpreter<'a> {
        Self::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::default(), DisplayWaitQuirk::default(), ClippingQuirk::default(), ShiftingQuirk::default(), JumpingQuirk::default())
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
            }

            self.keyboard.insert(key);
        }
    }

    pub fn handle_key_release(&mut self, keycode: Keycode) {
        if let Some(key) = Self::get_key_mapping(keycode) {
            self.keyboard.remove(&key);
            if self.should_wait_for_key && self.registers[self.wait_for_key_register] == key {
                self.should_wait_for_key = false;
            }
        }
    }

    pub fn handle_cycle(&mut self) {
        if self.should_wait_for_key || self.should_wait_for_display_refresh {
            return;
        }

        let opcode = OpcodeBytes::build(&self.ram[self.program_counter as usize..=(self.program_counter + 1) as usize]);
        let opcode = opcode.get_opcode();
        self.program_counter += PROGRAM_COUNTER_INCREMENT;
        self.handle_opcode(opcode);
    }

    pub fn handle_frame(&mut self) {
        self.handle_timers();
        if let Some(canvas) = self.canvas.as_mut() {
            canvas.set_draw_color(Interpreter::get_bg_colour());
            canvas.clear();

            let mut pixels = Vec::new();
            for (i, bit) in self.drawing_buffer.iter().enumerate() {
                if !*bit {
                    continue;
                }

                let x = (i as u32 % SCREEN_WIDTH) * SCREEN_SCALE;
                let y = (i as u32 / SCREEN_WIDTH) * SCREEN_SCALE;
                pixels.push(Rect::new(x as i32, y as i32, SCREEN_SCALE, SCREEN_SCALE))
            }

            canvas.set_draw_color(Interpreter::get_fg_colour());
            if let Err(e) = canvas.fill_rects(&pixels) {
                eprintln!("Error drawing: {e}");
            }

            canvas.present();
        }

        if self.should_wait_for_display_refresh {
            self.complete_draw(self.wait_for_display_refresh_data.0, self.wait_for_display_refresh_data.1, self.wait_for_display_refresh_data.2);
            self.should_wait_for_display_refresh = false;
        }
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

    fn get_bg_colour() -> Color {
        Color::RGB(0x0, 0x0, 0x0)
    }

    fn get_fg_colour() -> Color {
        Color::RGB(0x0, 0xFF, 0x0)
    }

    fn handle_reset_quirk(&mut self) {
        match self.quirk_reset_vf {
            ResetVfQuirk::Reset => { self.registers[REGISTER_F] = 0x0; }
            ResetVfQuirk::NoReset => {}
        }
    }

    fn handle_memory_increment_quirk(&mut self) {
        match self.quirk_memory {
            MemoryIncrementQuirk::Increment => { self.register_i += 1; }
            MemoryIncrementQuirk::NoIncrement => {}
        }
    }

    fn handle_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::ClearScreen => self.clear_screen(),
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
            Opcode::BitShiftRight(first_register, second_register) => self.bit_shift_right(first_register, second_register),
            Opcode::SubtractFromSecondRegister(first_register, second_register) => self.bounded_subtraction(second_register, first_register, first_register),
            Opcode::BitShiftLeft(first_register, second_register) => self.bit_shift_left(first_register, second_register),
            Opcode::SkipRegistersNotEqual(first_register, second_register) => self.skip_registers_not_equal(first_register, second_register),
            Opcode::LoadRegisterI(address) => self.load_register_i(address),
            Opcode::JumpAddrV0(address) => self.jump_address_v0(address),
            Opcode::Random(register, value) => self.random(register, value),
            Opcode::Draw(first_register, second_register, length) => {
                match self.quirk_display_wait {
                    DisplayWaitQuirk::Wait => self.draw(first_register, second_register, length),
                    DisplayWaitQuirk::NoWait => self.complete_draw(first_register, second_register, length)
                }
            },
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
        self.handle_reset_quirk();
    }

    fn and(&mut self, first_register: usize, second_register: usize) {
        self.registers[first_register] &= self.registers[second_register];
        self.handle_reset_quirk();
    }

    fn xor(&mut self, first_register: usize, second_register: usize) {
        self.registers[first_register] ^= self.registers[second_register];
        self.handle_reset_quirk();
    }

    fn random(&mut self, register: usize, value: u8) {
        let random_byte: u8 = random();
        self.registers[register] = random_byte & value;
    }

    fn store_registers(&mut self, register: usize) {
        for i in 0..=register {
            let index_adjustment = match self.quirk_memory {
                MemoryIncrementQuirk::Increment => 0,
                MemoryIncrementQuirk::NoIncrement => i
            };

            self.ram[self.register_i as usize + index_adjustment] = self.registers[i];
            self.handle_memory_increment_quirk();
        }
    }

    fn load_registers(&mut self, register: usize) {
        for i in 0..=register {
            let index_adjustment = match self.quirk_memory {
                MemoryIncrementQuirk::Increment => 0,
                MemoryIncrementQuirk::NoIncrement => i
            };

            self.registers[i] = self.ram[self.register_i as usize + index_adjustment];
            self.handle_memory_increment_quirk();
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
        self.registers[first_register] = (sum & max_u8) as u8;
        self.registers[REGISTER_F] = if sum > max_u8 { 1 } else { 0 };
    }

    fn bounded_subtraction(&mut self, minuend_register: usize, subtrahend_register: usize, result_register: usize) {
        let (difference, did_underflow) = self.registers[minuend_register].overflowing_sub(self.registers[subtrahend_register]);
        self.registers[result_register] = difference;
        self.registers[REGISTER_F] = if !did_underflow { 1 } else { 0 };
    }

    fn bit_shift_right(&mut self, first_register: usize, second_register: usize) {
        let target_shift_register = match self.quirk_shifting {
            ShiftingQuirk::Vy => second_register,
            ShiftingQuirk::Vx => first_register
        };
        let will_bit_run_off = if (self.registers[target_shift_register] & LEAST_SIGNIFICANT_BIT_MASK) == 0x1 { 1 } else { 0 };
        self.registers[first_register] = self.registers[target_shift_register] >> 0x1;
        self.registers[REGISTER_F] = will_bit_run_off;
    }

    fn bit_shift_left(&mut self, first_register: usize, second_register: usize) {
        let target_shift_register = match self.quirk_shifting {
            ShiftingQuirk::Vy => second_register,
            ShiftingQuirk::Vx => first_register
        };
        let will_bit_run_off = if ((self.registers[target_shift_register] & MOST_SIGNIFICANT_BIT_MASK) >> 7) == 0x1 { 1 } else { 0 };
        self.registers[first_register] = self.registers[target_shift_register] << 0x1;
        self.registers[REGISTER_F] = will_bit_run_off;
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
        self.register_i = (self.registers[register] * HEXADECIMAL_DIGIT_SPRITE_LENGTH) as u16;
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

    fn clear_screen(&mut self) {
        self.drawing_buffer.fill(false);
        if let Some(canvas) = self.canvas.as_mut() {
            canvas.set_draw_color(Interpreter::get_bg_colour());
            canvas.clear();
        }
    }

    fn draw(&mut self, first_register: usize, second_register: usize, length: u8) {
        self.should_wait_for_display_refresh = true;
        self.wait_for_display_refresh_data = (first_register, second_register, length);
    }

    fn complete_draw(&mut self, first_register: usize, second_register: usize, length: u8) {
        let base_x = self.registers[first_register] as u32 % SCREEN_WIDTH;
        let base_y = self.registers[second_register] as u32 % SCREEN_HEIGHT;
        self.registers[REGISTER_F] = 0;

        for i in 0..length {
            let mut buffer_y = base_y + i as u32;
            match self.quirk_clipping {
                ClippingQuirk::Clip => {
                    if buffer_y >= SCREEN_HEIGHT {
                        continue;
                    }
                }
                ClippingQuirk::Wrap => {
                    buffer_y %= SCREEN_HEIGHT;
                }
            }

            let sprite_byte = self.ram[(self.register_i + i as u16) as usize];
            for j in 0..8 {
                let mut buffer_x = base_x + j;
                match self.quirk_clipping {
                    ClippingQuirk::Clip => {
                        if buffer_x >= SCREEN_WIDTH {
                            continue;
                        }
                    }
                    ClippingQuirk::Wrap => {
                        buffer_x %= SCREEN_WIDTH;
                    }
                }

                let target_bit = (sprite_byte >> (7 - j)) & 1;
                let drawing_buffer_index = (buffer_y * SCREEN_WIDTH + buffer_x) as usize;
                let display_bit = self.drawing_buffer[drawing_buffer_index];

                if display_bit && target_bit == 1 {
                    self.registers[REGISTER_F] = 1;
                }

                let is_set = display_bit ^ (target_bit == 1);
                self.drawing_buffer[drawing_buffer_index] = is_set;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_interpreter() {
        let interpreter = Interpreter::new();
        assert_eq!(interpreter.register_i, 0, "Register I initialized incorrectly.");
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
        interpreter.program_counter = PROGRAM_START_ADDRESS;
        interpreter.handle_cycle();
        assert_eq!(interpreter.register_i, 0xAAA, "Opcode not handled.");
        assert_eq!(interpreter.program_counter, PROGRAM_START_ADDRESS + PROGRAM_COUNTER_INCREMENT, "Program counter not incremented.");

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
    fn handle_frame() {
        let mut interpreter = Interpreter::new();

        interpreter.delay_timer = 0x1;
        interpreter.sound_timer = 0x1;
        interpreter.handle_timers();
        assert_eq!(interpreter.delay_timer, 0x0, "Delay timer not decremented.");
        assert_eq!(interpreter.sound_timer, 0x0, "Sound timer not decremented.");
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
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of key presses stored.");

        // Testing that repeated press doesn't break anything
        interpreter.handle_key_press(Keycode::Q);
        assert!(interpreter.keyboard.contains(q_key_mapping), "Key press not stored.");
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of key presses stored.");

        interpreter.handle_key_press(Keycode::F);
        assert!(interpreter.keyboard.contains(f_key_mapping), "Key press not stored.");
        assert!(interpreter.keyboard.contains(q_key_mapping), "Stored key press removed.");
        assert_eq!(interpreter.keyboard.len(), 2, "Wrong number of key presses stored.");
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
        assert_eq!(interpreter.keyboard.len(), 2, "Wrong number of key presses stored.");

        interpreter.handle_key_release(Keycode::Q);
        assert!(!interpreter.keyboard.contains(q_key_mapping), "Key press stored.");
        assert!(interpreter.keyboard.contains(f_key_mapping), "Key press not stored.");
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of key presses stored.");

        // Testing that repeated release doesn't break anything
        interpreter.handle_key_release(Keycode::Q);
        assert!(!interpreter.keyboard.contains(q_key_mapping), "Key press stored.");
        assert!(interpreter.keyboard.contains(f_key_mapping), "Key press not stored.");
        assert_eq!(interpreter.keyboard.len(), 1, "Wrong number of key presses stored.");

        interpreter.handle_key_release(Keycode::F);
        assert!(!interpreter.keyboard.contains(f_key_mapping), "Key press stored.");
        assert!(!interpreter.keyboard.contains(q_key_mapping), "Key press stored.");
        assert_eq!(interpreter.keyboard.len(), 0, "Wrong number of key presses stored.");
    }

    #[test]
    fn reset_quirk() {
        let mut reset_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::Reset, MemoryIncrementQuirk::default(), DisplayWaitQuirk::default(), ClippingQuirk::default(), ShiftingQuirk::default(), JumpingQuirk::default());
        let mut no_reset_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::NoReset, MemoryIncrementQuirk::default(), DisplayWaitQuirk::default(), ClippingQuirk::default(), ShiftingQuirk::default(), JumpingQuirk::default());

        let first_register = 0x0;
        let second_register = 0x1;
        let first_value = 0xAA;
        let second_value = 0xF0;
        reset_interpreter.registers[first_register] = first_value;
        reset_interpreter.registers[second_register] = second_value;
        reset_interpreter.registers[REGISTER_F] = 0x1;
        no_reset_interpreter.registers[first_register] = first_value;
        no_reset_interpreter.registers[second_register] = second_value;
        no_reset_interpreter.registers[REGISTER_F] = 0x1;
        reset_interpreter.handle_opcode(Opcode::And(first_register, second_register));
        no_reset_interpreter.handle_opcode(Opcode::And(first_register, second_register));
        assert_eq!(reset_interpreter.registers[REGISTER_F], 0x0, "Register F not reset.");
        assert_eq!(no_reset_interpreter.registers[REGISTER_F], 0x1, "Register F reset.");

        reset_interpreter.registers[REGISTER_F] = 0x1;
        no_reset_interpreter.registers[REGISTER_F] = 0x1;
        reset_interpreter.handle_opcode(Opcode::Or(first_register, second_register));
        no_reset_interpreter.handle_opcode(Opcode::Or(first_register, second_register));
        assert_eq!(reset_interpreter.registers[REGISTER_F], 0x0, "Register F not reset.");
        assert_eq!(no_reset_interpreter.registers[REGISTER_F], 0x1, "Register F reset.");

        reset_interpreter.registers[REGISTER_F] = 0x1;
        no_reset_interpreter.registers[REGISTER_F] = 0x1;
        reset_interpreter.handle_opcode(Opcode::Xor(first_register, second_register));
        no_reset_interpreter.handle_opcode(Opcode::Xor(first_register, second_register));
        assert_eq!(reset_interpreter.registers[REGISTER_F], 0x0, "Register F not reset.");
        assert_eq!(no_reset_interpreter.registers[REGISTER_F], 0x1, "Register F reset.");
    }

    #[test]
    fn memory_quirk() {
        let mut increment_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::Increment, DisplayWaitQuirk::default(), ClippingQuirk::default(), ShiftingQuirk::default(), JumpingQuirk::default());
        let mut no_increment_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::NoIncrement, DisplayWaitQuirk::default(), ClippingQuirk::default(), ShiftingQuirk::default(), JumpingQuirk::default());

        let register_values = &[0x32, 0xBC, 0x12, 0xFF, 0x74];
        let register = 0x4;
        let starting_address = 0x834;
        increment_interpreter.register_i = starting_address;
        no_increment_interpreter.register_i = starting_address;
        for i in 0..=register {
            increment_interpreter.registers[i] = register_values[i];
            no_increment_interpreter.registers[i] = register_values[i];
        }

        increment_interpreter.handle_opcode(Opcode::StoreRegisters(register));
        no_increment_interpreter.handle_opcode(Opcode::StoreRegisters(register));

        assert_eq!(increment_interpreter.register_i, starting_address + register as u16 + 1, "Register I value not incremented.");
        assert_eq!(no_increment_interpreter.register_i, starting_address, "Register I value incremented.");
    }

    #[test]
    fn display_wait_quirk() {
        let mut wait_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::default(), DisplayWaitQuirk::Wait, ClippingQuirk::default(), ShiftingQuirk::default(), JumpingQuirk::default());
        let mut no_wait_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::default(), DisplayWaitQuirk::NoWait, ClippingQuirk::default(), ShiftingQuirk::default(), JumpingQuirk::default());

        let first_register = 0x0;
        let second_register = 0x1;
        let sprite = 0xAA;
        let sprite_location = 0x999;
        wait_interpreter.register_i = sprite_location;
        no_wait_interpreter.register_i = sprite_location;
        wait_interpreter.ram[sprite_location as usize] = sprite;
        no_wait_interpreter.ram[sprite_location as usize] = sprite;
        wait_interpreter.handle_opcode(Opcode::Draw(first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH));
        no_wait_interpreter.handle_opcode(Opcode::Draw(first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH));
        assert!(wait_interpreter.should_wait_for_display_refresh, "Not waiting for display refresh.");
        assert!(!no_wait_interpreter.should_wait_for_display_refresh, "Waiting for display refresh.");
        assert_eq!(wait_interpreter.wait_for_display_refresh_data, (first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH), "Wrong data set to wait for display refresh.");
        assert_eq!(no_wait_interpreter.wait_for_display_refresh_data, (0x0, 0x0, 0x0), "Data set to wait for display refresh.");
        assert!(!wait_interpreter.drawing_buffer[0], "Data drawn to buffer.");
        assert!(no_wait_interpreter.drawing_buffer[0], "Data not drawn to buffer.");
    }

    #[test]
    fn shifting_quirk() {
        let mut vy_shift_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::default(), DisplayWaitQuirk::default(), ClippingQuirk::default(), ShiftingQuirk::Vy, JumpingQuirk::default());
        let mut vx_shift_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::default(), DisplayWaitQuirk::default(), ClippingQuirk::default(), ShiftingQuirk::Vx, JumpingQuirk::default());

        let first_register = 0x0;
        let second_register = 0x1;
        let first_value = 0xAA;
        let second_value = 0xF0;
        vy_shift_interpreter.registers[first_register] = first_value;
        vy_shift_interpreter.registers[second_register] = second_value;
        vx_shift_interpreter.registers[first_register] = first_value;
        vx_shift_interpreter.registers[second_register] = second_value;
        vy_shift_interpreter.handle_opcode(Opcode::BitShiftLeft(first_register, second_register));
        vx_shift_interpreter.handle_opcode(Opcode::BitShiftLeft(first_register, second_register));
        assert_eq!(vy_shift_interpreter.registers[first_register], second_value << 1, "Left shift not performed correctly.");
        assert_eq!(vy_shift_interpreter.registers[second_register], second_value, "Second register modified.");
        assert_eq!(vy_shift_interpreter.registers[REGISTER_F], 0x1, "Register F not set.");
        assert_eq!(vx_shift_interpreter.registers[first_register], first_value << 1, "Left shift not performed correctly.");
        assert_eq!(vx_shift_interpreter.registers[second_register], second_value, "Second register modified.");
        assert_eq!(vx_shift_interpreter.registers[REGISTER_F], 0x1, "Register F not set.");

        vy_shift_interpreter.registers[first_register] = first_value;
        vx_shift_interpreter.registers[first_register] = first_value;
        vy_shift_interpreter.handle_opcode(Opcode::BitShiftRight(first_register, second_register));
        vx_shift_interpreter.handle_opcode(Opcode::BitShiftRight(first_register, second_register));
        assert_eq!(vy_shift_interpreter.registers[first_register], second_value >> 1, "Left shift not performed correctly.");
        assert_eq!(vy_shift_interpreter.registers[second_register], second_value, "Second register modified.");
        assert_eq!(vy_shift_interpreter.registers[REGISTER_F], 0x0, "Register F not set.");
        assert_eq!(vx_shift_interpreter.registers[first_register], first_value >> 1, "Left shift not performed correctly.");
        assert_eq!(vx_shift_interpreter.registers[second_register], second_value, "Second register modified.");
        assert_eq!(vx_shift_interpreter.registers[REGISTER_F], 0x0, "Register F not set.");
    }

    #[test]
    fn clipping_quirk() {
        let mut clip_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::default(), DisplayWaitQuirk::default(), ClippingQuirk::Clip, ShiftingQuirk::default(), JumpingQuirk::default());
        let mut wrap_interpreter = Interpreter::new_with_sdl(None, None, ResetVfQuirk::default(), MemoryIncrementQuirk::default(), DisplayWaitQuirk::default(), ClippingQuirk::Wrap, ShiftingQuirk::default(), JumpingQuirk::default());

        let first_register = 0x0;
        let second_register = 0x1;
        let first_value = (SCREEN_WIDTH - 1) as u8;
        let second_value = (SCREEN_HEIGHT - 1) as u8;
        let sprite = 0xFF;
        let sprite_height = 0x2;
        let start_address = 0x888;
        let start_address_usize = start_address as usize;
        clip_interpreter.registers[first_register] = first_value;
        clip_interpreter.registers[second_register] = second_value;
        clip_interpreter.register_i = start_address;
        clip_interpreter.ram[start_address_usize] = sprite;
        clip_interpreter.ram[start_address_usize + 1] = sprite;
        wrap_interpreter.registers[first_register] = first_value;
        wrap_interpreter.registers[second_register] = second_value;
        wrap_interpreter.register_i = start_address;
        wrap_interpreter.ram[start_address_usize] = sprite;
        wrap_interpreter.ram[start_address_usize + 1] = sprite;
        clip_interpreter.complete_draw(first_register, second_register, sprite_height);
        wrap_interpreter.complete_draw(first_register, second_register, sprite_height);

        assert!(clip_interpreter.drawing_buffer[DRAWING_BUFFER_SIZE - 1], "Pre-clip sprite not drawn.");
        assert!(wrap_interpreter.drawing_buffer[DRAWING_BUFFER_SIZE - 1], "Pre-wrap sprite not drawn.");
        assert!(!clip_interpreter.drawing_buffer[SCREEN_WIDTH as usize - 1], "Sprite did not clip on the Y axis.");
        assert!(wrap_interpreter.drawing_buffer[SCREEN_WIDTH as usize - 1], "Sprite did not wrap around on the Y axis.");

        let bottom_row = ((SCREEN_HEIGHT - 1) * SCREEN_WIDTH) as usize;
        for x in 0..7 {
            assert!(!clip_interpreter.drawing_buffer[x], "Sprite did not clip on the X and Y axes.");
            assert!(wrap_interpreter.drawing_buffer[x], "Sprite did not wrap around on the X and Y axes.");
            assert!(!clip_interpreter.drawing_buffer[bottom_row + x], "Sprite did not clip on the X axis.");
            assert!(wrap_interpreter.drawing_buffer[bottom_row + x], "Sprite did not wrap around on the X axis.");
        }
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
            interpreter.registers[REGISTER_F] = 0x1;
            interpreter.handle_opcode(Opcode::Or(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], first_value | second_value, "Bitwise OR not applied correctly.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0x0, "Register F not reset.");
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
            interpreter.registers[REGISTER_F] = 0x1;
            interpreter.handle_opcode(Opcode::And(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], first_value & second_value, "Bitwise AND not applied correctly.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0x0, "Register F not reset.");
        }

        #[test]
        fn handle_xor_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0xB;
            let second_register = 0xE;
            let first_value = 0x33;
            let second_value = 0x55;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.registers[REGISTER_F] = 0x1;
            interpreter.handle_opcode(Opcode::Xor(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], first_value ^ second_value, "Bitwise XOR not applied correctly.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register value modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0x0, "Register F not reset.");
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
            assert_eq!(interpreter.register_i, starting_address + register as u16 + 1, "Register I value not incremented.");

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
            assert_eq!(interpreter.register_i, starting_address + register as u16 + 1, "Register I value not incremented.");

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
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Overflow bit incorrectly set.");

            interpreter.registers[REGISTER_F] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::AddRegisters(REGISTER_F, second_register));
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Overflow bit incorrectly set.");

            interpreter.registers[first_register] = first_value;
            interpreter.registers[REGISTER_F] = second_value;
            interpreter.handle_opcode(Opcode::AddRegisters(first_register, REGISTER_F));
            assert_eq!(interpreter.registers[first_register], first_value + second_value, "Basic addition with register F failed.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Overflow bit incorrectly set.");

            let first_value = 0xEE;
            let second_value = 0xDD;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::AddRegisters(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], 0xCB, "Addition with overflow failed.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Overflow bit incorrectly not set.");

            interpreter.registers[REGISTER_F] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::AddRegisters(REGISTER_F, second_register));
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Overflow bit incorrectly set.");

            interpreter.registers[first_register] = first_value;
            interpreter.registers[REGISTER_F] = second_value;
            interpreter.handle_opcode(Opcode::AddRegisters(first_register, REGISTER_F));
            assert_eq!(interpreter.registers[first_register], 0xCB, "Addition with overflow with register F failed.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Overflow bit incorrectly set.");
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
            let underflow_difference = difference.overflowing_sub(third_value).0;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::SubtractFromFirstRegister(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], difference, "Basic subtraction failed.");
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Borrow bit incorrectly not set.");

            interpreter.registers[second_register] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromFirstRegister(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], underflow_difference, "Underflow subtraction failed.");
            assert_eq!(interpreter.registers[second_register], third_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Borrow bit incorrectly set.");

            interpreter.registers[second_register] = first_value;
            interpreter.registers[first_register] = second_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], difference, "Basic subtraction failed.");
            assert_eq!(interpreter.registers[second_register], first_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Borrow bit incorrectly not set.");

            interpreter.registers[second_register] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(second_register, first_register));
            assert_eq!(interpreter.registers[second_register], underflow_difference, "Underflow subtraction failed.");
            assert_eq!(interpreter.registers[first_register], difference, "First register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Borrow bit incorrectly set.");

            interpreter.registers[first_register] = third_value;
            interpreter.registers[second_register] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(second_register, first_register));
            assert_eq!(interpreter.registers[second_register], 0x0, "Zero-result subtraction failed.");
            assert_eq!(interpreter.registers[first_register], third_value, "First register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Borrow bit incorrectly not set.");

            interpreter.registers[REGISTER_F] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.handle_opcode(Opcode::SubtractFromFirstRegister(REGISTER_F, second_register));
            assert_eq!(interpreter.registers[second_register], second_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Borrow bit incorrectly not set.");

            interpreter.registers[second_register] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromFirstRegister(REGISTER_F, second_register));
            assert_eq!(interpreter.registers[second_register], third_value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Borrow bit incorrectly set.");

            interpreter.registers[REGISTER_F] = first_value;
            interpreter.registers[first_register] = second_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(first_register, REGISTER_F));
            assert_eq!(interpreter.registers[first_register], difference, "Basic subtraction with register F failed.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Borrow bit incorrectly not set.");

            interpreter.registers[REGISTER_F] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(REGISTER_F, first_register));
            assert_eq!(interpreter.registers[first_register], difference, "First register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Borrow bit incorrectly set.");

            interpreter.registers[REGISTER_F] = third_value;
            interpreter.registers[second_register] = third_value;
            interpreter.handle_opcode(Opcode::SubtractFromSecondRegister(second_register, REGISTER_F));
            assert_eq!(interpreter.registers[second_register], 0x0, "Zero-result subtraction failed.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Borrow bit incorrectly not set.");
        }

        #[test]
        fn handle_bit_shift_right_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x3;
            let second_register = 0x5;
            let value = 0xAA;
            interpreter.registers[second_register] = value;
            interpreter.handle_opcode(Opcode::BitShiftRight(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], value >> 1, "Bit shift right failed.");
            assert_eq!(interpreter.registers[second_register], value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Shift bit incorrectly set");

            interpreter.registers[second_register] = interpreter.registers[first_register];
            interpreter.handle_opcode(Opcode::BitShiftRight(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], value >> 2, "Bit shift right failed.");
            assert_eq!(interpreter.registers[second_register], value >> 1, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Shift bit incorrectly not set");

            interpreter.handle_opcode(Opcode::BitShiftRight(REGISTER_F, second_register));
            assert_eq!(interpreter.registers[second_register], value >> 1, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Shift bit incorrectly set");
        }

        #[test]
        fn handle_bit_shift_left_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0xC;
            let second_register = 0xB;
            let value = 0xAA;
            interpreter.registers[second_register] = value;
            interpreter.handle_opcode(Opcode::BitShiftLeft(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], value << 1, "Bit shift left failed.");
            assert_eq!(interpreter.registers[second_register], value, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 1, "Shift bit incorrectly not set");

            interpreter.registers[second_register] = interpreter.registers[first_register];
            interpreter.handle_opcode(Opcode::BitShiftLeft(first_register, second_register));
            assert_eq!(interpreter.registers[first_register], value << 2, "Bit shift left failed.");
            assert_eq!(interpreter.registers[second_register], value << 1, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Shift bit incorrectly set");

            interpreter.handle_opcode(Opcode::BitShiftLeft(REGISTER_F, second_register));
            assert_eq!(interpreter.registers[second_register], value << 1, "Second register modified.");
            assert_eq!(interpreter.registers[REGISTER_F], 0, "Shift bit incorrectly set");
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
        fn handle_call_addr_opcode() {
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
            assert_eq!(interpreter.register_i, 0x0, "Opcode handled when execution should have been paused.");
            assert_eq!(interpreter.program_counter, PROGRAM_START_ADDRESS, "Program counter incremented incorrectly.");
            assert!(interpreter.should_wait_for_key, "Not waiting for key press.");
            assert_eq!(interpreter.registers[register], 0x4, "Wrong key loaded into register.");

            interpreter.handle_key_release(Keycode::Q);
            interpreter.handle_cycle();
            assert_eq!(interpreter.register_i, 0xAAA, "Opcode not handled.");
            assert_eq!(interpreter.program_counter, PROGRAM_START_ADDRESS + PROGRAM_COUNTER_INCREMENT, "Program counter not incremented.");
            assert!(!interpreter.should_wait_for_key, "Waiting for key press.");
            assert_eq!(interpreter.registers[register], 0x4, "Wrong key loaded into register.");
        }

        #[test]
        fn handle_clear_screen_opcode() {
            let mut interpreter = Interpreter::new();

            interpreter.drawing_buffer.iter_mut().for_each(|x| *x = random());
            interpreter.handle_opcode(Opcode::ClearScreen);
            assert_eq!(interpreter.drawing_buffer, [false; DRAWING_BUFFER_SIZE], "Drawing buffer was not cleared.");
        }

        #[test]
        fn handle_draw_opcode() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x0;
            let second_register = 0x2;
            interpreter.handle_opcode(Opcode::Draw(first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH));
            assert!(interpreter.should_wait_for_display_refresh, "Not waiting for display refresh.");
            assert_eq!(interpreter.wait_for_display_refresh_data, (first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH), "Wrong data set to wait for display refresh.");
        }

        #[test]
        fn draw_complete() {
            let mut interpreter = Interpreter::new();

            let first_register = 0x0;
            let second_register = 0x2;
            let first_value = 0x1;
            let second_value = 0x0;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.set_register_i_hex_sprite_location(first_register);
            let first_value = 0x0;
            interpreter.registers[first_register] = first_value;
            interpreter.complete_draw(first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH);

            // Draw a regular sprite
            let ram_values = &HEXADECIMAL_DIGIT_SPRITES[HEXADECIMAL_DIGIT_SPRITE_LENGTH as usize..HEXADECIMAL_DIGIT_SPRITE_LENGTH as usize * 2];
            assert_eq!(interpreter.registers[REGISTER_F], 0x0, "Collision bit incorrectly set.");
            for i in 0..HEXADECIMAL_DIGIT_SPRITE_LENGTH as usize {
                for j in 0..8 {
                    assert_eq!(interpreter.drawing_buffer[i * SCREEN_WIDTH as usize + j], ((ram_values[i] >> (7 - j)) & 1) == 0x1, "Simple drawn value is incorrect.");
                }
            }

            // Draw a sprite that clips at the edge of the screen
            let first_value = 20;
            let second_value = (SCREEN_HEIGHT - 1) as u8;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.complete_draw(first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH);
            assert_eq!(interpreter.registers[REGISTER_F], 0x0, "Collision bit incorrectly set.");
            for i in 0..HEXADECIMAL_DIGIT_SPRITE_LENGTH as usize {
                for j in 0..8 {
                    let drawing_buffer_index = ((i + second_value as usize) * SCREEN_WIDTH as usize) + first_value as usize + j;
                    let target_value = if drawing_buffer_index >= DRAWING_BUFFER_SIZE { false } else { ((ram_values[i] >> (7 - j)) & 1) == 0x1 };
                    assert_eq!(interpreter.drawing_buffer[drawing_buffer_index % DRAWING_BUFFER_SIZE], target_value, "Clipping drawn value is incorrect.");
                }
            }

            // Draw a sprite that wraps around the edge of the screen
            let first_value = SCREEN_WIDTH as u8;
            let second_value = 10;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.complete_draw(first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH);
            assert_eq!(interpreter.registers[REGISTER_F], 0x0, "Collision bit incorrectly set.");
            for i in 0..HEXADECIMAL_DIGIT_SPRITE_LENGTH as usize {
                for j in 0..8 {
                    let drawing_buffer_index = ((i + second_value as usize) * SCREEN_WIDTH as usize) + (first_value % SCREEN_WIDTH as u8) as usize + j;
                    let target_value = ((ram_values[i] >> (7 - j)) & 1) == 0x1;
                    assert_eq!(interpreter.drawing_buffer[drawing_buffer_index], target_value, "Wrapping drawn value is incorrect.");
                }
            }

            // Draw a colliding sprite
            let first_value = 0x0;
            let second_value = 0x0;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.set_register_i_hex_sprite_location(first_register);
            let new_ram_values = &HEXADECIMAL_DIGIT_SPRITES[0..HEXADECIMAL_DIGIT_SPRITE_LENGTH as usize];
            interpreter.complete_draw(first_register, second_register, HEXADECIMAL_DIGIT_SPRITE_LENGTH);
            assert_eq!(interpreter.registers[REGISTER_F], 0x1, "Collision bit incorrectly not set.");
            for i in 0..HEXADECIMAL_DIGIT_SPRITE_LENGTH as usize {
                let sprite_one_byte = ram_values[i];
                let sprite_two_byte = new_ram_values[i];
                for j in 0..8 {
                    let sprite_one_bit = (sprite_one_byte >> (7 - j)) & 1;
                    let sprite_two_bit = (sprite_two_byte >> (7 - j)) & 1;
                    assert_eq!(interpreter.drawing_buffer[i * SCREEN_WIDTH as usize + j], (sprite_one_bit ^ sprite_two_bit) == 0x1, "Overlapping drawn value is incorrect.");
                }
            }

            // Draw a simple colliding sprite to ensure the collision bit is set correctly
            let first_value = 0xA;
            let second_value = 0x0;
            interpreter.registers[first_register] = first_value;
            interpreter.registers[second_register] = second_value;
            interpreter.ram[0xCCC] = 0xFF;
            interpreter.register_i = 0xCCC;
            interpreter.registers[REGISTER_F] = 0x0;
            interpreter.complete_draw(first_register, second_register, 1);
            assert_eq!(interpreter.registers[REGISTER_F], 0x0, "Collision bit incorrectly not set.");

            interpreter.complete_draw(first_register, second_register, 1);
            assert_eq!(interpreter.registers[REGISTER_F], 0x1, "Collision bit incorrectly not set.");
        }
    }
}
