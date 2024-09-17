//! Error handling

#![allow(missing_docs)]

error_chain! {
    errors {
        ProgramTooLarge(program_size: usize, memory_size: usize) {
            description("Program too large")
            display("Program too large: memory is {} bytes, but program was {} bytes",
                    memory_size,
                    program_size)
        }
        InvalidOpcode(opcode: String) {
            description("Invalid opcode")
            display("Invalid opcode: {}", opcode)
        }
        InvalidAddress(address: usize, instruction: &'static str) {
            description("Invalid address")
            display("Invalid address: {} ({})", address, instruction)
        }
        UnknownCharacter(character: u8) {
            description("Unknown character")
            display("No sprite for character: {}", character)
        }
        UnknownKey(key: u8, instruction: &'static str) {
            description("Unknown key")
            display("Unknown key: {} ({})", key, instruction)
        }
        PixelOutOfBounds(x: usize, y: usize) {
            description("Attemped to draw a pixel at invalid coordinates")
            display("Invalid pixel coordinates: ({}, {})", x, y)
        }
    }
}
