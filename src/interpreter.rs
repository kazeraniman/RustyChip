use rand::random;
use crate::opcodes::Opcode;

const BYTE_MASK: u16 = 0xFF;

struct Interpreter {
    ram: [u8; 4096],
    registers: [u8; 16],
    register_i: u16,
    register_f: u8, // TODO: Confirm the size
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    stack_pointer: u8
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            ram: [0; 4096],
            registers: [0; 16],
            register_i: 0,
            register_f: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0,
            stack_pointer: 0,
        }
    }

    fn handle_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::SystemAddr(_) => {}
            Opcode::ClearScreen => {}
            Opcode::Return => {}
            Opcode::JumpAddr(address) => self.jump_addr(address),
            Opcode::CallAddr(_) => {}
            Opcode::SkipRegisterEqualsValue(register, value) => self.skip_register_equals_value(register, value),
            Opcode::SkipRegisterNotEqualsValue(register, value) => self.skip_register_not_equals_value(register, value),
            Opcode::SkipRegistersEqual(first_register, second_register) => self.skip_registers_equal(first_register, second_register),
            Opcode::LoadValue(register, value) => self.load_value(register, value),
            Opcode::AddValue(register, value) => self.add_value(register, value),
            Opcode::LoadRegisterValue(first_register, second_register) => self.load_register_value(first_register, second_register),
            Opcode::Or(first_register, second_register) => self.or(first_register, second_register),
            Opcode::And(first_register, second_register) => self.and(first_register, second_register),
            Opcode::Xor(first_register, second_register) => self.xor(first_register, second_register),
            Opcode::AddRegisters(_, _) => {}
            Opcode::SubtractFromFirstRegister(_, _) => {}
            Opcode::BitShiftRight(_, _) => {}
            Opcode::SubtractFromSecondRegister(_, _) => {}
            Opcode::BitShiftLeft(_, _) => {}
            Opcode::SkipRegistersNotEqual(first_register, second_register) => self.skip_registers_not_equal(first_register, second_register),
            Opcode::SetRegisterI(_) => {}
            Opcode::JumpAddrV0(_) => {}
            Opcode::Random(register, value) => self.random(register, value),
            Opcode::Draw(_, _, _) => {}
            Opcode::SkipKeyPressed(_) => {}
            Opcode::SkipKeyNotPressed(_) => {}
            Opcode::LoadDelayTimer(_) => {}
            Opcode::LoadKeyPress(_) => {}
            Opcode::SetDelayTimer(_) => {}
            Opcode::SetSoundTimer(_) => {}
            Opcode::AddRegisterI(_) => {}
            Opcode::SetIHexSpriteLocation(_) => {}
            Opcode::BinaryCodedDecimal(_) => {}
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_interpreter() {
        let interpreter = Interpreter::new();
        assert_eq!(interpreter.register_i, 0);
        assert_eq!(interpreter.register_f, 0);
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
    }

    #[test]
    fn handle_jump_addr_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.handle_opcode(Opcode::JumpAddr(0x381));
        assert_eq!(interpreter.program_counter, 0x381, "Program counter not updated.");
    }

    #[test]
    fn handle_skip_register_equals_value_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.handle_opcode(Opcode::SkipRegisterEqualsValue(0x5, 0xA2));
        assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when register value doesn't match.");
        assert_eq!(interpreter.registers[0x5], 0x0, "Register value modified.");

        interpreter.registers[0x5] = 0xA2;
        interpreter.handle_opcode(Opcode::SkipRegisterEqualsValue(0x5, 0xA2));
        assert_eq!(interpreter.program_counter, 0x2, "Program counter not updated when register value matches.");
        assert_eq!(interpreter.registers[0x5], 0xA2, "Register value modified.");
    }

    #[test]
    fn handle_skip_register_not_equals_value_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0xA] = 0x23;
        interpreter.handle_opcode(Opcode::SkipRegisterNotEqualsValue(0xA, 0x23));
        assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when register value matches.");
        assert_eq!(interpreter.registers[0xA], 0x23, "Register value modified.");

        interpreter.registers[0xA] = 0x24;
        interpreter.handle_opcode(Opcode::SkipRegisterNotEqualsValue(0xA, 0x23));
        assert_eq!(interpreter.program_counter, 0x2, "Program counter not updated when register value doesn't match.");
        assert_eq!(interpreter.registers[0xA], 0x24, "Register value modified.");
    }

    #[test]
    fn handle_skip_registers_equal_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0x1] = 0x4;
        interpreter.registers[0x2] = 0x5;
        interpreter.handle_opcode(Opcode::SkipRegistersEqual(0x1, 0x2));
        assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when registers don't match.");
        assert_eq!(interpreter.registers[0x1], 0x4, "First register value modified.");
        assert_eq!(interpreter.registers[0x2], 0x5, "Second register value modified.");

        interpreter.registers[0x1] = 0x5;
        interpreter.handle_opcode(Opcode::SkipRegistersEqual(0x1, 0x2));
        assert_eq!(interpreter.program_counter, 0x2, "Program counter not updated when registers match.");
        assert_eq!(interpreter.registers[0x1], 0x5, "First register value modified.");
        assert_eq!(interpreter.registers[0x2], 0x5, "Second register value modified.");
    }

    #[test]
    fn handle_skip_registers_not_equal_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0x1] = 0x4;
        interpreter.registers[0x2] = 0x4;
        interpreter.handle_opcode(Opcode::SkipRegistersNotEqual(0x1, 0x2));
        assert_eq!(interpreter.program_counter, 0x0, "Program counter updated when registers match.");
        assert_eq!(interpreter.registers[0x1], 0x4, "First register value modified.");
        assert_eq!(interpreter.registers[0x2], 0x4, "Second register value modified.");

        interpreter.registers[0x1] = 0x5;
        interpreter.handle_opcode(Opcode::SkipRegistersNotEqual(0x1, 0x2));
        assert_eq!(interpreter.program_counter, 0x2, "Program counter not updated when registers don't match.");
        assert_eq!(interpreter.registers[0x1], 0x5, "First register value modified.");
        assert_eq!(interpreter.registers[0x2], 0x4, "Second register value modified.");
    }

    #[test]
    fn handle_load_value_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.handle_opcode(Opcode::LoadValue(0x0, 0x58));
        assert_eq!(interpreter.registers[0x0], 0x58, "Value not loaded into register.");
    }

    #[test]
    fn handle_add_value_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0x6] = 0xAB;
        interpreter.handle_opcode(Opcode::AddValue(0x6, 0x11));
        assert_eq!(interpreter.registers[0x6], 0xBC, "Regular addition failed.");

        interpreter.handle_opcode(Opcode::AddValue(0x6, 0xAA));
        assert_eq!(interpreter.registers[0x6], 0x66, "Overflow not handled. Should truncate.");
    }

    #[test]
    fn handle_load_register_value_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0x1] = 0xDD;
        interpreter.handle_opcode(Opcode::LoadRegisterValue(0x0, 0x1));
        assert_eq!(interpreter.registers[0x0], 0xDD, "Value not loaded into register.");
        assert_eq!(interpreter.registers[0x1], 0xDD, "Original register value modified.");
    }

    #[test]
    fn handle_or_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0x2] = 0xCC;
        interpreter.registers[0x4] = 0xC3;
        interpreter.handle_opcode(Opcode::Or(0x2, 0x4));
        assert_eq!(interpreter.registers[0x2], 0xCF, "Bitwise OR not applied correctly.");
        assert_eq!(interpreter.registers[0x4], 0xC3, "Second register value modified.");
    }

    #[test]
    fn handle_and_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0x6] = 0xAA;
        interpreter.registers[0x3] = 0xCC;
        interpreter.handle_opcode(Opcode::And(0x6, 0x3));
        assert_eq!(interpreter.registers[0x6], 0x88, "Bitwise AND not applied correctly.");
        assert_eq!(interpreter.registers[0x3], 0xCC, "Second register value modified.");
    }

    #[test]
    fn handle_xor_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.registers[0xB] = 0x33;
        interpreter.registers[0xF] = 0x55;
        interpreter.handle_opcode(Opcode::Xor(0xB, 0xF));
        assert_eq!(interpreter.registers[0xB], 0x66, "Bitwise XOR not applied correctly.");
        assert_eq!(interpreter.registers[0xF], 0x55, "Second register value modified.");
    }

    #[test]
    fn handle_random_opcode() {
        let mut interpreter = Interpreter::new();

        // Since the result is random, we basically just check to make sure that it doesn't panic
        interpreter.handle_opcode(Opcode::Random(0x9, 0x53));
    }

    #[test]
    fn store_registers() {
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
    fn load_registers() {
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
}
