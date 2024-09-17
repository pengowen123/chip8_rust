//! Storage of I/O state

use std::fmt;

use super::{SCREEN_WIDTH, SCREEN_HEIGHT};

/// The amount of pixels in the display
pub const PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// I/O state, including graphics, sound, and keyboard input
pub struct Io {
    /// The pixels of the display
    pixels: [bool; PIXELS],
    /// Whether the pixels should be drawn
    draw_flag: bool,
    /// Keys being pressed
    keys: Keys,
}

impl fmt::Debug for Io {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.draw_flag.fmt(f)?;
        self.keys.fmt(f)?;
        self.pixels.fmt(f)?;

        Ok(())
    }
}

/// The state of keyboard input
pub type Keys = [bool; 16];

impl Io {
    /// Initializes and returns the I/O state
    pub fn new() -> Io {
        Io {
            pixels: [false; PIXELS],
            draw_flag: true,
            keys: [false; 16],
        }
    }

    /// Clears the screen
    pub fn clear_screen(&mut self) {
        self.pixels = [false; PIXELS];
        self.set_draw_flag();
    }

    /// Returns whether the key is pressed
    pub fn is_key_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    /// Sets the draw flag to true (causes the screen to be redrawn)
    pub fn set_draw_flag(&mut self) {
        self.draw_flag = true;
    }

    /// Returns the draw flag (whether the screen should be redrawn)
    pub fn draw_flag(&self) -> bool {
        self.draw_flag
    }

    /// Returns a mutable reference to the pixel at the given index
    pub fn get_mut_pixel(&mut self, index: usize) -> &mut bool {
        &mut self.pixels[index]
    }

    /// Returns a slice containing the pixels of the screen
    pub fn pixels(&self) -> &[bool] {
        &self.pixels
    }

    /// Sets the keyboard input state
    pub fn set_keys(&mut self, keys: Keys) {
        self.keys = keys;
    }

    /// Waits for a key to be pressed, and returns it
    /// Requires a type that implements `Chip8IO` to do I/O (see `Chip8IO` for more)
    pub fn wait_key<T: ::Chip8IO>(&self, mut io: &mut T) -> u8 {
        loop {
            let new_keys = io.get_keys();

            for (i, key) in new_keys.iter().enumerate() {
                // If the key was not pressed, but it was just pressed, return the key
                if !self.keys[i] && *key {
                    return i as u8;
                }
            }
        }
    }
}
