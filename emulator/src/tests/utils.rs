//! Helpers for testing

use std::{mem, ops};

use ::*;
use super::run_program;

/// A simulated key press, activated at cycle `start` and lasting for `duration` cycles
#[derive(Debug, PartialEq)]
pub struct Keypress {
    key: usize,
    start: usize,
    duration: usize,
}

impl Keypress {
    pub fn new(key: usize, range: ops::Range<usize>) -> Keypress {
        assert!(key < 16, "Invalid keypress: {}", key);

        Keypress {
            key: key,
            start: range.start,
            duration: range.end - range.start,
        }
    }
}

/// Implemented by all types that can be used for I/O by tests
pub trait TestIO {
    fn new(keypresses: Vec<Keypress>) -> Self;
    fn simulate_keypresses(&mut self);
}

/// A struct implements `Chip8IO`
/// Stores internal state for simulating keypresses
/// When a sound is played, this implementation panics, so sound is tested by making a `should_panic`
/// test
pub struct Io {
    pub keys: Keys,
    pub keypresses: Vec<Keypress>,
}

// A simple implementation of `Chip8IO` for use in tests
// Panics when a sound is played
impl Chip8IO for Io {
    fn draw(&mut self, _: &[bool]) {}
    fn get_keys(&mut self) -> Keys {
        self.keys
    }
    fn play_sound(&mut self) {
        panic!("playing sound");
    }
    fn should_close(&self) -> bool {
        false
    }
}

impl TestIO for Io {
    fn new(keypresses: Vec<Keypress>) -> Io {
        Io {
            keys: [false; 16],
            keypresses: keypresses,
        }
    }

    fn simulate_keypresses(&mut self) {
        let mut keypresses = mem::replace(&mut self.keypresses, Vec::new());

        // Unpress expired keypresses
        keypresses = keypresses.into_iter()
            .filter(|k| if k.duration == 0 {
                self.keys[k.key] = false;
                false
            } else {
                true
            })
            .collect();

        self.keypresses = keypresses;

        // Simulate keypresses
        for key in &mut self.keypresses {
            if key.start == 0 && key.duration > 0 {
                self.keys[key.key] = true;
                key.duration -= 1;
            }

            if key.start > 0 {
                key.start -= 1;
            }
        }

    }
}

/// A struct that implements `Chip8IO` used for testing the `WaitKey` instruction
pub struct KeyIO {
    pub get_keys_counter: usize,
}

// A simple implementation of `Chip8IO` for use in the `WaitKey` instruction test
// Increments an internal counter when get_keys is called, and presses the last key when the
// counter reaches 10
impl Chip8IO for KeyIO {
    fn draw(&mut self, _: &[bool]) {}
    fn get_keys(&mut self) -> Keys {
        self.get_keys_counter += 1;

        if self.get_keys_counter >= 10 {
            let mut keys = Keys::default();
            keys[15] = true;

            keys
        } else {
            Default::default()
        }
    }
    fn play_sound(&mut self) {}
    fn should_close(&self) -> bool {
        false
    }
}

impl TestIO for KeyIO {
    fn new(_: Vec<Keypress>) -> Self {
        KeyIO { get_keys_counter: 0 }
    }

    fn simulate_keypresses(&mut self) {}
}

/// A helper macro to create a list of simulated keypresses
macro_rules! keypresses {
    () => {{
        Vec::new()
    }};
    ($($key:tt @ $range:expr),*) => {{
        vec![$(Keypress::new($key, $range)),*]
    }};
}

/// A helper macro to create programs
macro_rules! program {
    () => {{
        Vec::new()
    }};
    ($($opcode:expr),*) => {{
        let mut vec = Vec::new();

        $(
            let opcode = $opcode as u16;
            // Push the first byte of the opcode
            vec.push(((opcode & 0xFF00) >> 8) as u8);
            // Push the second byte of the opcode
            vec.push((opcode & 0xFF) as u8);
         )*

        vec
    }};
}

/// Returns a matrix, represented by a 2 dimensional vector, created from a slice.
/// The matrix is generated assuming the slice is in row-major order, and has the given width and
/// height
pub fn to_matrix<T: Copy>(slice: &[T], width: usize, height: usize) -> Vec<Vec<T>> {
    (0..height)
        .map(|h| slice.iter().cloned().skip(h * width).take(width).collect::<Vec<_>>())
        .collect()
}

// Macro tests:

/// Tests the `program` macro
#[test]
fn test_program_macro() {
    let program = program!(0x1234, 0x5678, 0xABCD);

    assert_eq!(vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD], program);
}

/// Tests the `keypresses` macro
#[test]
fn test_keypresses_macro() {
    let keypresses = keypresses!(0 @ 0..1, 2 @ 2..3, 4 @ 4..5);

    assert_eq!(vec![Keypress::new(0, 0..1), Keypress::new(2, 2..3), Keypress::new(4, 4..5)],
               keypresses);
}

// Simulated keypress tests:

/// Asserts that key 0 is not pressed at cycle 0 (before the range)
#[test]
fn test_simulate_keypress_before() {
    let program = program!();
    let keypresses = keypresses!(0 @ 1..2);
    let (_, io) = run_program::<Io>(&program, Some(keypresses), None);

    assert!(!io.keys[0]);
}

/// Asserts that key 0 is pressed at cycle 0 (start of the range)
#[test]
fn test_simulate_keypress_start() {
    let program = program!(0x00E0);
    let keypresses = keypresses!(0 @ 0..2);
    let (_, io) = run_program::<Io>(&program, Some(keypresses), None);

    assert!(io.keys[0]);
}

/// Asserts that key 0 is pressed at cycle 2 (end of the range)
#[test]
fn test_simulate_keypress_end() {
    let program = program!(0x00E0, 0x00E0);
    let keypresses = keypresses!(0 @ 0..2);
    let (_, io) = run_program::<Io>(&program, Some(keypresses), None);

    assert!(io.keys[0]);
}

/// Asserts that key 0 is not pressed at cycle 3 (after the range)
#[test]
fn test_simulate_keypress_unpress() {
    let program = program!(0x00E0, 0x00E0, 0x00E0);
    let keypresses = keypresses!(0 @ 0..2);
    let (_, io) = run_program::<Io>(&program, Some(keypresses), None);

    assert!(!io.keys[0]);
}

// Helper function tests:

/// Tests the `to_matrix` function
#[test]
fn test_to_matrix() {
    let slice = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let matrix = to_matrix(slice, 3, 3);
    let expected = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

    assert_eq!(expected, matrix);
}
