use crate::opcodes::Opcode;

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
            Opcode::SkipRegisterNotEqualsValue(_, _) => {}
            Opcode::SkipRegistersEqual(_, _) => {}
            Opcode::LoadValue(_, _) => {}
            Opcode::AddValue(_, _) => {}
            Opcode::LoadRegisterValue(_, _) => {}
            Opcode::Or(_, _) => {}
            Opcode::And(_, _) => {}
            Opcode::Xor(_, _) => {}
            Opcode::AddRegisters(_, _) => {}
            Opcode::SubtractFromFirstRegister(_, _) => {}
            Opcode::BitShiftRight(_, _) => {}
            Opcode::SubtractFromSecondRegister(_, _) => {}
            Opcode::BitShiftLeft(_, _) => {}
            Opcode::SkipRegistersNotEqual(_, _) => {}
            Opcode::SetRegisterI(_) => {}
            Opcode::JumpAddrV0(_) => {}
            Opcode::Random(_, _) => {}
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
            Opcode::StoreRegisters(_) => {}
            Opcode::LoadRegisters(_) => {}
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
        assert_eq!(interpreter.program_counter, 0x381);
    }

    #[test]
    fn handle_skip_registers_equal_opcode() {
        let mut interpreter = Interpreter::new();

        interpreter.handle_opcode(Opcode::SkipRegisterEqualsValue(0x5, 0xA2));
        assert_eq!(interpreter.program_counter, 0x0);

        interpreter.registers[0x5] = 0xA2;
        interpreter.handle_opcode(Opcode::SkipRegisterEqualsValue(0x5, 0xA2));
        assert_eq!(interpreter.program_counter, 0x2);
    }
}
