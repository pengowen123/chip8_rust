extern crate chip8;

use chip8::config::Log;

struct Io;

// First implement `Chip8IO`
impl chip8::Chip8IO for Io {
    fn draw(&mut self, _pixels: &[bool]) {}
    fn get_keys(&mut self) -> chip8::Keys {
        [false; 16]
    }
    fn play_sound(&mut self) {}
    fn should_close(&self) -> bool {
        false
    }
}

fn main() {
    // Create a program
    let program = &[0x61, 0xFF, 0xF1, 0x18];
    // Initialize I/O state
    let mut io = Io;
    // Run the program with the emulator
    chip8::run(program, &mut io, Log::Enabled).unwrap();
}
