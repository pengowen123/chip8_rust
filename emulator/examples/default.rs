//! This example must be run with the `default_io` feature

extern crate chip8;

#[cfg(feature = "default_io")]
use chip8::default_io::Io;
#[cfg(feature = "default_io")]
use chip8::config::Log;

#[cfg(feature = "default_io")]
fn main() {
    let program = &[0x61, 0xFF, 0xF1, 0x18];
    let mut io = Io::new("beep.wav");
    chip8::run(program, &mut io, Log::Enabled).unwrap();
}

#[cfg(not(feature = "default_io"))]
fn main() {
    panic!("This example must be run with the `default_io` feature");
}
