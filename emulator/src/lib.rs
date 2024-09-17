//! A Chip-8 emulator
//!
//! This library does not provide a display, sound, or input handling by default. That functionality
//! is provided by the user by defining a type that implements the `Chip8IO` trait. Alternatively,
//! the `default_io` feature can be used to enable a basic implementation.
//!
//! See the `default_io` module for an example of an implementation of `Chip8IO`.
//!
//! # Examples
//! Example usage of the emulator with a dummy implementation of `Chip8IO`:
//!
//! ```rust
//! # /*
//! use chip8::config::Log;
//!
//! struct Io;
//!
//! // First implement `Chip8IO`
//! impl chip8::Chip8IO for Io {
//!     fn draw(&mut self, _pixels: &[bool]) {}
//!     fn get_keys(&mut self) -> chip8::Keys {
//!         [false; 16]
//!     }
//!     fn play_sound(&mut self) {}
//!     fn should_close(&self) -> bool {
//!         false
//!     }
//! }
//!
//! // Create a program
//! let program = &[0x61, 0xFF, 0xF1, 0x18];
//! // Initialize I/O state
//! let mut io = Io;
//! // Run the program with the emulator
//! chip8::run(program, &mut io, Log::Disabled).unwrap();
//!
//! # */
//! ```
//!
//! Or using the default implementation (requires the `default_io` feature):
//!
//! ```rust
//! # /*
//! use chip8::default_io::Io;
//! use chip8::config::Log;
//!
//! let program = &[0x61, 0xFF, 0xF1, 0x18];
//! let mut io = Io::new("beep.wav");
//! chip8::run(program, &mut io, Log::Disabled).unwrap();
//! # */
//! ```

// FIXME: In every pong game, pixels are not being cleared, causing buildup of powered pixels
//        Fix the problem and add more tests for drawing
//        If the screen width and height are set to 64 and 32, invalid pixel errors are thrown
//        This may be related to the problem

// TODO: Apply clippy changes
//
// TODO: Make it a command line option to change display size (everything seems to just work)

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![deny(missing_docs, missing_debug_implementations, clippy)]
#![cfg_attr(feature = "clippy", deny(missing_docs_in_private_items))]

/// The width of the display
pub const SCREEN_WIDTH: usize = 128;
/// The height of the display
pub const SCREEN_HEIGHT: usize = 64;

#[macro_use]
extern crate error_chain;
extern crate rand;
#[macro_use]
extern crate log;

#[cfg(test)]
mod tests;

mod register;
mod io;
mod fontset;
mod instruction;
mod interpreter;
mod errors;
mod cpu;
mod utils;
pub mod config;
#[cfg(feature = "default_io")]
pub mod default_io;

use std::time::{Duration, Instant};
use std::fmt;

use register::Registers;
use io::Io;
use fontset::{FONTSET, FONTSET_START};
use config::Log;

pub use errors::*;
pub use io::Keys;

/// The size of memory
const MEMORY: usize = 4096;
/// Where to put the program in memory
const PROGRAM_START: usize = 0x200;
/// The number of times to count down the timers per second
const TIMER_SPEED: u64 = 60;

/// A trait implemented by types used for doing I/O
pub trait Chip8IO {
    /// Draws the array to the screen based on the following:
    ///
    /// - The array is a matrix of pixels of width `SCREEN_WIDTH` and height `SCREEN_HEIGHT`, stored
    /// in row-major order
    ///
    /// - A pixel is `true` if it is on, `false` otherwise
    ///
    /// - The top left corner is pixel (0, 0), and the bottom right corner is pixel
    /// (`SCREEN_WIDTH - 1`, `SCREEN_HEIGHT - 1`)
    fn draw(&mut self, pixels: &[bool]);
    /// Returns the current state of of the keyboard
    fn get_keys(&mut self) -> Keys;
    /// Plays a sound
    fn play_sound(&mut self);
    /// Returns whether the emulator should exit
    fn should_close(&self) -> bool;
}

/// Creates a Chip-8 emulator and runs it. Returns an error in the case of something invalid, for
/// example an invalid opcode. Requires a type that implements `Chip8IO` to do I/O (see `Chip8IO`
/// for more). Logging can be enabled with the `log` argument.
pub fn run<T: Chip8IO>(program: &[u8], io: &mut T, log: Log) -> Result<()> {
    let mut chip8 = Chip8::new(program, log).chain_err(|| "Failed to initialize emulator")?;
    // The time when the next timer update should happen
    // Used for capping the timer speed
    let mut next_tick = Instant::now();

    loop {
        // Run a CPU cycle
        chip8.cycle(io)?;

        // Detect end conditions
        if chip8.program_ended() | io.should_close() {
            break;
        }

        if Instant::now() > next_tick {
            // Run the next cycle `1000 / HERTZ` milliseconds from now
            next_tick += Duration::from_millis(1000 / TIMER_SPEED);

            chip8.update_timers(io);
        }
    }

    Ok(())
}

/// A Chip-8 emulator
struct Chip8 {
    /// RAM
    memory: [u8; MEMORY],
    /// The stack; used for storing addresses to return to from subroutines
    stack: Vec<u16>,
    /// Register state
    registers: Registers,
    /// I/O state
    io: Io,
    /// A general purpose timer that counts down at 60 hz
    delay_timer: u8,
    /// A timer that counts down at 60 hz
    /// A sound is played when this timer reaches zero
    sound_timer: u8,
    /// Whether the program has ended
    program_ended: bool,
    /// Whether to log things
    log: Log,
}

impl Chip8 {
    /// Initializes and returns a Chip-8 emulator
    fn new(program: &[u8], log: Log) -> Result<Chip8> {
        let mut memory = [0; MEMORY];

        // Make sure the fontset doesn't go into program memory
        assert!(0x50 + FONTSET.len() < PROGRAM_START, "Fontset too large");

        // Load fontset into memory starting at address 0x50
        memory[FONTSET_START..FONTSET_START + FONTSET.len()].copy_from_slice(FONTSET);

        let program_memory_size = memory.len() - PROGRAM_START;

        if program.len() >= program_memory_size {
            bail!(ErrorKind::ProgramTooLarge(program_memory_size, program.len()));
        }

        // Load the program into memory starting at address 0x200
        memory[PROGRAM_START..PROGRAM_START + program.len()].copy_from_slice(program);

        Ok(Chip8 {
            memory: memory,
            stack: Vec::new(),
            registers: Registers::new(),
            io: Io::new(),
            delay_timer: 0,
            sound_timer: 0,
            program_ended: false,
            log: log,
        })
    }

    /// Returns whether the program has ended
    fn program_ended(&self) -> bool {
        self.program_ended
    }

    /// Updates the timers, and plays a sound if the sound timer reaches zero
    fn update_timers<T: Chip8IO>(&mut self, io: &mut T) {
        // Update the delay timer
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        // Update the sound timer, and play a sound if it reaches zero
        if self.sound_timer > 0 {
            self.sound_timer -= 1;

            if self.sound_timer == 0 {
                io.play_sound();
            }
        }
    }
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.memory.fmt(f)?;
        self.stack.fmt(f)?;
        self.registers.fmt(f)?;
        self.io.fmt(f)?;
        self.stack.fmt(f)?;
        self.delay_timer.fmt(f)?;
        self.program_ended.fmt(f)?;

        Ok(())
    }
}
