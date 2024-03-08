use rand::random;
use crate::opcodes::Opcode;

const PROGRAM_COUNTER_INCREMENT: u16 = 0x2;
const BYTE_MASK: u16 = u8::MAX as u16;
const LEAST_SIGNIFICANT_BIT_MASK: u8 = 0x1;
const MOST_SIGNIFICANT_BIT_MASK: u8 = 0x80;

struct Interpreter {
    ram: [u8; 4096],
    registers: [u8; 16],
    register_i: u16,
    register_f: bool,
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    stack_pointer: usize,
    stack: [u16; 16]
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            ram: [0; 4096],
            registers: [0; 16],
            register_i: 0,
            register_f: false,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0,
            stack_pointer: 0,
            stack: [0; 16]
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
            Opcode::SkipKeyPressed(_) => {}
            Opcode::SkipKeyNotPressed(_) => {}
            Opcode::LoadDelayTimer(register) => self.load_delay_timer(register),
            Opcode::LoadKeyPress(_) => {}
            Opcode::SetDelayTimer(register) => self.set_delay_timer(register),
            Opcode::SetSoundTimer(register) => self.set_sound_timer(register),
            Opcode::AddRegisterI(register) => self.add_register_i(register),
            Opcode::SetIHexSpriteLocation(_) => {}
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
            self.program_counter += 2;
        }
    }

    fn skip_register_not_equals_value(&mut self, register: usize, value: u8) {
        if self.registers[register] != value {
            self.program_counter += 2;
        }
    }

    fn skip_registers_equal(&mut self, first_register: usize, second_register: usize) {
        if self.registers[first_register] == self.registers[second_register] {
            self.program_counter += 2;
        }
    }

    fn skip_registers_not_equal(&mut self, first_register: usize, second_register: usize) {
        if self.registers[first_register] != self.registers[second_register] {
            self.program_counter += 2;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_interpreter() {
        let interpreter = Interpreter::new();
        assert_eq!(interpreter.register_i, 0);
        assert_eq!(interpreter.register_f, false);
        assert_eq!(interpreter.delay_timer, 0);
        assert_eq!(interpreter.sound_timer, 0);
        assert_eq!(interpreter.program_counter, 0);
        assert_eq!(interpreter.stack_pointer, 0);

        for byte in interpreter.ram.iter() {
            assert_eq!(byte, &0);
        }

        for byte in interpreter.registers.iter() {
            assert_eq!(byte, &0);
        }

        for address in interpreter.stack.iter() {
            assert_eq!(address, &0);
        }
    }

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
}
