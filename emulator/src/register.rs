//! Storage of registers

type GeneralRegisters = [u8; 16];

/// The registers of the CHIP-8
#[derive(Debug)]
pub struct Registers {
    /// General purpose registers
    general: GeneralRegisters,
    /// Index register, used for accessing memory
    pub index: u16,
    /// Program counter register, points at the instruction being executed
    pub program_counter: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            general: [0; 16],
            index: 0,
            program_counter: ::PROGRAM_START as u16,
        }
    }

    /// Sets the register to the given value
    pub fn set(&mut self, id: u8, value: u8) {
        self.general[id as usize] = value;
    }

    /// Returns the value of the register
    pub fn get(&self, id: u8) -> u8 {
        self.general[id as usize]
    }

    /// Returns a reference to the general purpose registers
    pub fn get_registers(&self) -> &GeneralRegisters {
        &self.general
    }

    pub fn get_mut_registers(&mut self) -> &mut GeneralRegisters {
        &mut self.general
    }

    pub fn get_u16(&self, id: u8) -> u16 {
        self.get(id) as u16
    }
}
